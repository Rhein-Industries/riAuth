//! Purpose-bound account proofs and a leased, durable SMTP outbox.
use crate::{
    agent::{Agent, Principal},
    core::{Core, audit, user_by_name, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Session, User, UserView},
    store::Tx,
};
use axum::http::StatusCode;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::PathBuf, time::Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailConfig {
    pub host: String,
    pub port: u16,
    pub from: String,
    #[serde(default)]
    pub security: MailSecurity,
    pub username: Option<String>,
    pub password_file: Option<PathBuf>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailSecurity {
    #[default]
    Tls,
    Starttls,
    Loopback,
}
impl MailConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        let url = url::Url::parse(&format!("https://{}", self.host))?;
        anyhow::ensure!(
            url.host_str().is_some()
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
                && url.username().is_empty()
                && url.password().is_none()
                && url.port().is_none()
                && self.port != 0,
            "Invalid SMTP host or port"
        );
        self.from.parse::<Mailbox>()?;
        anyhow::ensure!(
            self.username.is_some() == self.password_file.is_some(),
            "SMTP authentication requires username and password_file together"
        );
        if matches!(self.security, MailSecurity::Loopback) {
            anyhow::ensure!(
                self.host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback()),
                "Unencrypted SMTP is restricted to a literal loopback address"
            );
        }
        Ok(())
    }
    fn transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        self.validate().map_err(Error::internal)?;
        let builder = match self.security {
            MailSecurity::Tls => {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&self.host).map_err(Error::internal)?
            }
            MailSecurity::Starttls => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.host)
                    .map_err(Error::internal)?
            }
            MailSecurity::Loopback => {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.host)
            }
        }
        .port(self.port)
        .timeout(Some(Duration::from_secs(10)));
        let builder = if let (Some(username), Some(path)) = (&self.username, &self.password_file) {
            let password = crate::config::read_private_secret(path, 4096).map_err(|_| {
                Error::bad("SMTP credential must be a private file of at most 4096 bytes")
            })?;
            let password = password.trim_end_matches(['\r', '\n']);
            if password.is_empty() || password.len() > 4096 {
                return Err(Error::bad("Invalid SMTP credential file"));
            }
            builder.credentials(lettre::transport::smtp::authentication::Credentials::new(
                username.clone(),
                password.into(),
            ))
        } else {
            builder
        };
        Ok(builder.build())
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    Verify,
    Reset,
    Invite,
}
impl Purpose {
    fn name(self) -> &'static str {
        match self {
            Self::Verify => "verify",
            Self::Reset => "reset",
            Self::Invite => "accept",
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Proof {
    purpose: Purpose,
    user_id: String,
    email: String,
    epoch: u64,
    expires_at: u64,
    #[serde(default)]
    groups: BTreeSet<String>,
    creator: Option<String>,
}
/// A disabled invited account remains identifiable after its proof expires or is revoked.
/// The immutable user ID prevents a later account with the same username from inheriting it.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InvitationReservation {
    pub(crate) username: String,
    pub(crate) created_by: String,
    pub(crate) epoch: u64,
}
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProofEnd {
    Used,
    Revoked,
    Replaced,
    Expired,
}
#[derive(Serialize, Deserialize)]
struct ProofOutcome {
    purpose: Purpose,
    reason: ProofEnd,
    retain_until: u64,
}
#[derive(Clone, Serialize, Deserialize)]
struct Delivery {
    id: String,
    proof: String,
    recipient: String,
    subject: String,
    body: Option<String>,
    expires_at: u64,
    created_at: u64,
    next_attempt: u64,
    attempts: u32,
    lease: Option<String>,
    delivered_at: Option<u64>,
    stopped: bool,
}
/// How long an invitation link stays valid.
const INVITATION_SECONDS: u64 = 7 * 86400;
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invitation {
    pub username: String,
    pub email: String,
    pub display_name: String,
    #[serde(default)]
    pub groups: BTreeSet<String>,
}
pub(crate) fn require_mail(core: &Core) -> Result<()> {
    if core.config.mail.is_none() {
        return Err(Error::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "delivery_unavailable",
            "Account email delivery is not configured",
        ));
    }
    Ok(())
}
pub(crate) fn email(value: &str) -> Result<()> {
    crate::core::validate_email(value)?;
    value
        .parse::<lettre::Address>()
        .map_err(|_| Error::bad("Invalid email address"))?;
    Ok(())
}
fn proof_key(user: &User, purpose: Purpose) -> String {
    format!("{}:{}", user.id, purpose.name())
}
fn retire_proof(tx: &Tx<'_>, hash: &str, reason: ProofEnd) -> Result<()> {
    if let Some(proof) = tx.get::<Proof>("account_proofs", hash)? {
        tx.delete("account_proofs", hash)?;
        tx.put(
            "account_proof_outcomes",
            hash,
            &ProofOutcome {
                purpose: proof.purpose,
                reason,
                retain_until: proof.expires_at.max(now()).saturating_add(8 * 86400),
            },
        )?;
    }
    Ok(())
}
/// Retire the current invitation proof while retaining its revoked outcome.
/// A missing current proof is an idempotent no-op.
pub(crate) fn revoke_invitation_proof(tx: &Tx<'_>, user: &User) -> Result<bool> {
    let key = proof_key(user, Purpose::Invite);
    let Some(hash) = tx.get::<String>("account_latest", &key)? else {
        return Ok(false);
    };
    retire_proof(tx, &hash, ProofEnd::Revoked)?;
    tx.delete("account_latest", &key)?;
    Ok(true)
}
fn proof_error(reason: Option<ProofEnd>) -> Error {
    match reason {
        Some(ProofEnd::Expired) => Error::new(
            StatusCode::GONE,
            "account_code_expired",
            "Account link expired; request a new one",
        ),
        Some(ProofEnd::Revoked) => Error::new(
            StatusCode::GONE,
            "account_code_revoked",
            "Invitation was revoked; contact your administrator",
        ),
        Some(ProofEnd::Replaced) => Error::new(
            StatusCode::GONE,
            "account_code_replaced",
            "Account link was replaced by a newer one",
        ),
        Some(ProofEnd::Used) => Error::new(
            StatusCode::GONE,
            "account_code_used",
            "Account link was already used",
        ),
        None => Error::new(
            StatusCode::BAD_REQUEST,
            "account_code_invalid",
            "Invalid account link",
        ),
    }
}
pub(crate) fn enqueue(
    core: &Core,
    tx: &Tx<'_>,
    user: &User,
    purpose: Purpose,
    groups: BTreeSet<String>,
    creator: Option<String>,
) -> Result<()> {
    let recipient = user
        .email
        .as_deref()
        .ok_or_else(|| Error::bad("Account has no email address"))?;
    email(recipient)?;
    let key = proof_key(user, purpose);
    if let Some(old) = tx.get::<String>("account_latest", &key)? {
        let reason = if tx
            .get::<Proof>("account_proofs", &old)?
            .is_some_and(|proof| proof.expires_at <= now())
        {
            ProofEnd::Expired
        } else {
            ProofEnd::Replaced
        };
        retire_proof(tx, &old, reason)?;
    }
    let token = zeroize::Zeroizing::new(crypto::random_token("ri_mail_"));
    let hash = digest(&token);
    let expires_at = now()
        + match purpose {
            Purpose::Reset => 1800,
            Purpose::Verify => 86400,
            Purpose::Invite => INVITATION_SECONDS,
        };
    tx.put(
        "account_proofs",
        &hash,
        &Proof {
            purpose,
            user_id: user.id.clone(),
            email: recipient.into(),
            epoch: user.epoch,
            expires_at,
            groups,
            creator,
        },
    )?;
    tx.put("account_latest", &key, &hash)?;
    let subject = match purpose {
        Purpose::Verify => "Verify your riAuth email",
        Purpose::Reset => "Reset your riAuth password",
        Purpose::Invite => "Your riAuth account invitation",
    };
    // The proof travels in the fragment: opening or scanning the link never sends it.
    let browser_link = format!(
        "Open in your browser: {}/account/{}#token={}\n\n",
        core.config.issuer.trim_end_matches('/'),
        purpose.name(),
        token.as_str()
    );
    let body = format!(
        "{subject}\n\nServer: {}\nAccount: {}\n\n{browser_link}Run: riauth --server {} account {} --token-stdin\nPaste this one-use code when asked:\n{}\n\nExpires at Unix time {expires_at}. A password reset keeps your enrolled MFA factors. If you did not request this message, ignore it.\n",
        core.config.issuer,
        user.username,
        core.config.issuer,
        purpose.name(),
        token.as_str()
    );
    let delivery = Delivery {
        id: crypto::id(),
        proof: hash,
        recipient: recipient.into(),
        subject: subject.into(),
        body: Some(body),
        expires_at,
        created_at: now(),
        next_attempt: now(),
        attempts: 0,
        lease: None,
        delivered_at: None,
        stopped: false,
    };
    tx.put("mail_deliveries", &delivery.id, &delivery)
}
// Store the same throttling state and return the same result for unknown accounts.
fn request_allowed(tx: &Tx<'_>, username: &str, purpose: Purpose) -> Result<bool> {
    let key = digest(&format!("{}\0{username}", purpose.name()));
    let previous = tx
        .get::<(u64, u32, u64)>("mail_limits", &key)?
        .unwrap_or((0, 0, 0));
    let (start, count, previous_at) = if previous.0 + 3600 <= now() {
        (now(), 0, 0)
    } else {
        previous
    };
    if previous_at + 60 > now() || count >= 5 {
        return Ok(false);
    }
    if previous.0 == 0 && tx.collection_count("mail_limits")? >= 10_000 {
        tx.reclaim_expired_limits("mail_limits", now())?;
        if tx.collection_count("mail_limits")? >= 10_000 {
            return Ok(false);
        }
    }
    tx.put("mail_limits", &key, &(start, count + 1, now()))?;
    Ok(true)
}
fn verify_request_in(
    core: &Core,
    tx: &Tx<'_>,
    user: &User,
    session: &Session,
) -> Result<&'static str> {
    if now().saturating_sub(session.identity.auth_time) > 300 {
        return Err(Error::forbidden());
    }
    let allowed = request_allowed(tx, &user.username, Purpose::Verify)?;
    if user.email_verified {
        return Ok("already_verified");
    }
    if !allowed {
        return Ok("cooldown");
    }
    enqueue(core, tx, user, Purpose::Verify, BTreeSet::new(), None)?;
    audit(tx, &user.id, "account.verification.request", &user.id)?;
    Ok("queued")
}
pub(crate) fn pending_invitation_reservation(
    tx: &Tx<'_>,
    user: &User,
) -> Result<Option<InvitationReservation>> {
    if user.enabled
        || user.admin
        || !user.password_hash.is_empty()
        || user.email_verified
        || user.has_passkeys
        || user.totp_secret.is_some()
        || user.totp_pending.is_some()
        || !user.recovery_codes.is_empty()
    {
        return Ok(None);
    }
    if let Some(reservation) =
        tx.get::<InvitationReservation>("invitation_reservations", &user.id)?
    {
        return Ok((reservation.username == user.username
            && reservation.epoch == user.epoch
            && !reservation.created_by.is_empty())
        .then_some(reservation));
    }
    // Legacy invitations predate the durable reservation. A still-stored proof,
    // including one past expiry, can establish their provenance for migration.
    let Some(hash) = tx.get::<String>("account_latest", &proof_key(user, Purpose::Invite))? else {
        return Ok(None);
    };
    let Some(proof) = tx.get::<Proof>("account_proofs", &hash)? else {
        return Ok(None);
    };
    if proof.purpose != Purpose::Invite
        || proof.user_id != user.id
        || proof.epoch != user.epoch
        || user.email.as_deref() != Some(proof.email.as_str())
    {
        return Ok(None);
    }
    Ok(proof
        .creator
        .filter(|creator| !creator.is_empty())
        .map(|created_by| InvitationReservation {
            username: user.username.clone(),
            created_by,
            epoch: user.epoch,
        }))
}

