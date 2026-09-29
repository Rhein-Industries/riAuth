//! Concrete LDAP import planning, apply, login, and persistence transactions.
use crate::{
    agent::Principal,
    config::LdapReconciliationQuota,
    connector_guard::{
        ApplyGate, ReconciliationMode, RemovalImpact, ReviewBinding, plan_content, reconcile_plan,
        require_backup_safe_record,
    },
    core::{Core, Delivery, audit},
    crypto::{self, digest, now},
    directory::{
        Binding, Budget, Change, Directory, Entry, LDAP_APPLY_SNAPSHOTS, LDAP_SNAPSHOTS,
        LOGIN_BUDGET, Plan, Snapshot, SnapshotDraft, binding_key, stable_id, unavailable,
    },
    error::{Error, Result},
    model::{AuthenticationTransaction, Group, Identity, User, UserView},
    source::SourceIdentity,
    store::Tx,
};
use ldap3::{Scope, SearchEntry};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

/// Apply validation has its own durable cookie and is bound to one immutable
/// reviewed plan. A new plan cannot inherit pages from an old apply crawl.
#[derive(Clone, Serialize, Deserialize)]
struct ApplySnapshotDraft {
    plan_id: String,
    review: ReviewBinding,
    draft: SnapshotDraft,
}
impl ApplySnapshotDraft {
    fn new(
        directory: &str,
        actor: &Principal,
        plan: &Plan,
        quota: &LdapReconciliationQuota,
    ) -> Self {
        Self {
            plan_id: plan.id.clone(),
            review: plan.review.clone(),
            draft: SnapshotDraft::new(
                directory.into(),
                actor.id.clone(),
                plan.revision,
                plan.fingerprint.clone(),
                plan.review.authority_digest.clone(),
                quota,
            ),
        }
    }
    fn valid(&self, directory_id: &str, directory: &Directory, plan: &Plan) -> bool {
        self.plan_id == plan.id
            && self.review == plan.review
            && self.draft.directory == directory_id
            && self.draft.actor == plan.actor
            && self.draft.revision == plan.revision
            && self.draft.fingerprint == plan.fingerprint
            && self.draft.authority_digest == plan.review.authority_digest
            && self.draft.expires_at > now()
            && !self.draft.complete(directory)
            && self.draft.phase <= directory.group_user_filters.len()
    }
    fn bounded(&self, quota: &LdapReconciliationQuota) -> Result<()> {
        self.draft.bounded(quota)?;
        if serde_json::to_vec(self).map_err(Error::internal)?.len() > quota.max_snapshot_bytes {
            return Err(Error::bad("LDAP snapshot staging quota exceeded"));
        }
        Ok(())
    }
    fn progress(&self, directory: &Directory, restarted: bool) -> Value {
        let mut progress = self.draft.progress(directory, restarted);
        progress["plan_id"] = json!(self.plan_id);
        progress["operation"] = json!("apply_validation");
        progress
    }
}
#[derive(Serialize, Deserialize, Default)]
struct Attempts {
    start: u64,
    count: u32,
}
impl Core {
    fn directory_mode(&self, id: &str) -> ReconciliationMode {
        self.config
            .ldap_reconciliation_modes
            .get(id)
            .copied()
            .unwrap_or_default()
    }

    fn directory_fingerprint(&self, id: &str, directory: &Directory) -> Result<String> {
        let quota = self.config.reconciliation_quotas.ldap;
        quota
            .validate()
            .map_err(|error| Error::bad(error.to_string()))?;
        let base = directory.fingerprint()?;
        let base = if quota == LdapReconciliationQuota::default() {
            base
        } else {
            crate::connector_guard::hash(&(base, quota))?
        };
        self.directory_mode(id).fingerprint(&base)
    }

