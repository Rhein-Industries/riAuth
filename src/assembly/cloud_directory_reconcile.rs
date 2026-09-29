//! Concrete cloud reconciliation over the caller's reviewed transaction.

use crate::{
    agent::Principal,
    cloud_directory::{self, Binding, Change, CloudSnapshotDraft, Entry, Settings, binding_key},
    connector_guard::RemovalImpact,
    crypto::{self, now},
    error::{Error, Result},
    model::{Group, User},
    store::Tx,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn removal_impact(
    tx: &Tx<'_>,
    settings: &Settings,
    snapshot: &[Entry],
) -> Result<RemovalImpact> {
    let entries: BTreeMap<_, _> = snapshot
        .iter()
        .map(|entry| (entry.external_id.as_str(), entry))
        .collect();
    let mut active_linked = 0usize;
    let mut impact = RemovalImpact::default();
    for (_, binding) in tx.list::<Binding>("cloud_directory_bindings")? {
        if binding.kind != settings.kind || binding.directory != settings.id {
            continue;
        }
        let user = tx
            .get::<User>("users", &binding.user_id)?
            .ok_or_else(|| Error::conflict("Cloud directory owned user is missing"))?;
        let desired = entries.get(binding.external_id.as_str()).copied();
        if desired.is_none() {
            impact.missing_users += 1;
        }
        if user.enabled {
            active_linked += 1;
            if desired.is_none_or(|entry| entry.disabled) {
                impact.disabled_users += 1;
            }
        }
        for group in &binding.groups {
            if !settings.reconcile_groups().contains_key(group) {
                continue;
            }
            if desired.is_none_or(|entry| !entry.groups.contains(group)) {
                impact.removed_memberships += 1;
            }
        }
    }
    impact.assess(active_linked);
    Ok(impact)
}

fn linked_usernames(tx: &Tx<'_>, settings: &Settings) -> Result<BTreeMap<String, String>> {
    let mut linked = BTreeMap::new();
    for (_, binding) in tx.list::<Binding>("cloud_directory_bindings")? {
        if binding.kind == settings.kind && binding.directory == settings.id {
            if binding.identity_fingerprint != settings.reconcile_identity_fingerprint() {
                return Err(Error::conflict(
                    "Cloud directory identity mapping changed while accounts are linked; use a new directory ID",
                ));
            }
            let user = tx
                .get::<User>("users", &binding.user_id)?
                .ok_or_else(|| Error::conflict("Cloud directory owned user is missing"))?;
            linked.insert(binding.external_id, user.username);
        }
    }
    Ok(linked)
}

pub(crate) fn materialize_completed_draft(
    tx: &Tx<'_>,
    settings: &Settings,
    draft: &CloudSnapshotDraft,
) -> Result<Vec<Entry>> {
    let linked = linked_usernames(tx, settings)?;
    cloud_directory::materialize_completed_draft(settings, draft, linked)
}

#[expect(
    clippy::too_many_arguments,
    reason = "Reviewed transaction inputs remain explicit"
)]
fn membership(
    config: &crate::config::Config,
    tx: &Tx<'_>,
    actor: &Principal,
    scope: &str,
    uid: &str,
    allow: &BTreeSet<String>,
    old: &BTreeSet<String>,
    desired: &BTreeSet<String>,
) -> Result<bool> {
    let old: BTreeSet<_> = old
        .iter()
        .filter(|name| allow.contains(*name))
        .cloned()
        .collect();
    let desired: BTreeSet<_> = desired
        .iter()
        .filter(|name| allow.contains(*name))
        .cloned()
        .collect();
    let mut changed = false;
    for name in old.union(&desired) {
        actor.require_directory_group(scope, name)?;
        if tx.get::<Group>("groups", name)?.is_none() {
            return Err(Error::bad(
                "Cloud directory mappings require an existing local group",
            ));
        }
        changed |= crate::management::write_group(
            config,
            tx,
            actor,
            name,
            crate::management::GroupIntent::DirectoryMember {
                user_id: uid,
                present: desired.contains(name),
                scope,
            },
            crate::management::GroupAudit::Scoped {
                action: "group.cloud_directory_membership",
                target: name,
                scope,
            },
        )?
        .changed;
    }
    Ok(changed)
}