/// Recovery advances account epochs to revoke restored authority. Keep the
/// durable provenance of a still-pending invitation bound to that same account
/// so an authorized administrator can reissue it after restore. An already
/// mismatched reservation remains mismatched and cannot be revived.
pub(crate) fn rebase_invitation_reservation(
    tx: &Tx<'_>,
    user_id: &str,
    username: &str,
    old_epoch: u64,
    new_epoch: u64,
) -> Result<()> {
    if let Some(mut reservation) =
        tx.get::<InvitationReservation>("invitation_reservations", user_id)?
        && reservation.username == username
        && reservation.epoch == old_epoch
    {
        reservation.epoch = new_epoch;
        tx.put("invitation_reservations", user_id, &reservation)?;
    }
    Ok(())
}
/// An invitation's creator as `actor` may see it. Administrators may list every person and
/// agent, and an agent knows itself; otherwise a person needs `user.read` and another
/// agent stays hidden, since agents cannot list agents.
fn visible_inviter(tx: &Tx<'_>, actor: &Principal, id: &str) -> Result<Option<String>> {
    if !actor.agent || actor.id == id {
        return Ok(Some(id.into()));
    }
    if id.starts_with("agent:") {
        return Ok(None);
    }
    Ok(tx
        .get::<User>("users", id)?
        .filter(|user| actor.allows("user.read", &format!("user/{}", user.username)))
        .map(|_| id.into()))
}
/// Whether acceptance could complete now: the checks `account_complete` makes through
/// `creator` and `management::accept_invitation`, without writing. The creator must still
/// be an enabled administrator or active agent with `user.write` on the invitee and
/// `group.members` on every group, and each group must still exist.
fn acceptance_authorized(tx: &Tx<'_>, user: &User, proof: &Proof) -> Result<bool> {
    let Some(id) = proof.creator.as_deref() else {
        return Ok(false);
    };
    let actor = match creator(tx, id) {
        Ok(actor) => actor,
        Err(error) if error.status == StatusCode::FORBIDDEN => return Ok(false),
        Err(error) => return Err(error),
    };
    if !actor.allows("user.write", &format!("user/{}", user.username)) {
        return Ok(false);
    }
    for name in &proof.groups {
        if !actor.allows("group.members", &format!("group/{name}"))
            || !tx
                .get::<crate::model::Group>("groups", name)?
                .is_some_and(|group| &group.name == name)
        {
            return Ok(false);
        }
    }
    Ok(true)
}
fn creator(tx: &Tx<'_>, id: &str) -> Result<Principal> {
    if let Some(name) = id.strip_prefix("agent:") {
        let agent = tx
            .get::<Agent>("agents", name)?
            .ok_or_else(Error::forbidden)?;
        if !crate::agent::authority_active(tx, &agent)? {
            return Err(Error::forbidden());
        }
        Ok(Principal {
            id: id.into(),
            agent: true,
            permissions: agent.permissions,
        })
    } else {
        tx.get::<User>("users", id)?
            .filter(|u| u.enabled && u.admin)
            .ok_or_else(Error::forbidden)?;
        Ok(Principal {
            id: id.into(),
            agent: false,
            permissions: vec![],
        })
    }
}
impl Core {
    pub fn account_verify_request(&self, token: &str) -> Result<Value> {
        require_mail(self)?;
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            verify_request_in(self, tx, &user, &session)?;
            Ok(json!({"accepted":true}))
        })
    }
    pub fn portal_verify_request(&self, sso: Option<&str>) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.portal_session(tx, sso)?;
            require_mail(self)?;
            if now().saturating_sub(session.identity.auth_time) > 300 {
                return Err(Error::new(
                    StatusCode::FORBIDDEN,
                    "reauthentication_required",
                    "Sign in again before requesting a verification email",
                ));
            }
            let status = verify_request_in(self, tx, &user, &session)?;
            Ok(json!({"accepted":true,"status":status}))
        })
    }
    pub fn account_reset_request(&self, username: &str) -> Result<Value> {
        require_mail(self)?;
        validate_name(username)?;
        self.store.write(|tx| {
            if request_allowed(tx, username, Purpose::Reset)? {
                let user = match user_by_name(tx, username) {
                    Ok(u) => Some(u),
                    Err(e) if e.status == axum::http::StatusCode::NOT_FOUND => None,
                    Err(e) => return Err(e),
                };
                // Only a local password can be recovered here. Directory, passkey-only and
                // upstream-only accounts get the same answer and no local password.
                if let Some(user) = user.filter(|u| u.enabled && u.email_verified)
                    && crate::password::Kind::of(tx, &user)? == crate::password::Kind::Local
                {
                    enqueue(self, tx, &user, Purpose::Reset, BTreeSet::new(), None)?;
                }
            }
            Ok(json!({"accepted":true}))
        })
    }
    pub fn account_invite(&self, token: &str, input: Invitation) -> Result<Value> {
        require_mail(self)?;
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            crate::management::invite_user(self, tx, &actor, input)
        })
    }
    pub fn account_invitation_revoke(&self, token: &str, username: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            crate::management::revoke_invitation(tx, &actor, username)
        })
    }
    /// Invited accounts that have not accepted yet, with the state of their current link.
    /// Each part obeys the reader's own read permissions: people by `user.read`, groups by
    /// `group.read`, the inviter by `visible_inviter` and delivery state by the permission
    /// `mail_deliveries` needs. Codes and message bodies are never returned.
    pub fn account_invitations(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let deliveries: std::collections::BTreeMap<String, Delivery> =
                if actor.allows("operations.read", "operations/mail") {
                    tx.list::<Delivery>("mail_deliveries")?
                        .into_iter()
                        .map(|(_, delivery)| (delivery.proof.clone(), delivery))
                        .collect()
                } else {
                    Default::default()
                };
            let mut invitations = Vec::new();
            for (_, user) in tx.list::<User>("users")? {
                if !actor.allows("user.read", &format!("user/{}", user.username)) {
                    continue;
                }
                let Some(reservation) = pending_invitation_reservation(tx, &user)? else {
                    continue;
                };
                // The same conditions account_complete checks before accepting the code.
                let key = proof_key(&user, Purpose::Invite);
                let current = match tx.get::<String>("account_latest", &key)? {
                    Some(hash) => tx
                        .get::<Proof>("account_proofs", &hash)?
                        .filter(|proof| {
                            proof.purpose == Purpose::Invite
                                && proof.user_id == user.id
                                && proof.epoch == user.epoch
                                && user.email.as_deref() == Some(proof.email.as_str())
                        })
                        .map(|proof| (hash, proof)),
                    None => None,
                };
                // Without a current code the last link was revoked, or expired and was
                // cleaned up; either way it no longer works. An unexpired code whose
                // acceptance would be refused is blocked, without saying whose authority failed.
                let status = match &current {
                    Some((_, proof)) if proof.expires_at <= now() => "expired",
                    Some((_, proof)) if !acceptance_authorized(tx, &user, proof)? => "blocked",
                    Some(_) => "pending",
                    None => "inactive",
                };
                let delivery = current
                    .as_ref()
                    .and_then(|(hash, _)| deliveries.get(hash))
                    .map(|d| {
                        let state = if d.delivered_at.is_some() {
                            "sent"
                        } else if d.stopped {
                            "stopped"
                        } else {
                            "queued"
                        };
                        json!({
                            "status": state,
                            "queued_at": d.created_at,
                            "attempts": d.attempts,
                            "delivered_at": d.delivered_at,
                        })
                    });
                let (invited_by, expires_at, groups) = match current {
                    Some((_, proof)) => (
                        proof.creator.unwrap_or(reservation.created_by),
                        Some(proof.expires_at),
                        proof.groups,
                    ),
                    None => (reservation.created_by, None, BTreeSet::new()),
                };
                // Groups follow list_groups: a scoped reader sees only the ones it may read.
                let groups: Vec<_> = groups
                    .into_iter()
                    .filter(|name| actor.allows("group.read", &format!("group/{name}")))
                    .collect();
                invitations.push(json!({
                    "user": UserView::from(&user),
                    "status": status,
                    "expires_at": expires_at,
                    "groups": groups,
                    "invited_by": visible_inviter(tx, &actor, &invited_by)?,
                    "delivery": delivery,
                }));
            }
            Ok(json!({
                "delivery_configured": self.config.mail.is_some(),
                "lifetime": INVITATION_SECONDS,
                "invitations": invitations,
            }))
        })
    }
    pub fn account_complete(
        &self,
        token: String,
        purpose: Purpose,
        password: Option<String>,
    ) -> Result<Value> {
        let token = zeroize::Zeroizing::new(token);
        if !token.starts_with("ri_mail_") || token.len() > 128 {
            return Err(proof_error(None));
        }
        let password = password.map(zeroize::Zeroizing::new);
        let password_hash = match purpose {
            Purpose::Verify if password.is_some() => {
                return Err(Error::bad("Verification does not accept a password"));
            }
            Purpose::Verify => None,
            _ => Some(crypto::password_hash(
                password
                    .as_deref()
                    .map(String::as_str)
                    .ok_or_else(|| Error::bad("New password required"))?,
            )?),
        };
        self.store.write(|tx| {
            let hash = digest(&token);
            let proof = match tx.get::<Proof>("account_proofs", &hash)? {
                Some(proof) if proof.purpose != purpose => return Err(proof_error(None)),
                Some(proof) if proof.expires_at <= now() => {
                    return Err(proof_error(Some(ProofEnd::Expired)));
                }
                Some(proof) => proof,
                None => {
                    let reason = tx
                        .get::<ProofOutcome>("account_proof_outcomes", &hash)?
                        .filter(|outcome| {
                            outcome.purpose == purpose && outcome.retain_until > now()
                        })
                        .map(|outcome| outcome.reason);
                    return Err(proof_error(reason));
                }
            };
            let mut user = tx
                .get::<User>("users", &proof.user_id)?
                .filter(|u| u.epoch == proof.epoch && u.email.as_deref() == Some(&proof.email))
                .ok_or_else(|| {
                    Error::new(
                        StatusCode::CONFLICT,
                        "account_changed",
                        "Account changed; request a new link",
                    )
                })?;
            let apply_password = |user: &mut User| -> Result<()> {
                let hashed = password_hash
                    .clone()
                    .ok_or_else(|| Error::bad("New password required"))?;
                let plaintext = password
                    .as_deref()
                    .ok_or_else(|| Error::bad("New password required"))?;
                crate::identity::password_history::accept(
                    tx,
                    self.config.password_history,
                    &user.id,
                    &user.password_hash,
                    plaintext.as_str(),
                    &hashed,
                )?;
                user.password_hash = hashed;
                Ok(())
            };
            let local = crate::password::Kind::of(tx, &user)? == crate::password::Kind::Local;
            match purpose {
                Purpose::Verify if user.enabled => user.email_verified = true,
                // Recovery replaces only the password: enrolled factors stay and are still
                // required at the next sign-in, which this proof never performs.
                Purpose::Reset if user.enabled && user.email_verified && local => {
                    apply_password(&mut user)?;
                    user.epoch += 1;
                }
                Purpose::Invite
                    if !user.enabled && !user.admin && user.password_hash.is_empty() =>
                {
                    let actor =
                        creator(tx, proof.creator.as_deref().ok_or_else(Error::forbidden)?)?;
                    crate::management::accept_invitation(tx, &actor, &mut user, &proof.groups)?;
                    apply_password(&mut user)?;
                }
                _ => return Err(Error::forbidden()),
            }
            retire_proof(tx, &hash, ProofEnd::Used)?;
            tx.delete("account_latest", &proof_key(&user, purpose))?;
            tx.put("users", &user.id, &user)?;
            if purpose == Purpose::Invite {
                tx.delete("invitation_reservations", &user.id)?;
            }
            if purpose != Purpose::Verify {
                tx.delete("attempts", &user.username)?;
                crate::logout::queue_user(tx, &user.id)?;
            }
            audit(
                tx,
                &user.id,
                &format!("user.account.{}", purpose.name()),
                &user.id,
            )?;
            Ok(json!({"completed":true,"login_required":purpose!=Purpose::Verify}))
        })
    }
    pub fn mail_deliveries(&self, token: &str) -> Result<Value> {
        self.store.read(|tx|{
            self.management(tx,token,"operations.read","operations/mail")?;
            Ok(json!(tx.list::<Delivery>("mail_deliveries")?.into_iter().map(|(_,d)|json!({"id":d.id,"created_at":d.created_at,"expires_at":d.expires_at,"attempts":d.attempts,"next_attempt":d.next_attempt,"delivered_at":d.delivered_at,"stopped":d.stopped})).collect::<Vec<_>>()))
        })
    }
    fn claim_mail(&self) -> Result<Vec<Delivery>> {
        self.store.write(|tx| {
            let mut ready = Vec::new();
            for (id, mut delivery) in tx.due::<Delivery>("mail_deliveries", now(), 32)? {
                if delivery.delivered_at.is_some() || delivery.stopped {
                    continue;
                }
                if delivery.expires_at <= now()
                    || delivery.attempts >= 12
                    || tx
                        .get::<Proof>("account_proofs", &delivery.proof)?
                        .is_none()
                {
                    delivery.stopped = true;
                    delivery.body = None;
                    delivery.lease = None;
                    tx.put("mail_deliveries", &id, &delivery)?;
                    continue;
                }
                if delivery.next_attempt > now() || ready.len() >= 16 {
                    continue;
                }
                delivery.attempts += 1;
                delivery.next_attempt = now() + 60;
                delivery.lease = Some(crypto::id());
                tx.put("mail_deliveries", &id, &delivery)?;
                ready.push(delivery);
            }
            Ok(ready)
        })
    }
    fn finish_mail(&self, delivery: &Delivery, sent: bool) -> Result<()> {
        self.store.write(|tx| {
            let Some(mut current) = tx
                .get::<Delivery>("mail_deliveries", &delivery.id)?
                .filter(|d| d.lease == delivery.lease)
            else {
                return Ok(());
            };
            current.lease = None;
            if sent {
                current.delivered_at = Some(now());
                current.body = None;
            } else {
                current.next_attempt = now() + 2u64.pow(current.attempts.min(12)).min(3600);
            }
            tx.put("mail_deliveries", &current.id, &current)
        })
    }
}
pub async fn deliver(core: Core) -> Result<()> {
    let Some(config) = &core.config.mail else {
        return Ok(());
    };
    let transport = config.transport()?;
    let worker = core.clone();
    let pending = tokio::task::spawn_blocking(move || {
        crate::telemetry::in_activity(crate::telemetry::Activity::Mail, || worker.claim_mail())
    })
    .await
    .map_err(Error::internal)??;
    // Build the complete bounded batch before spawning any child work, so a
    // malformed message cannot detach sends/finishes from this pass.
    let messages = pending
        .into_iter()
        .map(|delivery| {
            let message = Message::builder()
                .from(config.from.parse::<Mailbox>().map_err(Error::internal)?)
                .to(delivery
                    .recipient
                    .parse::<Mailbox>()
                    .map_err(Error::internal)?)
                .subject(&delivery.subject)
                .header(lettre::message::header::ContentType::TEXT_PLAIN)
                .body(
                    delivery
                        .body
                        .clone()
                        .ok_or_else(|| Error::internal("Missing delivery body"))?,
                )
                .map_err(Error::internal)?;
            Ok((delivery, message))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut jobs = tokio::task::JoinSet::new();
    for (delivery, message) in messages {
        let transport = transport.clone();
        let core = core.clone();
        jobs.spawn(async move {
            let sent = tokio::time::timeout(Duration::from_secs(30), transport.send(message))
                .await
                .is_ok_and(|r| r.is_ok());
            tokio::task::spawn_blocking(move || {
                crate::telemetry::in_activity(crate::telemetry::Activity::Mail, || {
                    core.finish_mail(&delivery, sent)
                })
            })
            .await
            .map_err(Error::internal)?
        });
    }
    let mut outcome = Ok(());
    while let Some(result) = jobs.join_next().await {
        let result = result.map_err(Error::internal).and_then(|result| result);
        if outcome.is_ok() {
            outcome = result;
        }
    }
    outcome
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (key, proof) in tx.maintenance_page::<Proof>("account_proofs")? {
        if proof.expires_at <= at {
            retire_proof(tx, &key, ProofEnd::Expired)?;
        }
    }
    for (key, outcome) in tx.maintenance_page::<ProofOutcome>("account_proof_outcomes")? {
        if outcome.retain_until <= at {
            tx.delete("account_proof_outcomes", &key)?;
        }
    }
    for (key, hash) in tx.maintenance_page::<String>("account_latest")? {
        if tx.get::<Proof>("account_proofs", &hash)?.is_none() {
            tx.delete("account_latest", &key)?;
        }
    }
    for (key, (start, _, _)) in tx.maintenance_page::<(u64, u32, u64)>("mail_limits")? {
        if start + 3600 <= at {
            tx.delete("mail_limits", &key)?;
        }
    }
    for (key, mut delivery) in tx.maintenance_page::<Delivery>("mail_deliveries")? {
        if delivery.created_at + 8 * 86400 <= at {
            tx.delete("mail_deliveries", &key)?;
        } else if delivery.body.is_some()
            && (delivery.expires_at <= at
                || tx
                    .get::<Proof>("account_proofs", &delivery.proof)?
                    .is_none())
        {
            delivery.body = None;
            delivery.stopped = true;
            delivery.lease = None;
            tx.put("mail_deliveries", &key, &delivery)?;
        }
    }
    Ok(())
}