    pub fn directories(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| { let actor=self.principal(tx,token)?; Ok(json!(self.config.directories.iter().filter(|(id,_)|actor.allows("directory.read",&format!("directory/{id}"))).map(|(id,d)|json!({"id":id,"url":d.url,"user_base":d.user_base,"groups":d.group_user_filters.keys().collect::<Vec<_>>(),"reconciliation_mode":self.directory_mode(id)})).collect::<Vec<_>>())) })
    }
    pub fn directory_plan_get(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            let plan = tx
                .get::<Plan>("directory_plans", id)?
                .ok_or_else(|| Error::missing("LDAP plan not found"))?;
            let actor = self.management(
                tx,
                token,
                "directory.read",
                &format!("directory/{}", plan.directory),
            )?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            Ok(json!(plan))
        })
    }
    /// A pending reviewed plan keeps its exact ID after a complete streamed
    /// source check. Eligible apply crawls resume their own durable cookie.
    pub fn directory_reconcile(&self, token: &str, id: &str) -> Result<Value> {
        let directory = self
            .config
            .directories
            .get(id)
            .ok_or_else(|| Error::missing("LDAP directory not configured"))?;
        let mode = self.directory_mode(id);
        let key = digest(id);
        let pending = self.store.read(|tx| {
            let Some(apply) = tx.get::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS, &key)? else {
                return Ok(None);
            };
            let Some(plan) = tx.get::<Plan>("directory_plans", &apply.plan_id)? else {
                return Ok(None);
            };
            if plan.applied
                || plan.expires_at <= now()
                || plan.directory != id
                || plan.fingerprint != self.directory_fingerprint(id, directory)?
                || !apply.valid(id, directory, &plan)
                || mode.decide(&plan.removal_impact)
                    != crate::connector_guard::ReconciliationDecision::Eligible
            {
                return Ok(None);
            }
            let actor = self.directory_snapshot_actor(
                tx,
                token,
                id,
                directory,
                &plan.actor,
                plan.revision,
                &plan.fingerprint,
                &plan.review.authority_digest,
            )?;
            plan.review.validate(tx, &actor, &plan_content(&plan)?)?;
            Ok(Some(json!(plan)))
        })?;
        let plan = match pending {
            Some(plan) => plan,
            None => self.directory_plan_internal(token, id, true)?,
        };
        if plan["decision"] == "snapshot_in_progress" {
            return Ok(json!({"decision":"snapshot_in_progress","mode":mode,"snapshot":plan}));
        }
        let impact: RemovalImpact =
            serde_json::from_value(plan["removal_impact"].clone()).map_err(Error::internal)?;
        reconcile_plan(mode, &impact, plan, |plan_id| {
            self.directory_apply_confirmed(token, plan_id, None)
        })
    }

    pub fn directory_plan(&self, token: &str, id: &str) -> Result<Value> {
        self.directory_plan_internal(token, id, false)
    }

    fn directory_snapshot_actor(
        &self,
        tx: &Tx<'_>,
        token: &str,
        id: &str,
        directory: &Directory,
        expected_actor: &str,
        revision: u64,
        fingerprint: &str,
        authority_digest: &str,
    ) -> Result<Principal> {
        let actor = self.management(tx, token, "directory.sync", &format!("directory/{id}"))?;
        if actor.id != expected_actor
            || tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision
            || self.directory_fingerprint(id, directory)? != fingerprint
            || ReviewBinding::new(tx, &actor, &json!([id, revision, fingerprint]))?.authority_digest
                != authority_digest
        {
            return Err(Error::conflict(
                "LDAP source, authority or local revision changed during snapshot",
            ));
        }
        Ok(actor)
    }

    fn directory_apply_actor(
        &self,
        tx: &Tx<'_>,
        token: &str,
        directory: &Directory,
        expected: &Plan,
        reviewed_plan: Option<&str>,
    ) -> Result<Principal> {
        let actor = self.directory_snapshot_actor(
            tx,
            token,
            &expected.directory,
            directory,
            &expected.actor,
            expected.revision,
            &expected.fingerprint,
            &expected.review.authority_digest,
        )?;
        let stored = tx
            .get::<Plan>("directory_plans", &expected.id)?
            .ok_or_else(|| Error::missing("LDAP plan not found"))?;
        if stored.applied
            || stored.directory != expected.directory
            || stored.review != expected.review
            || plan_content(&stored)? != plan_content(expected)?
            || stored.expires_at <= now()
        {
            return Err(Error::conflict(
                "LDAP plan changed during snapshot validation; create a new plan",
            ));
        }
        expected
            .review
            .validate(tx, &actor, &plan_content(expected)?)?;
        expected
            .review
            .confirm(&expected.id, &expected.removal_impact, reviewed_plan)?;
        Ok(actor)
    }

    fn directory_plan_internal(&self, token: &str, id: &str, supersede: bool) -> Result<Value> {
        let directory = self
            .config
            .directories
            .get(id)
            .ok_or_else(|| Error::missing("LDAP directory not configured"))?;
        let quota = self.config.reconciliation_quotas.ldap;
        quota
            .validate()
            .map_err(|error| Error::bad(error.to_string()))?;
        let key = digest(id);
        let (actor, revision, fingerprint, authority_digest, prior, mut draft, restarted) =
            self.store.read(|tx| {
                let actor =
                    self.management(tx, token, "directory.sync", &format!("directory/{id}"))?;
                let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
                let fingerprint = self.directory_fingerprint(id, directory)?;
                let authority_digest =
                    ReviewBinding::new(tx, &actor, &json!([id, revision, fingerprint]))?
                        .authority_digest;
                let previous = tx.get::<SnapshotDraft>(LDAP_SNAPSHOTS, &key)?;
                let prior = previous
                    .as_ref()
                    .map(|draft| (draft.id.clone(), draft.sequence));
                let valid = previous.as_ref().is_some_and(|draft| {
                    draft.directory == id
                        && draft.actor == actor.id
                        && draft.revision == revision
                        && draft.fingerprint == fingerprint
                        && draft.authority_digest == authority_digest
                        && draft.expires_at > now()
                        && draft.phase <= directory.group_user_filters.len()
                });
                let restarted = previous.is_some() && !valid;
                let draft = previous.filter(|_| valid).unwrap_or_else(|| {
                    SnapshotDraft::new(
                        id.into(),
                        actor.id.clone(),
                        revision,
                        fingerprint.clone(),
                        authority_digest.clone(),
                        &quota,
                    )
                });
                Ok((
                    actor,
                    revision,
                    fingerprint,
                    authority_digest,
                    prior,
                    draft,
                    restarted,
                ))
            })?;
        directory.advance_snapshot(&mut draft, &quota)?;
        draft.expires_at = now().saturating_add(quota.draft_ttl_seconds);
        draft.bounded(&quota)?;
        let same_prior = |tx: &Tx<'_>| -> Result<bool> {
            let current = tx.get::<SnapshotDraft>(LDAP_SNAPSHOTS, &key)?;
            Ok(current.as_ref().map(|draft| (&draft.id, draft.sequence))
                == prior.as_ref().map(|(id, sequence)| (id, *sequence)))
        };
        if !draft.complete(directory) {
            return self.store.write(|tx| {
                self.directory_snapshot_actor(
                    tx,
                    token,
                    id,
                    directory,
                    &actor.id,
                    revision,
                    &fingerprint,
                    &authority_digest,
                )?;
                if !same_prior(tx)? {
                    return Err(Error::conflict(
                        "LDAP snapshot advanced concurrently; resume the latest cursor",
                    ));
                }
                tx.put(LDAP_SNAPSHOTS, &key, &draft)?;
                Ok(draft.progress(directory, restarted))
            });
        }
        let snapshot = draft.into_snapshot();
        let (changes, impact) = self.store.preview(|tx| {
            self.directory_snapshot_actor(
                tx,
                token,
                id,
                directory,
                &actor.id,
                revision,
                &fingerprint,
                &authority_digest,
            )?;
            let impact = removal_impact(tx, id, &snapshot.users)?;
            Ok((
                reconcile(&self.config, tx, &actor, id, directory, &snapshot)?,
                impact,
            ))
        })?;
        self.store.write(|tx| {
            let current_actor = self.directory_snapshot_actor(
                tx,
                token,
                id,
                directory,
                &actor.id,
                revision,
                &fingerprint,
                &authority_digest,
            )?;
            if !same_prior(tx)? {
                return Err(Error::conflict(
                    "LDAP snapshot advanced concurrently; resume the latest cursor",
                ));
            }
            let plans = tx.list::<Plan>("directory_plans")?;
            if supersede {
                if let Some((_, existing)) = plans.iter().find(|(_, existing)| {
                    existing.actor == actor.id
                        && existing.directory == id
                        && !existing.applied
                        && existing.expires_at > now()
                        && existing.revision == revision
                        && existing.fingerprint == fingerprint
                        && existing.entries == snapshot.users
                        && existing.changes == changes
                        && existing.removal_impact == impact
                        && plan_content(existing)
                            .and_then(|content| {
                                existing.review.validate(tx, &current_actor, &content)
                            })
                            .is_ok()
                }) {
                    if prior.is_some() {
                        tx.delete(LDAP_SNAPSHOTS, &key)?;
                    }
                    return Ok(json!(existing));
                }
            }
            if plans
                .iter()
                .filter(|(_, p)| {
                    p.actor == actor.id
                        && p.expires_at > now()
                        && !p.applied
                        && !(supersede && p.directory == id)
                })
                .count()
                >= 16
            {
                return Err(Error::conflict("At most 16 unexpired LDAP plans per actor"));
            }
            let mut plan = Plan {
                id: crypto::id(),
                directory: id.into(),
                actor: actor.id.clone(),
                revision,
                expires_at: now() + 300,
                fingerprint: self.directory_fingerprint(id, directory)?,
                entries: snapshot.users,
                changes,
                removal_impact: impact,
                review: ReviewBinding::default(),
                applied: false,
            };
            plan.review = ReviewBinding::new(tx, &actor, &plan_content(&plan)?)?;
            plan.review
                .validate(tx, &current_actor, &plan_content(&plan)?)?;
            require_backup_safe_record(&plan, "LDAP plan exceeds the backup-safe record limit")?;
            if supersede {
                for (old_id, old) in plans {
                    if old.actor == actor.id && old.directory == id && !old.applied {
                        tx.delete("directory_plans", &old_id)?;
                    }
                }
            }
            tx.put("directory_plans", &plan.id, &plan)?;
            if prior.is_some() {
                tx.delete(LDAP_SNAPSHOTS, &key)?;
            }
            crate::delegation::audit_scoped(
                tx,
                &actor,
                "directory.plan",
                id,
                &format!("directory/{id}"),
            )?;
            Ok(json!(plan))
        })
    }
    pub fn directory_apply(&self, token: &str, id: &str) -> Result<Value> {
        self.directory_apply_confirmed(token, id, None)
    }
    pub fn directory_apply_confirmed(
        &self,
        token: &str,
        id: &str,
        reviewed_plan: Option<&str>,
    ) -> Result<Value> {
        let plan: Plan =
            serde_json::from_value(self.directory_plan_get(token, id)?).map_err(Error::internal)?;
        let directory = self
            .config
            .directories
            .get(&plan.directory)
            .ok_or_else(Error::forbidden)?;
        let quota = self.config.reconciliation_quotas.ldap;
        quota
            .validate()
            .map_err(|error| Error::bad(error.to_string()))?;
        let key = digest(&plan.directory);
        let initially_applied = plan.applied;
        let snapshot_prior =
            if initially_applied {
                self.store.read(|tx| {
                    let actor = self.management(
                        tx,
                        token,
                        "directory.sync",
                        &format!("directory/{}", plan.directory),
                    )?;
                    if actor.id != plan.actor {
                        return Err(Error::forbidden());
                    }
                    Ok(())
                })?;
                None
            } else {
                let (prior, mut apply, restarted) = self.store.read(|tx| {
                    let actor =
                        self.directory_apply_actor(tx, token, directory, &plan, reviewed_plan)?;
                    let previous = tx.get::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS, &key)?;
                    let prior = previous
                        .as_ref()
                        .map(|apply| (apply.draft.id.clone(), apply.draft.sequence));
                    let valid = previous
                        .as_ref()
                        .is_some_and(|apply| apply.valid(&plan.directory, directory, &plan));
                    let restarted = previous.is_some() && !valid;
                    let apply = previous.filter(|_| valid).unwrap_or_else(|| {
                        ApplySnapshotDraft::new(&plan.directory, &actor, &plan, &quota)
                    });
                    apply.bounded(&quota)?;
                    Ok((prior, apply, restarted))
                })?;
                directory.advance_snapshot(&mut apply.draft, &quota)?;
                apply.draft.expires_at = now().saturating_add(quota.draft_ttl_seconds);
                apply.bounded(&quota)?;
                if !apply.draft.complete(directory) {
                    return self.store.write(|tx| {
                    self.directory_apply_actor(tx, token, directory, &plan, reviewed_plan)?;
                    let current = tx.get::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS, &key)?;
                    if current.as_ref().map(|apply| (&apply.draft.id, apply.draft.sequence))
                        != prior.as_ref().map(|(id, sequence)| (id, *sequence))
                    {
                        return Err(Error::conflict(
                            "LDAP apply snapshot advanced concurrently; resume the latest cursor"));
                    }
                    tx.put(LDAP_APPLY_SNAPSHOTS, &key, &apply)?;
                    Ok(apply.progress(directory, restarted))
                });
                }
                self.store.read(|tx| {
                    self.directory_apply_actor(tx, token, directory, &plan, reviewed_plan)
                        .map(|_| ())
                })?;
                if apply.draft.into_snapshot().users != plan.entries {
                    return Err(Error::conflict(
                        "LDAP changed after planning; create a new plan",
                    ));
                }
                prior
            };
        let observed_review = plan.review.clone();
        let planned_directory = plan.directory.clone();
        self.mutation(token, |tx| {
            let actor = self.management(
                tx,
                token,
                "directory.sync",
                &format!("directory/{}", plan.directory),
            )?;
            let mut plan = tx
                .get::<Plan>("directory_plans", id)?
                .ok_or_else(|| Error::missing("LDAP plan not found"))?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            if plan.directory != planned_directory || plan.review != observed_review {
                return Err(Error::conflict(
                    "LDAP plan directory changed; create a new plan",
                ));
            }
            if initially_applied && !plan.applied {
                return Err(Error::conflict(
                    "LDAP plan changed during snapshot validation; create a new plan",
                ));
            }
            if plan.applied {
                return Ok(json!({"id":id,"applied":true,"changes":plan.changes}));
            }
            self.directory_apply_actor(tx, token, directory, &plan, reviewed_plan)?;
            let current = tx.get::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS, &key)?;
            if current
                .as_ref()
                .map(|apply| (&apply.draft.id, apply.draft.sequence))
                != snapshot_prior
                    .as_ref()
                    .map(|(id, sequence)| (id, *sequence))
            {
                return Err(Error::conflict(
                    "LDAP apply snapshot advanced concurrently; resume the latest cursor",
                ));
            }
            let impact = removal_impact(tx, &plan.directory, &plan.entries)?;
            ApplyGate {
                id,
                revision: plan.revision,
                expires_at: plan.expires_at,
                fingerprint_matches: plan.fingerprint
                    == self.directory_fingerprint(&plan.directory, directory)?,
                expected_impact: &plan.removal_impact,
                observed_impact: &impact,
                review: &plan.review,
                reviewed_plan,
            }
            .validate(tx, &actor, &plan)?;
            let changes = reconcile(
                &self.config,
                tx,
                &actor,
                &plan.directory,
                directory,
                &Snapshot {
                    users: plan.entries.clone(),
                },
            )?;
            if changes != plan.changes {
                return Err(Error::conflict("LDAP plan no longer matches local state"));
            }
            plan.applied = true;
            tx.put("directory_plans", id, &plan)?;
            if current.is_some() {
                tx.delete(LDAP_APPLY_SNAPSHOTS, &key)?;
            }
            crate::delegation::audit_scoped(
                tx,
                &actor,
                "directory.apply",
                &plan.directory,
                &format!("directory/{}", plan.directory),
            )?;
            Ok(json!({"id":id,"applied":true,"changes":changes}))
        })
    }
    /// The normal CLI password login dispatches here only for explicitly imported accounts.
    pub(crate) fn directory_login(
        &self,
        username: &str,
        password: &str,
        otp: Option<&str>,
        transaction: Option<&str>,
        delivery: Delivery,
    ) -> Result<Option<Value>> {
        let binding = self.store.read(|tx| {
            let Some(uid) = tx.get::<String>("usernames", username)? else {
                return Ok(None);
            };
            tx.get::<Binding>("directory_users", &uid)
        })?;
        let Some(binding) = binding else {
            return Ok(None);
        };
        let directory = self
            .config
            .directories
            .get(&binding.directory)
            .ok_or_else(Error::unauthorized)?;
        let before = self.store.write(|tx| {
            let mut attempts = tx
                .get::<Attempts>("directory_attempts", &binding.user_id)?
                .unwrap_or_default();
            if attempts.start + 900 <= now() {
                attempts = Attempts {
                    start: now(),
                    count: 0,
                };
            }
            if attempts.count >= 5 {
                return Err(Error::new(
                    axum::http::StatusCode::TOO_MANY_REQUESTS,
                    "rate_limited",
                    "Too many login attempts",
                ));
            }
            attempts.count += 1;
            tx.put("directory_attempts", &binding.user_id, &attempts)?;
            tx.get::<User>("users", &binding.user_id)?
                .ok_or_else(Error::unauthorized)
        })?;
        let eligible = before.enabled
            && !before.admin
            && binding.identity_fingerprint == directory.identity_fingerprint()
            && !password.is_empty()
            && password.len() <= 4096;
        let authenticated = if eligible {
            // Re-check the immutable ID and eligibility under the bound DN before attempting the password.
            let budget = Budget::Until(Instant::now() + LOGIN_BUDGET);
            let mut conn = directory.service(budget)?;
            let (rows, result) = conn
                .with_timeout(budget.next()?)
                .search(
                    &binding.dn,
                    Scope::Base,
                    &directory.user_filter,
                    vec![directory.id_attribute.clone()],
                )
                .map_err(|_| unavailable())?
                .success()
                .map_err(|_| unavailable())?;
            let matches = result.refs.is_empty()
                && rows.len() == 1
                && !rows[0].is_ref()
                && stable_id(
                    &SearchEntry::construct(rows[0].clone()),
                    &directory.id_attribute,
                )? == binding.external_id;
            let _ = conn.unbind();
            if matches {
                let mut conn = directory.connection(budget)?;
                let result = conn
                    .with_timeout(budget.next()?)
                    .simple_bind(&binding.dn, password)
                    .map_err(|_| unavailable())?;
                let _ = conn.unbind();
                match result.rc {
                    0 => true,
                    49 => false,
                    _ => return Err(unavailable()),
                }
            } else {
                false
            }
        } else {
            false
        };
        self.store.write(|tx| {
            let mut user=tx.get::<User>("users",&binding.user_id)?.ok_or_else(Error::unauthorized)?;
            let current=tx.get::<Binding>("directory_users",&user.id)?.ok_or_else(Error::unauthorized)?;
            let mut valid=authenticated && user.enabled && !user.admin && user.epoch==before.epoch && current.dn==binding.dn && current.external_id==binding.external_id && current.identity_fingerprint==binding.identity_fingerprint;
            if valid && let Some(secret)=&user.totp_secret {
                if let Some(code)=otp.filter(|c|c.starts_with("ri_recovery_")) {valid=user.recovery_codes.remove(&digest(code));}
                else {let step=crypto::totp_step_with(secret,&user.username,otp.unwrap_or(""),now(),user.totp_last_step,&user.totp_settings)?;valid=step.is_some();if valid {user.totp_last_step=step;}}
            }
            if !valid {audit(tx,"anonymous","directory.login_failed",username)?;return Ok(Err(Error::new(axum::http::StatusCode::UNAUTHORIZED,"invalid_credentials","Invalid username, password, or one-time code")));}
            // The verified factors are spent even when the transaction binding is refused.
            tx.put("users",&user.id,&user)?;tx.delete("directory_attempts",&user.id)?;
            let challenge=transaction.map(|t|{
                let challenge=tx.get::<AuthenticationTransaction>("authentication",&digest(t))?.filter(|c|c.expires_at>now()&&c.authenticated_session.is_none()).ok_or_else(||Error::bad("Authentication transaction expired or used"))?;
                crate::oidc::reject_embedded_stage(&challenge)?;
                if challenge.user_id.as_ref().is_some_and(|id|id!=&user.id) {return Err(Error::forbidden());}
                Ok(challenge)
            }).transpose();
            let challenge=match challenge {
                Err(error) if error.status.is_client_error()=>{audit(tx,&user.id,"login.transaction_rejected",username)?;return Ok(Err(error));}
                challenge=>challenge?,
            };
            let mfa=user.totp_secret.is_some();
            let identity=Identity{user_id:user.id.clone(),epoch:user.epoch,mfa,auth_time:now(),session_id:String::new(),amr:if mfa {vec!["pwd".into(),"otp".into()]}else{vec!["pwd".into()]},source:Some(SourceIdentity{id:format!("ldap/{}",binding.directory),fingerprint:directory.fingerprint()?,link:binding_key(&binding.directory,&binding.external_id),pin_retired:false})};
            if delivery==Delivery::Browser {
                let staged=self.stage_browser_login(tx,identity,now()+self.config.session_ttl,"password")?;
                audit(tx,&user.id,"directory.login_succeeded",&staged)?;
                return Ok(Ok(Some(json!({"staged":staged,"user":UserView::from(&user)}))));
            }
            let (session,token)=self.mint_bearer_session(tx,identity,now()+self.config.session_ttl)?;
            if let Some(mut challenge)=challenge{challenge.authenticated_session=Some(session.id.clone());tx.put("authentication",&digest(transaction.unwrap()),&challenge)?;}
            audit(tx,&user.id,"directory.login_succeeded",&session.id)?;Ok(Ok(Some(json!({"session_token":token,"expires_at":session.expires_at,"user":UserView::from(&user)}))))
        })?
    }
}

