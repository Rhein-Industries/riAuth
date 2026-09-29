//! Shared source-link validation and persistence for plan apply and verified login.

use crate::{
    agent::Principal,
    error::{Error, Result},
    model::User,
    source::{Link, LinkSpec, enabled, link_key},
    store::Tx,
};

pub(crate) enum SourceLinkAuthority<'a> {
    /// The enclosing immutable plan supplies its receipt and emits
    /// `source_link.reconcile` plus `state.apply` after full reconciliation.
    Plan {
        actor: &'a Principal,
        spec: &'a LinkSpec,
    },
    /// The source adapter verifies the upstream response, target binding and
    /// consent before this call. The completed login emits `source.link` or
    /// `source.login` only after its new session has been persisted.
    VerifiedLogin {
        source_id: &'a str,
        source_fingerprint: &'a str,
        user_id: &'a str,
        subject: &'a str,
        approved: bool,
    },
}

pub(crate) struct SourceLinkWrite {
    pub(crate) id: String,
    pub(crate) created: bool,
    pub(crate) previous_issuer: Option<String>,
}

/// Re-read the live source and account in the caller's transaction. No link
/// can be reassigned to a different account; doing so would leave sessions
/// issued under its old owner attached to the same link identifier.
pub(crate) fn write_source_link(
    tx: &Tx<'_>,
    authority: SourceLinkAuthority<'_>,
) -> Result<SourceLinkWrite> {
    let (source_id, subject) = match &authority {
        SourceLinkAuthority::Plan { spec, .. } => (spec.source.as_str(), spec.subject.as_str()),
        SourceLinkAuthority::VerifiedLogin {
            source_id, subject, ..
        } => (*source_id, *subject),
    };
    let source = enabled(tx, source_id)?;
    let (user, login) = match &authority {
        SourceLinkAuthority::Plan { actor, spec } => {
            actor.require("source.write", &format!("source/{}", spec.source))?;
            actor.require("user.write", &format!("user/{}", spec.username))?;
            let user = crate::core::user_by_name(tx, &spec.username)?;
            if user.admin && (actor.agent || !source.allow_admin_login) {
                return Err(Error::forbidden());
            }
            (user, false)
        }
        SourceLinkAuthority::VerifiedLogin {
            source_fingerprint,
            user_id,
            approved,
            ..
        } => {
            if !approved {
                return Err(Error::forbidden());
            }
            if *source_fingerprint != source.fingerprint()? {
                return Err(Error::bad("Source configuration changed; restart login"));
            }
            let user = tx
                .get::<User>("users", user_id)?
                .ok_or_else(Error::forbidden)?;
            if !user.enabled || user.admin && !source.allow_admin_login {
                return Err(Error::forbidden());
            }
            (user, true)
        }
    };
    if subject.is_empty() || subject.len() > 255 || subject.chars().any(char::is_control) {
        return Err(Error::bad("Invalid source subject"));
    }
    if let SourceLinkAuthority::Plan { spec, .. } = &authority
        && spec
            .issuer
            .as_ref()
            .is_some_and(|issuer| issuer != &source.issuer)
    {
        return Err(Error::conflict(
            "Target source link issuer is stale; export the target and convert again",
        ));
    }
    let id = link_key(&source.id, &source.issuer, subject);
    let existing = tx.get::<Link>("source_links", &id)?;
    if let Some(link) = &existing {
        if link.user_id != user.id {
            return Err(if login {
                Error::forbidden()
            } else {
                Error::conflict("Source identity already belongs to another local account")
            });
        }
        if !login {
            if let SourceLinkAuthority::Plan { spec, .. } = &authority
                && spec.issuer.is_some()
                && link.issuer != source.issuer
            {
                let previous_issuer = link.issuer.clone();
                tx.put(
                    "source_links",
                    &id,
                    &Link {
                        source: source.id,
                        issuer: source.issuer,
                        subject: subject.into(),
                        user_id: user.id,
                    },
                )?;
                return Ok(SourceLinkWrite {
                    id,
                    created: false,
                    previous_issuer: Some(previous_issuer),
                });
            }
            return Ok(SourceLinkWrite {
                id,
                created: false,
                previous_issuer: None,
            });
        }
    }
    // A verified login keeps the previous upsert behavior. Plan apply treats
    // an existing link to the same account as a no-op.
    tx.put(
        "source_links",
        &id,
        &Link {
            source: source.id,
            issuer: source.issuer,
            subject: subject.into(),
            user_id: user.id,
        },
    )?;
    Ok(SourceLinkWrite {
        id,
        created: existing.is_none(),
        previous_issuer: None,
    })
}