// Check the resources that reconciliation can touch before reporting plan or
// snapshot conflicts. Reconcile repeats these checks at each write boundary.
pub(crate) fn authorize_reconcile(
    tx: &Tx<'_>,
    actor: &Principal,
    settings: &Settings,
    snapshot: &[Entry],
) -> Result<()> {
    actor.require("directory.sync", &settings.resource())?;
    let scope = settings.resource();
    for entry in snapshot {
        actor.require_directory_user(&scope, &entry.username, None)?;
        for group in &entry.groups {
            if settings.reconcile_groups().contains_key(group) {
                actor.require_directory_group(&scope, group)?;
            }
        }
    }
    for (_, binding) in tx.list::<Binding>("cloud_directory_bindings")? {
        if binding.kind != settings.kind || binding.directory != settings.id {
            continue;
        }
        if let Some(user) = tx.get::<User>("users", &binding.user_id)? {
            actor.require_directory_user(&scope, &user.username, Some(&user.id))?;
        }
        for group in &binding.groups {
            if settings.reconcile_groups().contains_key(group) {
                actor.require_directory_group(&scope, group)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn reconcile(
    config: &crate::config::Config,
    tx: &Tx<'_>,
    actor: &Principal,
    settings: &Settings,
    snapshot: &[Entry],
) -> Result<Vec<Change>> {
    actor.require("directory.sync", &settings.resource())?;
    let scope = settings.resource();
    let allow: BTreeSet<_> = settings.reconcile_groups().keys().cloned().collect();
    let mut remaining: BTreeMap<_, _> = tx
        .list::<Binding>("cloud_directory_bindings")?
        .into_iter()
        .filter(|(_, binding)| binding.kind == settings.kind && binding.directory == settings.id)
        .map(|(_, binding)| (binding.external_id.clone(), binding))
        .collect();
    if remaining
        .values()
        .any(|binding| binding.identity_fingerprint != settings.reconcile_identity_fingerprint())
    {
        return Err(Error::conflict(
            "Cloud directory identity mapping changed while accounts are linked; use a new directory ID",
        ));
    }
    let mut changes = Vec::new();
    for entry in snapshot {
        let old = remaining.remove(&entry.external_id);
        actor.require_directory_user(&scope, &entry.username, None)?;
        let owner = crate::management::CloudUserOwner {
            kind: settings.kind,
            directory: &settings.id,
            tenant: settings.reconcile_tenant(),
            identity_fingerprint: settings.reconcile_identity_fingerprint(),
            external_id: &entry.external_id,
        };
        let mut user = if let Some(binding) = &old {
            let user = tx
                .get::<User>("users", &binding.user_id)?
                .ok_or_else(|| Error::conflict("Cloud directory owned user is missing"))?;
            if user.username != entry.username {
                return Err(Error::conflict(
                    "Linked cloud directory username changed; create a new plan",
                ));
            }
            user
        } else {
            User {
                has_passkeys: false,
                totp_settings: Default::default(),
                pairwise_seed: crypto::random_token(""),
                id: crypto::id(),
                username: entry.username.clone(),
                email: entry.email.clone(),
                display_name: entry.display_name.clone(),
                password_hash: String::new(),
                enabled: !entry.disabled,
                admin: false,
                epoch: 0,
                totp_secret: None,
                totp_pending: None,
                totp_last_step: None,
                created_at: now(),
                attributes: Default::default(),
                email_verified: false,
                subjects: Default::default(),
                recovery_codes: Default::default(),
            }
        };
        let is_new = old.is_none();
        let previous = old.as_ref().map(|_| user.clone());
        // The shared group writer validates members against local user rows.
        if is_new {
            crate::management::stage_cloud_user(tx, actor, owner, &user)?;
        } else {
            crate::management::check_cloud_user_owner(tx, actor, owner, &user)?;
        }
        let previous_groups = old
            .as_ref()
            .map(|binding| binding.groups.clone())
            .unwrap_or_default();
        let groups_changed = membership(
            config,
            tx,
            actor,
            &scope,
            &user.id,
            &allow,
            &previous_groups,
            &entry.groups,
        )?;
        let email_changed = user.email != entry.email;
        let display_changed = user.display_name != entry.display_name;
        let enabling = !entry.disabled && !user.enabled;
        let disabling = entry.disabled && user.enabled;
        if email_changed {
            user.email_verified = false;
            user.email = entry.email.clone();
        }
        if display_changed {
            user.display_name = entry.display_name.clone();
        }
        if disabling {
            user.enabled = false;
        } else if enabling {
            user.enabled = true;
        }
        let security = !is_new && (email_changed || disabling);
        if security {
            user.epoch = user.epoch.saturating_add(1);
        }
        let changed =
            is_new || email_changed || display_changed || disabling || enabling || groups_changed;
        if changed {
            crate::management::write_cloud_user(
                tx,
                actor,
                owner,
                previous.as_ref(),
                &user,
                false,
                security,
            )?;
            let action = if is_new {
                "create"
            } else if disabling {
                "disable"
            } else {
                "update"
            };
            changes.push(Change {
                username: user.username.clone(),
                action: action.into(),
                groups: entry.groups.clone(),
            });
        }
        let binding = Binding {
            kind: settings.kind.into(),
            directory: settings.id.clone(),
            tenant: settings.reconcile_tenant().to_owned(),
            identity_fingerprint: settings.reconcile_identity_fingerprint().to_owned(),
            external_id: entry.external_id.clone(),
            user_id: user.id.clone(),
            groups: entry
                .groups
                .iter()
                .filter(|name| allow.contains(*name))
                .cloned()
                .collect(),
        };
        tx.put(
            "cloud_directory_bindings",
            &binding_key(settings.kind, &settings.id, &entry.external_id),
            &binding,
        )?;
        tx.put("cloud_directory_users", &user.id, &binding)?;
    }
    // Only a completed snapshot reaches this loop. Missing linked users are disabled, not deleted.
    for (_, binding) in remaining {
        let mut user = tx
            .get::<User>("users", &binding.user_id)?
            .ok_or_else(|| Error::conflict("Cloud directory owned user is missing"))?;
        let owner = crate::management::CloudUserOwner {
            kind: settings.kind,
            directory: &settings.id,
            tenant: settings.reconcile_tenant(),
            identity_fingerprint: settings.reconcile_identity_fingerprint(),
            external_id: &binding.external_id,
        };
        crate::management::check_cloud_user_owner(tx, actor, owner, &user)?;
        let previous = user.clone();
        let groups_changed = membership(
            config,
            tx,
            actor,
            &scope,
            &user.id,
            &allow,
            &binding.groups,
            &BTreeSet::new(),
        )?;
        if user.enabled || groups_changed {
            let security = user.enabled;
            if security {
                user.epoch = user.epoch.saturating_add(1);
            }
            user.enabled = false;
            crate::management::write_cloud_user(
                tx,
                actor,
                owner,
                Some(&previous),
                &user,
                true,
                security,
            )?;
            changes.push(Change {
                username: user.username.clone(),
                action: "disable".into(),
                groups: BTreeSet::new(),
            });
        }
        let mut binding = binding;
        binding.groups.clear();
        tx.put(
            "cloud_directory_bindings",
            &binding_key(settings.kind, &settings.id, &binding.external_id),
            &binding,
        )?;
        tx.put("cloud_directory_users", &user.id, &binding)?;
    }
    changes.sort_by(|left, right| {
        (&left.username, &left.action).cmp(&(&right.username, &right.action))
    });
    Ok(changes)
}
