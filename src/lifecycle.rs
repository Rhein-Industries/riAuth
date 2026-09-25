//! Purpose-bound account proofs and a leased, durable SMTP outbox.
use crate::{
    agent::{Agent, Principal},
    core::{Core, audit, make_user, user_by_name, validate_display, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Group, NewUser, User, UserView},
    store::Tx,
};
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
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invitation {
    pub username: String,
    pub email: String,
    pub display_name: String,
    #[serde(default)]
    pub groups: BTreeSet<String>,
}
fn require_mail(core: &Core) -> Result<()> {
    if core.config.mail.is_none() {
        return Err(Error::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "delivery_unavailable",
            "Account email delivery is not configured",
        ));
    }
    Ok(())
}
fn email(value: &str) -> Result<()> {
    crate::core::validate_email(value)?;
    value
        .parse::<lettre::Address>()
        .map_err(|_| Error::bad("Invalid email address"))?;
    Ok(())
}
fn proof_key(user: &User, purpose: Purpose) -> String {
    format!("{}:{}", user.id, purpose.name())
}
fn enqueue(
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
        tx.delete("account_proofs", &old)?;
    }
    let token = zeroize::Zeroizing::new(crypto::random_token("ri_mail_"));
    let hash = digest(&token);
    let expires_at = now()
        + match purpose {
            Purpose::Reset => 1800,
            Purpose::Verify => 86400,
            Purpose::Invite => 7 * 86400,
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
    let body = format!(
        "{subject}\n\nServer: {}\nAccount: {}\n\nRun: riauth --server {} account {} --token-stdin\nPaste this one-use code when asked:\n{}\n\nExpires at Unix time {expires_at}. A password reset keeps your enrolled MFA factors. If you did not request this message, ignore it.\n",
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
            if now().saturating_sub(session.identity.auth_time) > 300 {
                return Err(Error::forbidden());
            }
            if request_allowed(tx, &user.username, Purpose::Verify)? && !user.email_verified {
                enqueue(self, tx, &user, Purpose::Verify, BTreeSet::new(), None)?;
                audit(tx, &user.id, "account.verification.request", &user.id)?;
            }
            Ok(json!({"accepted":true}))
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
                if let Some(user) =
                    user.filter(|u| u.enabled && u.email_verified && !u.password_hash.is_empty())
                {
                    enqueue(self, tx, &user, Purpose::Reset, BTreeSet::new(), None)?;
                }
            }
            Ok(json!({"accepted":true}))
        })
    }
    pub fn account_invite(&self, token: &str, input: Invitation) -> Result<Value> {
        require_mail(self)?;
        validate_name(&input.username)?;
        validate_display(&input.display_name)?;
        email(&input.email)?;
        self.mutation(token, |tx| {
            let actor =
                self.management(tx, token, "user.write", &format!("user/{}", input.username))?;
            for group in &input.groups {
                actor.require("group.members", &format!("group/{group}"))?;
                tx.get::<Group>("groups", group)?
                    .ok_or_else(|| Error::missing("Invitation group not found"))?;
            }
            if tx.get::<String>("usernames", &input.username)?.is_some() {
                return Err(Error::conflict("User already exists"));
            }
            let mut user = make_user(NewUser {
                username: input.username,
                email: Some(input.email),
                display_name: input.display_name,
                password: crypto::random_token(""),
                admin: false,
            })?;
            user.password_hash.clear();
            user.enabled = false;
            tx.put("users", &user.id, &user)?;
            tx.put("usernames", &user.username, &user.id)?;
            enqueue(
                self,
                tx,
                &user,
                Purpose::Invite,
                input.groups,
                Some(actor.id.clone()),
            )?;
            audit(tx, &actor.id, "user.invite", &user.id)?;
            Ok(json!({"user":UserView::from(&user),"delivery_queued":true}))
        })
    }
    pub fn account_invitation_revoke(&self, token: &str, username: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "user.write", &format!("user/{username}"))?;
            let user = user_by_name(tx, username)?;
            if let Some(hash) =
                tx.get::<String>("account_latest", &proof_key(&user, Purpose::Invite))?
            {
                tx.delete("account_proofs", &hash)?;
            }
            audit(tx, &actor.id, "user.invitation.revoke", &user.id)?;
            Ok(json!({"revoked":true}))
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
            return Err(Error::bad("Invalid account code"));
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
            let proof = tx
                .get::<Proof>("account_proofs", &hash)?
                .filter(|p| p.expires_at > now() && p.purpose == purpose)
                .ok_or_else(|| Error::bad("Account code expired, consumed or invalid"))?;
            let mut user = tx
                .get::<User>("users", &proof.user_id)?
                .filter(|u| u.epoch == proof.epoch && u.email.as_deref() == Some(&proof.email))
                .ok_or_else(|| Error::bad("Account changed; request a new code"))?;
            let apply_password = |user: &mut User| -> Result<()> {
                let hashed = password_hash
                    .clone()
                    .ok_or_else(|| Error::bad("New password required"))?;
                let plaintext = password
                    .as_deref()
                    .ok_or_else(|| Error::bad("New password required"))?;
                crate::password_history::accept(
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
            match purpose {
                Purpose::Verify if user.enabled => user.email_verified = true,
                Purpose::Reset
                    if user.enabled && user.email_verified && !user.password_hash.is_empty() =>
                {
                    apply_password(&mut user)?;
                    user.epoch += 1;
                }
                Purpose::Invite
                    if !user.enabled && !user.admin && user.password_hash.is_empty() =>
                {
                    let actor =
                        creator(tx, proof.creator.as_deref().ok_or_else(Error::forbidden)?)?;
                    actor.require("user.write", &format!("user/{}", user.username))?;
                    let mut groups = Vec::new();
                    for name in &proof.groups {
                        actor.require("group.members", &format!("group/{name}"))?;
                        let mut group = tx
                            .get::<Group>("groups", name)?
                            .ok_or_else(|| Error::missing("Invitation group was removed"))?;
                        group.members.insert(user.id.clone());
                        groups.push(group);
                    }
                    for group in groups {
                        tx.put("groups", &group.name, &group)?;
                    }
                    apply_password(&mut user)?;
                    user.enabled = true;
                    user.email_verified = true;
                    user.epoch += 1;
                }
                _ => return Err(Error::forbidden()),
            }
            tx.delete("account_proofs", &hash)?;
            tx.delete("account_latest", &proof_key(&user, purpose))?;
            tx.put("users", &user.id, &user)?;
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
    let pending = tokio::task::spawn_blocking(move || worker.claim_mail())
        .await
        .map_err(Error::internal)??;
    let mut jobs = tokio::task::JoinSet::new();
    for delivery in pending {
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
        let transport = transport.clone();
        let core = core.clone();
        jobs.spawn(async move {
            let sent = tokio::time::timeout(Duration::from_secs(30), transport.send(message))
                .await
                .is_ok_and(|r| r.is_ok());
            tokio::task::spawn_blocking(move || core.finish_mail(&delivery, sent))
                .await
                .map_err(Error::internal)?
        });
    }
    while let Some(result) = jobs.join_next().await {
        result.map_err(Error::internal)??;
    }
    Ok(())
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (key, proof) in tx.maintenance_page::<Proof>("account_proofs")? {
        if proof.expires_at <= at {
            tx.delete("account_proofs", &key)?;
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
