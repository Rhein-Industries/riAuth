//! A prepared OIDC request belongs to one workflow before any verifier runs.
//! Only W03's completion transaction can turn that reservation into a code.

use super::*;
use crate::{
    model::{AuthenticationTransaction, Client, Identity},
    oidc::Authorization,
};

const AUTHORIZATIONS: &str = "workflow_authorizations";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Pin {
    request_hash: String,
    authentication: String,
    client: String,
    client_fingerprint: String,
    expires_at: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bound {
    pin: Pin,
    request: Authorization,
    run: String,
    version: RunBinding,
    workflow_request: String,
    account: String,
    account_epoch: u64,
    session: String,
    completed: bool,
    expires_at: u64,
}

/// An unforgeable in-process handoff. No deserializer, public constructor,
/// caller-supplied success flag or reusable authentication capability.
pub(crate) struct Accepted {
    session: Session,
    request: Authorization,
    proof_key: String,
}

impl Accepted {
    pub(crate) fn into_parts(self) -> (Session, Authorization, String) {
        (self.session, self.request, self.proof_key)
    }

    pub(super) fn for_consent(session: Session, request: Authorization, proof_key: String) -> Self {
        Self {
            session,
            request,
            proof_key,
        }
    }
}

fn fingerprint(client: &Client) -> Result<String> {
    serde_json::to_string(client)
        .map(|value| digest(&value))
        .map_err(Error::internal)
}

/// Ordinary authorization cannot consume or bypass a workflow reservation,
/// even with a fresh bearer session. An omitted transaction ID is rejected by
/// the live-preparation index before ordinary completion.
pub(crate) fn reject_reserved(tx: &Tx<'_>, request: &Authorization) -> Result<()> {
    super::consent::reject_reserved(tx, request)?;
    let Some(transaction) = request.transaction_id.as_deref() else {
        return Ok(());
    };
    let key = digest(transaction);
    let legacy = tx.get::<Bound>(AUTHORIZATIONS, &request.request_hash()?)?;
    if tx.get::<Bound>(AUTHORIZATIONS, &key)?.is_some()
        || legacy.is_some_and(|bound| bound.pin.authentication == key)
    {
        return Err(Error::conflict("This authorization belongs to a workflow"));
    }
    Ok(())
}

fn bound_key(tx: &Tx<'_>, pin: &Pin) -> Result<String> {
    if tx.get::<Bound>(AUTHORIZATIONS, &pin.authentication)?.is_some() {
        return Ok(pin.authentication.clone());
    }
    if tx
        .get::<Bound>(AUTHORIZATIONS, &pin.request_hash)?
        .is_some_and(|bound| bound.pin.authentication == pin.authentication)
    {
        return Ok(pin.request_hash.clone());
    }
    Err(Error::forbidden())
}

impl Core {
    /// Explicit approval pins a prepared terminal OIDC request to the canonical
    /// password/MFA chain. Verification and code issuance remain separate steps.
    pub fn workflow_authorization_start(
        &self,
        token: &str,
        request: Authorization,
    ) -> Result<View> {
        self.start_authorization_workflow(
            token,
            &local_definition(PASSWORD_WORKFLOW)?,
            Some(request),
        )
    }

    /// The same request reservation with a real, user-verified passkey ceremony.
    pub fn workflow_passkey_authorization_start(
        &self,
        token: &str,
        request: Authorization,
    ) -> Result<View> {
        self.start_authorization_workflow(
            token,
            &local_definition(PASSKEY_WORKFLOW)?,
            Some(request),
        )
    }
}

pub(super) fn bind(
    tx: &Tx<'_>,
    run: &StoredRun,
    authority: &mut RequestAuthority,
    request: &Authorization,
    at: u64,
) -> Result<()> {
    if request.decision.as_deref() != Some("approve")
        || request.has_prompt("none")
        || request.has_prompt("select_account")
    {
        return Err(Error::bad("Explicit approval for this account is required"));
    }
    // Before reservation, the ordinary issuer must also require this prepared
    // transaction. Otherwise a fresh session could have issued a code without
    // spending it, leaving an apparently pending request available to bind.
    if !request.has_prompt("login") && request.max_age != Some(0) {
        return Err(Error::bad(
            "Workflow authorization requires prompt=login or max_age=0",
        ));
    }
    let (client, _) = crate::oidc::validate_authorization(tx, request)?;
    // A UV passkey or trusted upstream assurance can satisfy the RP's MFA
    // policy. The issuer checks the actual receipts at completion. The enrolled
    // local-factor requirement remains independently pinned in the workflow.
    if client.settings.source_stage.is_some()
        || (client.require_mfa
            && run.binding.workflow.as_str() == PASSWORD_WORKFLOW
            && !authority.requires_mfa)
    {
        return Err(Error::conflict(
            "This client needs a different verifier path",
        ));
    }
    let authentication = digest(
        request
            .transaction_id
            .as_deref()
            .ok_or_else(|| Error::bad("Prepare the authorization first"))?,
    );
    let pending: AuthenticationTransaction = tx
        .get("authentication", &authentication)?
        .ok_or_else(Error::forbidden)?;
    let request_hash = request.request_hash()?;
    if pending.request_hash != request_hash
        || pending.user_id.as_deref() != Some(&run.account)
        || pending.authenticated_session.is_some()
        || pending.source_stage.is_some()
        || pending.expires_at <= at
    {
        return Err(Error::forbidden());
    }
    reject_reserved(tx, request)?;
    let expires_at = authority
        .expires_at
        .min(pending.expires_at)
        .min(crate::authorization::reference_expiry(tx, request)?.unwrap_or(u64::MAX));
    if expires_at <= at {
        return Err(Error::forbidden());
    }
    let pin = Pin {
        request_hash,
        authentication,
        client: client.id.clone(),
        client_fingerprint: fingerprint(&client)?,
        expires_at,
    };
    let mut request = request.clone();
    // Persist only the existing transaction's digest, never its bearer handle.
    request.transaction_id = None;
    let bound = Bound {
        pin: pin.clone(),
        request,
        run: run.id.clone(),
        version: run.binding.clone(),
        workflow_request: run.request.clone(),
        account: run.account.clone(),
        account_epoch: run.account_epoch,
        session: authority.session.clone(),
        completed: false,
        expires_at,
    };
    tx.put(AUTHORIZATIONS, &pin.authentication, &bound)?;
    authority.expires_at = expires_at;
    authority.authorization = Some(pin);
    Ok(())
}

fn pending(tx: &Tx<'_>, run: &StoredRun, authority: &RequestAuthority, at: u64) -> Result<Bound> {
    let pin = authority
        .authorization
        .as_ref()
        .ok_or_else(Error::forbidden)?;
    let bound: Bound = tx
        .get(AUTHORIZATIONS, &bound_key(tx, pin)?)?
        .ok_or_else(Error::forbidden)?;
    if bound.pin != *pin
        || bound.completed
        || pin.expires_at <= at
        || bound.expires_at != pin.expires_at
        || bound.run != run.id
        || bound.version != run.binding
        || bound.workflow_request != run.request
        || bound.account != run.account
        || bound.account_epoch != run.account_epoch
        || run.session.as_deref() != Some(bound.session.as_str())
        || bound.session != authority.session
        || bound.request.request_hash()? != pin.request_hash
        || bound.request.client_id != pin.client
        || bound.request.decision.as_deref() != Some("approve")
        || bound.request.transaction_id.is_some()
    {
        return Err(Error::forbidden());
    }
    let (client, _) = crate::oidc::validate_authorization(tx, &bound.request)?;
    if fingerprint(&client)? != pin.client_fingerprint {
        return Err(Error::conflict("Authorization client changed"));
    }
    let proof: AuthenticationTransaction = tx
        .get("authentication", &pin.authentication)?
        .ok_or_else(Error::forbidden)?;
    if proof.request_hash != pin.request_hash
        || proof.user_id.as_deref() != Some(&run.account)
        || proof.authenticated_session.is_some()
        || proof.source_stage.is_some()
        || proof.expires_at < pin.expires_at
        || proof.expires_at <= at
    {
        return Err(Error::forbidden());
    }
    Ok(bound)
}

pub(super) fn check(
    tx: &Tx<'_>,
    run: &StoredRun,
    authority: &RequestAuthority,
    at: u64,
) -> Result<()> {
    if authority.authorization.is_some() {
        pending(tx, run, authority, at)?;
    }
    Ok(())
}

fn grant_identity(
    tx: &Tx<'_>,
    run: &StoredRun,
    authority: &RequestAuthority,
    session: &Session,
    evidence: &[StoredEvidence],
) -> Result<Identity> {
    // Accept only the canonical chain's exact proof shape. In particular,
    // upstream MFA never substitutes for an enrolled local factor.
    let (primary_kind, local_factor) = match run.binding.workflow.as_str() {
        PASSWORD_WORKFLOW => (Proof::Password, false),
        password::TOTP_WORKFLOW => (Proof::Password, true),
        PASSKEY_WORKFLOW => (Proof::Passkey, false),
        source::SOURCE_WORKFLOW => (Proof::Source, false),
        source::TOTP_WORKFLOW => (Proof::Source, true),
        _ => return Err(Error::forbidden()),
    };
    let primary = evidence
        .iter()
        .find(|e| e.proof == primary_kind)
        .ok_or_else(Error::forbidden)?;
    let factor = evidence
        .iter()
        .find(|e| matches!(e.proof, Proof::Totp | Proof::RecoveryCode));
    if authority.requires_mfa != local_factor
        || factor.is_some() != local_factor
        || evidence.len() != 1 + usize::from(local_factor)
        || authority.source.is_some() != (primary_kind == Proof::Source)
    {
        return Err(Error::forbidden());
    }
    let (mut amr, upstream_mfa, source) = match primary_kind {
        Proof::Password => (vec!["pwd".into()], false, None),
        // The adapter only emits this proof after the existing verifier's UV
        // requirement and its normalized [webauthn, mfa] assurance agree.
        Proof::Passkey => (vec!["webauthn".into(), "mfa".into()], true, None),
        Proof::Source => {
            let receipt = primary.source.as_ref().ok_or_else(Error::forbidden)?;
            let mut amr = vec!["federated".into()];
            if receipt.mfa {
                amr.push("mfa".into());
            }
            (
                amr,
                receipt.mfa,
                Some(upstream::authorization_identity(tx, session, receipt)?),
            )
        }
        _ => return Err(Error::forbidden()),
    };
    if let Some(factor) = factor {
        amr.push(
            if factor.proof == Proof::Totp {
                "otp"
            } else {
                "recovery_code"
            }
            .into(),
        );
    }
    Ok(Identity {
        user_id: run.account.clone(),
        epoch: run.account_epoch,
        session_id: session.id.clone(),
        auth_time: primary.verified_at,
        mfa: upstream_mfa || local_factor,
        amr,
        source,
    })
}

/// Called only after the W03 completion store validates and consumes every
/// receipt, inside the same transaction as factor consumption and run completion.
pub(super) fn complete(
    core: &Core,
    tx: &Tx<'_>,
    run: &StoredRun,
    evidence: &[StoredEvidence],
    at: u64,
) -> Result<Option<String>> {
    let (_, authority) = super::authority(core, tx, run, at)?;
    if authority.authorization.is_none() {
        return Ok(None);
    }
    let mut bound = pending(tx, run, &authority, at)?;
    let mut session: Session = tx
        .get("sessions", &bound.session)?
        .ok_or_else(Error::forbidden)?;
    // Assurance belongs to this authorization grant, never to the stored session.
    session.identity = grant_identity(tx, run, &authority, &session, evidence)?;
    let mut proof: AuthenticationTransaction = tx
        .get("authentication", &bound.pin.authentication)?
        .ok_or_else(Error::forbidden)?;
    let session_expires_at = session.expires_at;
    proof.authenticated_session = Some(bound.session.clone());
    tx.put("authentication", &bound.pin.authentication, &proof)?;
    let response = core.authorize_workflow(
        tx,
        Accepted {
            session,
            request: bound.request.clone(),
            proof_key: bound.pin.authentication.clone(),
        },
    )?;
    let finished_at = now();
    if bound.expires_at <= finished_at
        || session_expires_at <= finished_at
        || evidence.iter().any(|receipt| {
            receipt.expires_at <= finished_at
                || receipt.verified_at.saturating_add(RECEIPT_SECONDS) <= finished_at
        })
    {
        return Err(Error::conflict(
            "Authorization proof expired during completion",
        ));
    }
    // Requests needing no reauthentication must also spend their transaction.
    tx.delete("authentication", &bound.pin.authentication)?;
    bound.completed = true;
    tx.put(AUTHORIZATIONS, &bound_key(tx, &bound.pin)?, &bound)?;
    Ok(Some(response))
}

pub(super) fn abandon(tx: &Tx<'_>, run: &StoredRun) -> Result<()> {
    let Some(authority) = tx.get::<RequestAuthority>(REQUESTS, &run.request)? else {
        return Ok(());
    };
    let Some(pin) = authority.authorization else {
        return Ok(());
    };
    let key = bound_key(tx, &pin)?;
    let mut bound: Bound = tx.get(AUTHORIZATIONS, &key)?.ok_or_else(Error::forbidden)?;
    if bound.pin != pin || bound.run != run.id || bound.workflow_request != run.request {
        return Err(Error::forbidden());
    }
    bound.completed = true;
    tx.put(AUTHORIZATIONS, &key, &bound)?;
    tx.delete("authentication", &pin.authentication)
}

pub(super) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (key, bound) in tx.maintenance_page::<Bound>(AUTHORIZATIONS)? {
        if bound.expires_at.saturating_add(RETAIN_FINAL_SECONDS) <= at {
            tx.delete(AUTHORIZATIONS, &key)?;
        }
    }
    Ok(())
}