fn removal_impact(tx: &Tx<'_>, directory: &str, entries: &[Entry]) -> Result<RemovalImpact> {
    let entries: BTreeMap<_, _> = entries
        .iter()
        .map(|e| (e.external_id.as_str(), e))
        .collect();
    let mut active = 0;
    let mut impact = RemovalImpact::default();
    for (_, binding) in tx.list::<Binding>("directory_bindings")? {
        if binding.directory != directory {
            continue;
        }
        let user = tx
            .get::<User>("users", &binding.user_id)?
            .ok_or_else(|| Error::conflict("LDAP owned user is missing"))?;
        let desired = entries.get(binding.external_id.as_str());
        if desired.is_none() {
            impact.missing_users += 1;
        }
        if user.enabled {
            active += 1;
            if desired.is_none() {
                impact.disabled_users += 1;
            }
        }
        for group in &binding.groups {
            if desired.is_none_or(|e| !e.groups.contains(group))
                && tx
                    .get::<Group>("groups", group)?
                    .is_some_and(|g| g.members.contains(&user.id))
            {
                impact.removed_memberships += 1;
            }
        }
    }
    impact.assess(active);
    Ok(impact)
}

fn reconcile(
    config: &crate::config::Config,
    tx: &Tx<'_>,
    actor: &Principal,
    id: &str,
    directory: &Directory,
    snapshot: &Snapshot,
) -> Result<Vec<Change>> {
    actor.require("directory.sync", &format!("directory/{id}"))?;
    let fingerprint = directory.identity_fingerprint();
    let mut remaining: BTreeMap<_, _> = tx
        .list::<Binding>("directory_bindings")?
        .into_iter()
        .filter(|(_, b)| b.directory == id)
        .map(|(_, b)| (b.external_id.clone(), b))
        .collect();
    if remaining
        .values()
        .any(|b| b.identity_fingerprint != fingerprint)
    {
        return Err(Error::conflict(
            "LDAP identity mapping changed while accounts are linked; use a new directory ID",
        ));
    }
    let mut changes = Vec::new();
    for entry in &snapshot.users {
        actor.require_directory_user(&format!("directory/{id}"), &entry.username, None)?;
        let old = remaining.remove(&entry.external_id);
        let owner = crate::management::DirectoryUserOwner {
            directory: id,
            identity_fingerprint: &fingerprint,
            external_id: &entry.external_id,
        };
        let mut user = if let Some(binding) = &old {
            tx.get::<User>("users", &binding.user_id)?
                .ok_or_else(|| Error::conflict("LDAP owned user is missing"))?
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
                enabled: true,
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
        let previous = old.as_ref().map(|_| user.clone());
        actor.require_directory_user(&format!("directory/{id}"), &user.username, Some(&user.id))?;
        if user.admin || !user.password_hash.is_empty() {
            return Err(Error::conflict(
                "LDAP may only manage non-administrator directory accounts",
            ));
        }
        if tx
            .get::<String>("usernames", &entry.username)?
            .is_some_and(|uid| uid != user.id)
        {
            return Err(Error::conflict(
                "LDAP username collides with an existing account; accounts are never automatically linked",
            ));
        }
        if old.is_none() {
            crate::management::stage_directory_user(tx, actor, owner, &user)?;
        } else {
            crate::management::check_directory_user_owner(tx, actor, owner, &user)?;
        }
        let groups_changed = membership(
            config,
            tx,
            actor,
            id,
            &user.id,
            old.as_ref().map(|b| &b.groups),
            &entry.groups,
        )?;
        let changed = old.as_ref().is_none_or(|b| b.dn != entry.dn)
            || user.username != entry.username
            || user.email != entry.email
            || user.display_name != entry.display_name
            || !user.enabled
            || groups_changed;
        if changed {
            if old.is_some() {
                user.epoch += 1;
            }
            if user.email != entry.email {
                user.email_verified = false;
            }
            user.username = entry.username.clone();
            user.email = entry.email.clone();
            user.display_name = entry.display_name.clone();
            user.enabled = true;
            crate::management::write_directory_user(
                tx,
                actor,
                owner,
                previous.as_ref(),
                &user,
                false,
            )?;
            changes.push(Change {
                username: user.username.clone(),
                action: if old.is_some() { "update" } else { "create" }.into(),
                groups: entry.groups.clone(),
            });
        }
        let binding = Binding {
            directory: id.into(),
            identity_fingerprint: fingerprint.clone(),
            external_id: entry.external_id.clone(),
            user_id: user.id.clone(),
            dn: entry.dn.clone(),
            groups: entry.groups.clone(),
        };
        tx.put(
            "directory_bindings",
            &binding_key(id, &entry.external_id),
            &binding,
        )?;
        tx.put("directory_users", &user.id, &binding)?;
    }
    for (_, mut binding) in remaining {
        let mut user = tx
            .get::<User>("users", &binding.user_id)?
            .ok_or_else(|| Error::conflict("LDAP owned user is missing"))?;
        let owner = crate::management::DirectoryUserOwner {
            directory: id,
            identity_fingerprint: &fingerprint,
            external_id: &binding.external_id,
        };
        let previous = user.clone();
        actor.require_directory_user(&format!("directory/{id}"), &user.username, Some(&user.id))?;
        if user.admin {
            return Err(Error::conflict("LDAP cannot disable an administrator"));
        }
        crate::management::check_directory_user_owner(tx, actor, owner, &user)?;
        let groups_changed = membership(
            config,
            tx,
            actor,
            id,
            &user.id,
            Some(&binding.groups),
            &BTreeSet::new(),
        )?;
        if user.enabled || groups_changed {
            user.enabled = false;
            user.epoch += 1;
            crate::management::write_directory_user(
                tx,
                actor,
                owner,
                Some(&previous),
                &user,
                true,
            )?;
            changes.push(Change {
                username: user.username.clone(),
                action: "disable".into(),
                groups: BTreeSet::new(),
            });
        }
        binding.groups.clear();
        tx.put(
            "directory_bindings",
            &binding_key(id, &binding.external_id),
            &binding,
        )?;
        tx.put("directory_users", &user.id, &binding)?;
    }
    changes.sort_by(|a, b| a.username.cmp(&b.username));
    Ok(changes)
}
fn membership(
    config: &crate::config::Config,
    tx: &Tx<'_>,
    actor: &Principal,
    directory_id: &str,
    uid: &str,
    old: Option<&BTreeSet<String>>,
    desired: &BTreeSet<String>,
) -> Result<bool> {
    let empty = BTreeSet::new();
    let old = old.unwrap_or(&empty);
    let mut changed = false;
    let scope = format!("directory/{directory_id}");
    for name in old.union(desired) {
        actor.require_directory_group(&scope, name)?;
        if tx.get::<Group>("groups", name)?.is_none() {
            return Err(Error::bad("LDAP mappings require an existing local group"));
        }
        changed |= crate::management::write_group(
            config,
            tx,
            actor,
            name,
            crate::management::GroupIntent::DirectoryMember {
                user_id: uid,
                present: desired.contains(name),
                scope: &scope,
            },
            crate::management::GroupAudit::Scoped {
                action: "group.directory_membership",
                target: name,
                scope: &scope,
            },
        )?
        .changed;
    }
    Ok(changed)
}
/// The imported directory verifies and changes this account's password, even when its
/// configuration has since been removed.
pub(crate) fn manages(tx: &Tx<'_>, user_id: &str) -> Result<bool> {
    Ok(tx.get::<Binding>("directory_users", user_id)?.is_some())
}
pub(crate) fn validate_identity(core: &Core, tx: &Tx<'_>, identity: &Identity) -> Result<()> {
    let binding = tx.get::<Binding>("directory_users", &identity.user_id)?;
    if let Some(binding) = binding {
        let directory = core
            .config
            .directories
            .get(&binding.directory)
            .ok_or_else(Error::unauthorized)?;
        if binding.identity_fingerprint != directory.identity_fingerprint() {
            return Err(Error::unauthorized());
        }
        if let Some(source) = &identity.source
            && source.id.starts_with("ldap/")
            && (source.id != format!("ldap/{}", binding.directory)
                || source.fingerprint != directory.fingerprint()?
                || source.link != binding_key(&binding.directory, &binding.external_id))
        {
            return Err(Error::unauthorized());
        }
    } else if identity
        .source
        .as_ref()
        .is_some_and(|s| s.id.starts_with("ldap/"))
    {
        return Err(Error::unauthorized());
    }
    Ok(())
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    // Sweep by stored expiry, including drafts for directory IDs no longer in
    // configuration. A live draft remains resumable until its own deadline.
    for (id, draft) in tx.maintenance_page::<SnapshotDraft>(LDAP_SNAPSHOTS)? {
        if draft.expires_at <= at {
            tx.delete(LDAP_SNAPSHOTS, &id)?;
        }
    }
    for (id, apply) in tx.maintenance_page::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS)? {
        if apply.draft.expires_at <= at {
            tx.delete(LDAP_APPLY_SNAPSHOTS, &id)?;
        }
    }
    for (id, p) in tx.maintenance_page::<Plan>("directory_plans")? {
        if p.expires_at + 86400 < at {
            tx.delete("directory_plans", &id)?;
        }
    }
    for (id, a) in tx.maintenance_page::<Attempts>("directory_attempts")? {
        if a.start + 1800 < at {
            tx.delete("directory_attempts", &id)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "directory_tests.rs"]
mod backup_and_snapshot_regression;
