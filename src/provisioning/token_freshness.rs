//! Non-secret, shared OAuth cache stamps. Tokens stay in process memory.
//! Token acquisition runs outside the writer; publication and invalidation
//! compare the observed owner/generation in one short transaction.
use super::{Core, Error, OauthTokenRecord, Result, crypto};
use serde::{Deserialize, Serialize};

const BUCKET: &str = "scim_oauth_freshness";

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Stamp {
    generation: u64,
    owner: String,
}

impl Stamp {
    fn successor(&self) -> Result<Self> {
        Ok(Self {
            generation: self
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::conflict("SCIM OAuth freshness generation exhausted"))?,
            owner: crypto::id(),
        })
    }
}

pub(super) fn current(core: &Core, name: &str) -> Result<Stamp> {
    Ok(core.store.get(BUCKET, name)?.unwrap_or_default())
}

pub(super) fn publish(
    core: &Core,
    name: &str,
    expected: &Stamp,
    expires_at: u64,
    fingerprint: &str,
) -> Result<Stamp> {
    core.store.write(|tx| {
        let current = tx.get::<Stamp>(BUCKET, name)?.unwrap_or_default();
        if current != *expected {
            return Err(Error::conflict(
                "SCIM OAuth freshness changed during token acquisition; retry",
            ));
        }
        let next = current.successor()?;
        tx.put(BUCKET, name, &next)?;
        tx.put(
            "scim_oauth_cache",
            name,
            &OauthTokenRecord {
                expires_at,
                fingerprint: fingerprint.into(),
            },
        )?;
        Ok(next)
    })
}

pub(super) fn invalidate(core: &Core, name: &str, expected: &Stamp) -> Result<()> {
    core.store.write(|tx| {
        let current = tx.get::<Stamp>(BUCKET, name)?.unwrap_or_default();
        if current == *expected {
            tx.put(BUCKET, name, &current.successor()?)?;
            tx.delete("scim_oauth_cache", name)?;
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_invalidation_does_not_clear_a_newer_issuance() {
        use super::super::{
            Issued, cached_bearer, discard_cached_bearer, invalidate_generation, store_bearer,
        };
        use zeroize::Zeroizing;
        let dir = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            crate::config::Config {
                data_dir: dir.path().into(),
                ..Default::default()
            },
            crate::model::NewUser {
                username: "admin".into(),
                password: "freshness-fixture-password".into(),
                email: None,
                display_name: "Admin".into(),
                admin: true,
            },
        )
        .unwrap();
        let name = crypto::id();
        let key = format!("{name}\0config");
        let issued = Issued {
            token: Zeroizing::new("synthetic-access".into()),
            expires_at: crypto::now() + 600,
            fingerprint: "first-digest".into(),
        };
        let initial = current(&core, &name).unwrap();
        let first = publish(&core, &name, &initial, 100, "first-digest").unwrap();
        let generation = store_bearer(&name, &key, "synthetic-secret-digest", &issued, &first);
        discard_cached_bearer(&name);
        let second = publish(&core, &name, &first, 200, "second-digest").unwrap();
        let replacement = store_bearer(&name, &key, "synthetic-secret-digest", &issued, &second);
        assert_eq!(
            replacement, generation,
            "the local counter can restart after eviction"
        );
        invalidate_generation(&core, &name, &key, generation, &first).unwrap();
        assert!(cached_bearer(&key, "synthetic-secret-digest", &second).is_some());
        invalidate(&core, &name, &first).unwrap();
        assert!(current(&core, &name).unwrap() == second);
        assert_eq!(
            core.store
                .get::<OauthTokenRecord>("scim_oauth_cache", &name)
                .unwrap()
                .unwrap()
                .fingerprint,
            "second-digest"
        );
        assert!(publish(&core, &name, &first, 300, "late-digest").is_err());
        invalidate(&core, &name, &second).unwrap();
        assert!(current(&core, &name).unwrap() != second);
        assert!(
            core.store
                .get::<OauthTokenRecord>("scim_oauth_cache", &name)
                .unwrap()
                .is_none()
        );
        discard_cached_bearer(&name);
    }
}
