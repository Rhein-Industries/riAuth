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
}

fn fingerprint(client: &Client) -> Result<String> {
    serde_json::to_string(client)
        .map(|value| digest(&value))
        .map_err(Error::internal)
}

/// Ordinary authorization cannot consume or bypass a workflow reservation,
/// including by omitting the transaction id or using a fresh bearer session.
pub(crate) fn reject_reserved(tx: &Tx<'_>, request: &Authorization) -> Result<()> {
    if tx
        .get::<Bound>(AUTHORIZATIONS, &request.request_hash()?)?
        .is_some()
    {
        return Err(Error::conflict("This authorization belongs to a workflow"));
    }
    Ok(())
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
    if client.settings.source_stage.is_some() || client.require_mfa && !authority.requires_mfa {
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
        || tx.get::<Bound>(AUTHORIZATIONS, &request_hash)?.is_some()
    {
        return Err(Error::forbidden());
    }
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
    tx.put(AUTHORIZATIONS, &pin.request_hash, &bound)?;
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
        .get(AUTHORIZATIONS, &pin.request_hash)?
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
    let password = evidence
        .iter()
        .find(|e| e.proof == Proof::Password)
        .ok_or_else(Error::forbidden)?;
    if !matches!(
        run.binding.workflow.as_str(),
        PASSWORD_WORKFLOW | password::TOTP_WORKFLOW
    ) || evidence
        .iter()
        .any(|e| !matches!(e.proof, Proof::Password | Proof::Totp | Proof::RecoveryCode))
    {
        return Err(Error::forbidden());
    }
    let mut session: Session = tx
        .get("sessions", &bound.session)?
        .ok_or_else(Error::forbidden)?;
    let factor = evidence
        .iter()
        .find(|e| matches!(e.proof, Proof::Totp | Proof::RecoveryCode));
    let mut methods = vec!["pwd".to_owned()];
    if let Some(factor) = factor {
        methods.push(
            if factor.proof == Proof::Totp {
                "otp"
            } else {
                "recovery_code"
            }
            .into(),
        );
    }
    // Assurance belongs to this authorization grant, never to the stored session.
    session.identity = Identity {
        user_id: run.account.clone(),
        epoch: run.account_epoch,
        session_id: bound.session.clone(),
        auth_time: password.verified_at,
        mfa: factor.is_some(),
        amr: methods,
        source: None,
    };
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
    tx.put(AUTHORIZATIONS, &bound.pin.request_hash, &bound)?;
    Ok(Some(response))
}

pub(super) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (key, bound) in tx.maintenance_page::<Bound>(AUTHORIZATIONS)? {
        if bound.expires_at.saturating_add(RETAIN_FINAL_SECONDS) <= at {
            tx.delete(AUTHORIZATIONS, &key)?;
        }
    }
    Ok(())
}
