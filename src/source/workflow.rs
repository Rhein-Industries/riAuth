//! Adapter from the existing upstream verifier to W03 evidence. Only the
//! executor can reserve and consume these logins; no assertion is an input.

pub(crate) use super::WorkflowBinding as Binding;
use super::*;
use crate::workflow::{
    Id,
    evidence::{SourceEvidence, SourceSession},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pin {
    pub source: Id,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Attempt {
    pub login: String,
    pub nonce: String,
}

pub(crate) enum Verification {
    Pending,
    Failed,
    Verified {
        authority: SourceEvidence,
        auth_time: u64,
        expires_at: u64,
    },
}

pub(crate) fn pin(tx: &Tx<'_>, id: &Id) -> Result<Pin> {
    if !cfg!(feature = "platform") {
        return Err(Error::forbidden());
    }
    let source = enabled(tx, id.as_str())?;
    // OIDC and SAML verifiers both preserve signed authentication time and
    // assertion expiry. OAuth profile responses cannot supply either fact.
    if source.id != id.as_str() || source.oauth_profile.is_some() {
        return Err(Error::conflict("Workflow requires an OIDC or SAML source"));
    }
    Ok(Pin {
        source: id.clone(),
        fingerprint: source.fingerprint()?,
    })
}

pub(crate) fn authority(tx: &Tx<'_>, expected: &Pin, user: &User) -> Result<()> {
    if pin(tx, &expected.source)? != *expected || !user.enabled {
        return Err(Error::forbidden());
    }
    let source = enabled(tx, expected.source.as_str())?;
    if user.admin && !source.allow_admin_login {
        return Err(Error::forbidden());
    }
    Ok(())
}

pub(crate) fn evidence_authority(
    tx: &Tx<'_>,
    pin: &Pin,
    user: &User,
    evidence: &SourceEvidence,
) -> Result<()> {
    authority(tx, pin, user)?;
    let source = enabled(tx, pin.source.as_str())?;
    let link = tx
        .get::<Link>("source_links", &evidence.link)?
        .ok_or_else(Error::forbidden)?;
    if evidence.source != pin.source
        || evidence.fingerprint != pin.fingerprint
        || evidence.transaction.is_empty()
        || evidence.link != link_key(&source.id, &source.issuer, &evidence.subject)
        || link.source != source.id
        || link.issuer != source.issuer
        || link.subject != evidence.subject
        || link.user_id != user.id
    {
        return Err(Error::forbidden());
    }
    Ok(())
}

impl Core {
    pub(crate) fn begin_workflow_source(
        &self,
        tx: &Tx<'_>,
        pin: &Pin,
        binding: Binding,
        expires_at: u64,
    ) -> Result<(Attempt, String)> {
        let started = self.source_start_in(
            tx,
            pin.source.as_str(),
            &Start {
                link: false,
                authentication_transaction: None,
            },
            None,
            None,
        )?;
        let key = digest(&started.state);
        let mut login = tx
            .get::<Login>("source_logins", &key)?
            .ok_or_else(Error::forbidden)?;
        if login.fingerprint != pin.fingerprint || login.started_at < binding.started_at {
            return Err(Error::forbidden());
        }
        login.expires_at = login.expires_at.min(expires_at);
        login.workflow = Some(binding);
        tx.put("source_logins", &key, &login)?;
        // The ordinary CLI completion credential is neither returned nor usable.
        tx.delete("source_polls", &login.poll_hash)?;
        Ok((
            Attempt {
                login: key,
                nonce: login.nonce,
            },
            started.authorization_url,
        ))
    }
}

pub(crate) fn consume(
    tx: &Tx<'_>,
    pin: &Pin,
    attempt: &Attempt,
    binding: &Binding,
    user: &User,
    at: u64,
) -> Result<Verification> {
    authority(tx, pin, user)?;
    let login = tx
        .get::<Login>("source_logins", &attempt.login)?
        .ok_or_else(Error::forbidden)?;
    if login.workflow.as_ref() != Some(binding)
        || login.source != pin.source.as_str()
        || login.fingerprint != pin.fingerprint
        || login.nonce != attempt.nonce
        || login.started_at < binding.started_at
        || login.started_at > at
        || login.expires_at <= at
        || login.target.is_some()
        || login.authentication.is_some()
        || login.stage.is_some()
        || user.id != binding.account
        || user.epoch != binding.account_epoch
    {
        return Err(Error::forbidden());
    }
    if login.failed {
        discard(tx, attempt, binding)?;
        return Ok(Verification::Failed);
    }
    let Some(identity) = &login.result else {
        return Ok(Verification::Pending);
    };
    let source = enabled(tx, pin.source.as_str())?;
    // The source fingerprint pins the protocol as well as its trust settings.
    // A SAML receipt also cannot outlive the signed upstream session bound.
    if source.saml.is_some() != identity.saml_session.is_some() {
        return Err(Error::forbidden());
    }
    let session_expiry = identity
        .saml_session
        .as_ref()
        .and_then(|session| session.expires_at);
    let expires_at = identity
        .expires_at
        .ok_or_else(Error::forbidden)?
        .min(login.expires_at)
        .min(session_expiry.unwrap_or(u64::MAX));
    if !login.claimed
        || identity.auth_time < login.started_at
        || identity.auth_time > at
        || expires_at <= at
    {
        return Err(Error::conflict("Upstream workflow verification is stale"));
    }
    let evidence = SourceEvidence {
        source: pin.source.clone(),
        fingerprint: pin.fingerprint.clone(),
        link: link_key(&source.id, &source.issuer, &identity.subject),
        subject: identity.subject.clone(),
        transaction: attempt.login.clone(),
        mfa: identity.mfa,
        saml_session: identity.saml_session.as_ref().map(|session| SourceSession {
            subject: session.subject.clone(),
            index: session.index.clone(),
        }),
    };
    // Existing explicit links only: a workflow cannot create or reattach an account.
    evidence_authority(tx, pin, user, &evidence)?;
    discard(tx, attempt, binding)?;
    Ok(Verification::Verified {
        authority: evidence,
        auth_time: identity.auth_time,
        expires_at,
    })
}

/// Retain source provenance on the grant without upgrading the bearer session.
/// A SAML grant must retain the existing session's exact logout association;
/// a fresh assertion cannot overwrite it or borrow another source's session.
pub(crate) fn authorization_identity(
    tx: &Tx<'_>,
    session: &Session,
    evidence: &SourceEvidence,
) -> Result<SourceIdentity> {
    let source = enabled(tx, evidence.source.as_str())?;
    if source.saml.is_some() {
        let verified = evidence
            .saml_session
            .as_ref()
            .ok_or_else(Error::forbidden)?;
        let existing = session
            .identity
            .source
            .as_ref()
            .ok_or_else(Error::forbidden)?;
        let upstream: saml::UpstreamSession = tx
            .get("saml_source_sessions", &session.id)?
            .ok_or_else(Error::forbidden)?;
        if existing.id != evidence.source.as_str()
            || existing.fingerprint != evidence.fingerprint
            || existing.link != evidence.link
            || verified.subject.is_none()
            || upstream.subject != verified.subject
            || upstream.index != verified.index
            || upstream.expires_at.is_some_and(|expiry| expiry <= now())
        {
            return Err(Error::forbidden());
        }
    } else if evidence.saml_session.is_some() {
        return Err(Error::forbidden());
    }
    Ok(SourceIdentity {
        id: evidence.source.as_str().to_owned(),
        fingerprint: evidence.fingerprint.clone(),
        link: evidence.link.clone(),
    })
}

pub(crate) fn discard(tx: &Tx<'_>, attempt: &Attempt, binding: &Binding) -> Result<()> {
    if let Some(login) = tx.get::<Login>("source_logins", &attempt.login)? {
        if login.workflow.as_ref() != Some(binding) || login.nonce != attempt.nonce {
            return Err(Error::forbidden());
        }
        tx.delete("source_polls", &login.poll_hash)?;
        tx.delete("source_logins", &attempt.login)?;
    }
    Ok(())
}
