//! Agent-approved outbound SCIM reconciliation with immutable plans and durable jobs.
//! Targets use either a static `token_file` or an OAuth `client_credentials` / `refresh_token` grant.
use crate::{
    agent::{Agent, Principal},
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Group, User},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use zeroize::{Zeroize, Zeroizing};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub url: String,
    /// Static bearer credential. Mutually exclusive with `oauth`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_file: Option<PathBuf>,
    /// OAuth client used to acquire a bearer token. Mutually exclusive with `token_file`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth: Option<Oauth>,
    pub ca_file: Option<PathBuf>,
    pub groups: BTreeSet<String>,
    #[serde(default)]
    pub export_groups: bool,
}
/// Supported OAuth grants for an outbound SCIM target. The password grant is not accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OauthGrant {
    ClientCredentials,
    RefreshToken,
}
impl OauthGrant {
    fn as_str(self) -> &'static str {
        match self {
            Self::ClientCredentials => "client_credentials",
            Self::RefreshToken => "refresh_token",
        }
    }
}
/// OAuth token-endpoint client. Secrets stay in the named private files.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Oauth {
    pub token_url: String,
    pub grant: OauthGrant,
    pub client_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret_file: Option<PathBuf>,
    /// Read on every refresh-token acquisition. riAuth never writes this file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token_file: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
    /// Optional CA for the token endpoint. Falls back to the target `ca_file`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ca_file: Option<PathBuf>,
}
impl Target {
    pub fn validate(&self) -> Result<()> {
        crate::config::validate_server_url(&self.url)
            .map_err(|_| Error::bad("SCIM target requires canonical HTTPS or HTTP loopback"))?;
        if self.groups.is_empty() || self.groups.len() > 64 {
            return Err(Error::bad(
                "Select one to 64 explicit groups for SCIM provisioning",
            ));
        }
        for group in &self.groups {
            crate::core::validate_name(group)?;
        }
        if self.token_file.is_some() == self.oauth.is_some() {
            return Err(Error::bad(
                "SCIM target requires exactly one of token_file or oauth",
            ));
        }
        if let Some(oauth) = &self.oauth {
            validate_oauth(oauth)?;
        }
        Ok(())
    }
    fn fingerprint(&self) -> Result<String> {
        Ok(digest(
            &serde_json::to_string(self).map_err(Error::internal)?,
        ))
    }
    fn http(&self) -> Result<reqwest::blocking::Client> {
        http_client(self.ca_file.as_deref())
    }
    fn token_http(&self) -> Result<reqwest::blocking::Client> {
        let oauth_ca = self
            .oauth
            .as_ref()
            .and_then(|oauth| oauth.ca_file.as_deref());
        http_client(oauth_ca.or(self.ca_file.as_deref()))
    }
    /// Resolve the bearer credential for this target.
    /// Static files are read on every call. OAuth access tokens are cached in memory until
    /// shortly before expiry; client secrets and refresh tokens are re-read when acquiring.
    pub fn bearer(&self, core: &Core, name: &str) -> Result<Bearer> {
        self.validate()?;
        let Some(oauth) = &self.oauth else {
            let path = self.token_file.as_deref().ok_or_else(|| {
                Error::bad("SCIM target requires exactly one of token_file or oauth")
            })?;
            return Ok(Bearer {
                generation: 0,
                token: read_secret_file(path)?,
            });
        };
        let key = cache_key(name, &self.fingerprint()?);
        let lock = target_lock(name);
        let _guard = mutex_guard(&lock);
        let secrets = read_oauth_secrets(oauth)?;
        if let Some(cached) = cached_bearer(&key, &secrets.fingerprint) {
            return Ok(cached);
        }
        let issued = request_token(self, oauth, &secrets)?;
        let generation = store_bearer(name, &key, &secrets.fingerprint, &issued);
        if let Err(error) = persist_oauth_meta(core, name, issued.expires_at, &issued.fingerprint) {
            forget_cache_key(&key);
            return Err(error);
        }
        Ok(Bearer {
            generation,
            token: issued.token,
        })
    }
}
/// In-memory access token. `Debug` redacts the credential.
pub struct Bearer {
    pub(crate) generation: u64,
    token: Zeroizing<String>,
}
impl Bearer {
    pub fn as_str(&self) -> &str {
        &self.token
    }
}
impl std::fmt::Debug for Bearer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Bearer")
            .field("generation", &self.generation)
            .field("token", &"[redacted]")
            .finish()
    }
}
/// Drop every cached access token for `target_name`. The next acquisition contacts the token endpoint.
#[doc(hidden)]
pub fn discard_cached_bearer(target_name: &str) {
    let lock = target_lock(target_name);
    let _guard = mutex_guard(&lock);
    let mut cache = mutex_guard(token_cache());
    let prefix = format!("{target_name}\0");
    cache.retain(|key, _| !key.starts_with(&prefix));
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub kind: String,
    pub local_id: String,
    pub body: Value,
    pub member_ids: Vec<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub target: String,
    pub actor: String,
    pub revision: u64,
    pub expires_at: u64,
    pub target_fingerprint: String,
    pub resources: Vec<Resource>,
}
// Bound both a single snapshot and retained un-applied snapshots. Jobs keep
// their own immutable copy, so replacing a pending plan cannot alter a job.
const MAX_PLAN_RESOURCES: usize = 2_064;
const MAX_PLAN_BYTES: usize = 2 * 1024 * 1024;
const MAX_RETAINED_PLANS: usize = 32;
const MAX_RETAINED_PLAN_BYTES: usize = 16 * 1024 * 1024;
const MAX_RETAINED_JOBS: usize = 64;
const MAX_RETAINED_JOB_BYTES: usize = 32 * 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
struct Job {
    plan: Plan,
    cursor: usize,
    #[serde(default)]
    total: usize,
    completed: bool,
    stale: bool,
    next_attempt: u64,
    attempts: u32,
    lease: Option<String>,
    error: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Link {
    target: String,
    url: String,
    kind: String,
    local_id: String,
    remote_id: String,
    external_id: String,
    body: Value,
}
fn link_key(target: &str, kind: &str, id: &str) -> String {
    digest(&format!("{target}\0{kind}\0{id}"))
}
fn actor(tx: &Tx<'_>, id: &str) -> Result<Principal> {
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
fn job_view(job: &Job) -> Value {
    json!({"id":job.plan.id,"target":job.plan.target,"revision":job.plan.revision,"processed":job.cursor,"total":job.total.max(job.plan.resources.len()),"completed":job.completed,"stale":job.stale,"attempts":job.attempts,"next_attempt":job.next_attempt,"error":job.error})
}

fn compact_terminal_job(job: &mut Job) {
    if job.completed || job.stale {
        job.total = job.total.max(job.plan.resources.len());
        job.plan.resources.clear();
    }
}

fn ensure_job_capacity(tx: &Tx<'_>, next: &Job) -> Result<()> {
    let mut jobs = tx.list::<Job>("provisioning_jobs")?;
    let mut retained_bytes = serde_json::to_vec(next).map_err(Error::internal)?.len();
    let mut terminal = Vec::new();
    for (id, job) in &mut jobs {
        if (job.completed || job.stale) && !job.plan.resources.is_empty() {
            compact_terminal_job(job);
            tx.put("provisioning_jobs", id, job)?;
        }
        let bytes = serde_json::to_vec(job).map_err(Error::internal)?.len();
        retained_bytes = retained_bytes.saturating_add(bytes);
        if job.completed || job.stale {
            terminal.push((id.clone(), job.plan.expires_at, bytes));
        }
    }
    terminal.sort_by(|a, b| (a.1, &a.0).cmp(&(b.1, &b.0)));
    let mut count = jobs.len() + 1;
    for (id, _, bytes) in terminal {
        if count <= MAX_RETAINED_JOBS && retained_bytes <= MAX_RETAINED_JOB_BYTES {
            break;
        }
        tx.delete("provisioning_jobs", &id)?;
        // The job was terminal and is being evicted. Removing its original
        // plan also prevents a recently completed plan from being reapplied.
        tx.delete("provisioning_plans", &id)?;
        count -= 1;
        retained_bytes = retained_bytes.saturating_sub(bytes);
    }
    if count > MAX_RETAINED_JOBS || retained_bytes > MAX_RETAINED_JOB_BYTES {
        return Err(Error::bad(
            "Too many active SCIM jobs; complete existing work before applying another plan",
        ));
    }
    Ok(())
}
impl Core {
    pub fn provisioning_plan_get(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            let principal = self.principal(tx, token)?;
            let plan = tx
                .get::<Plan>("provisioning_plans", id)?
                .ok_or_else(|| Error::missing("Provisioning plan not found"))?;
            principal.require("provisioner.read", &format!("provisioner/{}", plan.target))?;
            if principal.id != plan.actor {
                return Err(Error::forbidden());
            }
            Ok(json!(plan))
        })
    }
    pub fn provisioning_targets(&self, token: &str) -> Result<Value> {
        self.store.read(|tx|{let actor=self.principal(tx,token)?;Ok(json!(self.config.scim_targets.iter().filter(|(id,_)|actor.allows("provisioner.read",&format!("provisioner/{id}"))).map(|(id,t)|json!({"id":id,"url":t.url,"groups":t.groups,"export_groups":t.export_groups})).collect::<Vec<_>>()))})
    }
    pub fn provisioning_plan(&self, token: &str, target_id: &str) -> Result<Value> {
        let target = self
            .config
            .scim_targets
            .get(target_id)
            .ok_or_else(|| Error::missing("SCIM target not configured"))?;
        target.validate()?;
        self.store.write(|tx|{
            let actor=self.management(tx,token,"provisioner.sync",&format!("provisioner/{target_id}"))?;
            let mut groups=Vec::new();let mut selected=BTreeSet::new();
            for name in &target.groups {let group=tx.get::<Group>("groups",name)?.ok_or_else(||Error::bad("SCIM target references a missing group"))?;selected.extend(group.members.iter().cloned());groups.push(group);}
            let prefix=format!("urn:riauth:{}",digest(&self.config.issuer));let mut resources=BTreeMap::new();let mut active=BTreeSet::new();
            for (id,user) in tx.list::<User>("users")? {
                if !selected.contains(&id) || !user.enabled || user.admin {continue;}
                active.insert(id.clone());
                let body=json!({"schemas":[crate::scim::USER],"externalId":format!("{prefix}:Users:{id}"),"userName":user.username,"displayName":user.display_name,"active":true,"emails":user.email.iter().map(|email|json!({"value":email,"primary":true})).collect::<Vec<_>>()});
                resources.insert(("Users".to_owned(),id.clone()),Resource{kind:"Users".into(),local_id:id,body,member_ids:vec![]});
            }
            if active.len()>2000 {return Err(Error::bad("This SCIM target profile supports at most 2000 selected users"));}
            if target.export_groups {
                for group in groups {let id=group.name;resources.insert(("Groups".into(),id.clone()),Resource{kind:"Groups".into(),local_id:id.clone(),body:json!({"schemas":[crate::scim::GROUP],"externalId":format!("{prefix}:Groups:{}",digest(&id)),"displayName":id,"members":[]}),member_ids:group.members.intersection(&active).cloned().collect()});}
            }
            // Retain ownership records and disable departed users; never delete remote accounts.
            for (_,link) in tx.list::<Link>("provisioning_links")?.into_iter().filter(|(_,l)|l.target==target_id) {
                if link.url!=target.url {return Err(Error::conflict("Target URL changed while remote accounts are linked; configure a new target ID"));}
                let key=(link.kind.clone(),link.local_id.clone());
                resources.entry(key).or_insert_with(||{let mut body=link.body;if link.kind=="Users"{body["active"]=json!(false);}else{body["members"]=json!([]);}Resource{kind:link.kind,local_id:link.local_id,body,member_ids:vec![]}});
            }
            if resources.len() > MAX_PLAN_RESOURCES {
                return Err(Error::bad("SCIM plan exceeds the total resource limit"));
            }
            let mut resources:Vec<_>=resources.into_values().collect();resources.sort_by(|a,b|(a.kind!="Users",&a.local_id).cmp(&(b.kind!="Users",&b.local_id)));
            let plan=Plan{id:crypto::id(),target:target_id.into(),actor:actor.id.clone(),revision:tx.get::<u64>("meta","revision")?.unwrap_or(0),expires_at:now()+3600,target_fingerprint:target.fingerprint()?,resources};
            let plan_bytes = serde_json::to_vec(&plan).map_err(Error::internal)?.len();
            if plan_bytes > MAX_PLAN_BYTES {
                return Err(Error::bad("SCIM plan exceeds the serialized size limit"));
            }
            // A fresh plan supersedes the actor's earlier pending plan for
            // this target. Keep the original while its job is still active;
            // completed jobs retain their own record and audit trail.
            for (id, old) in tx.list::<Plan>("provisioning_plans")? {
                let job = tx.get::<Job>("provisioning_jobs", &id)?;
                let active_job = job.as_ref().is_some_and(|job| !job.completed && !job.stale);
                if !active_job
                    && ((old.actor == actor.id && old.target == target_id)
                        || old.expires_at <= now())
                {
                    tx.delete("provisioning_plans", &id)?;
                }
            }
            let retained = tx.list::<Plan>("provisioning_plans")?;
            let retained_bytes = retained.iter().try_fold(0usize, |sum, (_, old)| {
                serde_json::to_vec(old)
                    .map(|bytes| sum.saturating_add(bytes.len()))
                    .map_err(Error::internal)
            })?;
            if retained.len() >= MAX_RETAINED_PLANS
                || retained_bytes.saturating_add(plan_bytes) > MAX_RETAINED_PLAN_BYTES
            {
                return Err(Error::bad("Too many retained SCIM plans; complete or expire older jobs"));
            }
            tx.put("provisioning_plans",&plan.id,&plan)?;audit(tx,&actor.id,"provisioner.plan",target_id)?;
            Ok(json!(plan))
        })
    }
    pub fn provisioning_apply(&self, token: &str, id: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            // A completed job remains an idempotent apply result even after
            // its bulky source plan has been superseded and removed.
            if let Some(job) = tx.get::<Job>("provisioning_jobs", id)? {
                actor.require(
                    "provisioner.sync",
                    &format!("provisioner/{}", job.plan.target),
                )?;
                if actor.id != job.plan.actor {
                    return Err(Error::forbidden());
                }
                return Ok(job_view(&job));
            }
            let plan = tx
                .get::<Plan>("provisioning_plans", id)?
                .ok_or_else(|| Error::missing("Provisioning plan not found"))?;
            actor.require("provisioner.sync", &format!("provisioner/{}", plan.target))?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            let target = self
                .config
                .scim_targets
                .get(&plan.target)
                .ok_or_else(Error::forbidden)?;
            if plan.expires_at <= now()
                || plan.revision != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
                || plan.target_fingerprint != target.fingerprint()?
            {
                return Err(Error::conflict(
                    "Provisioning plan expired or configuration changed",
                ));
            }
            for (_, job) in tx.list::<Job>("provisioning_jobs")? {
                if job.plan.target == plan.target && !job.completed && !job.stale {
                    return Err(Error::conflict(
                        "Target already has an unfinished job; inspect or retry it",
                    ));
                }
            }
            let job = Job {
                total: plan.resources.len(),
                plan,
                cursor: 0,
                completed: false,
                stale: false,
                next_attempt: now(),
                attempts: 0,
                lease: None,
                error: None,
            };
            ensure_job_capacity(tx, &job)?;
            tx.put("provisioning_jobs", id, &job)?;
            audit(tx, &actor.id, "provisioner.apply", &job.plan.target)?;
            Ok(job_view(&job))
        })
    }
    pub fn provisioning_jobs(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<Job>("provisioning_jobs")?
                    .into_iter()
                    .filter(|(_, j)| actor.allows(
                        "provisioner.read",
                        &format!("provisioner/{}", j.plan.target)
                    ))
                    .map(|(_, j)| job_view(&j))
                    .collect::<Vec<_>>()
            ))
        })
    }
    fn claim_provisioning(&self) -> Result<Option<Job>> {
        self.store.write(|tx|{
            for (id,mut job) in tx.due::<Job>("provisioning_jobs", now(), 16)? {
                if job.completed || job.stale || job.next_attempt>now(){continue;}
                let permitted=actor(tx,&job.plan.actor).and_then(|a|a.require("provisioner.sync",&format!("provisioner/{}",job.plan.target))).is_ok();
                if !permitted || job.plan.revision!=tx.get::<u64>("meta","revision")?.unwrap_or(0) || self.config.scim_targets.get(&job.plan.target).is_none_or(|t|t.fingerprint().ok().as_ref()!=Some(&job.plan.target_fingerprint)) || job.plan.expires_at+86400<now() {
                    job.stale=true;job.error=Some("Plan authority or source configuration changed; inspect partial results and create a new plan".into());compact_terminal_job(&mut job);tx.put("provisioning_jobs",&id,&job)?;continue;
                }
                job.next_attempt=now()+60;job.attempts+=1;job.lease=Some(crypto::id());tx.put("provisioning_jobs",&id,&job)?;return Ok(Some(job));
            }
            Ok(None)
        })
    }
    pub fn provisioning_step(&self) -> Result<()> {
        let Some(mut job) = self.claim_provisioning()? else {
            return Ok(());
        };
        let Some(resource) = job.plan.resources.get(job.cursor) else {
            return self.finish_provisioning(&mut job, None);
        };
        let target = self
            .config
            .scim_targets
            .get(&job.plan.target)
            .ok_or_else(Error::forbidden)?
            .clone();
        let result = (|| {
            let mut body = resource.body.clone();
            if resource.kind == "Groups" {
                let members = self.store.read(|tx| {
                    resource
                        .member_ids
                        .iter()
                        .map(|id| {
                            let link = tx
                                .get::<Link>(
                                    "provisioning_links",
                                    &link_key(&job.plan.target, "Users", id),
                                )?
                                .ok_or_else(|| {
                                    Error::conflict("Group user has not been provisioned")
                                })?;
                            Ok(json!({"value":link.remote_id}))
                        })
                        .collect::<Result<Vec<_>>>()
                })?;
                body["members"] = json!(members);
            }
            let external_id = body["externalId"]
                .as_str()
                .ok_or_else(|| Error::internal("Missing external identity"))?
                .to_owned();
            let http = target.http()?;
            let url = format!("{}/{}", target.url.trim_end_matches('/'), resource.kind);
            let filter = format!(
                "externalId eq {}",
                serde_json::to_string(&external_id).map_err(Error::internal)?
            );
            let response = authorized(self, &job.plan.target, &target, &http, |http, token| {
                http.get(&url)
                    .bearer_auth(token)
                    .query(&[("filter", filter.as_str()), ("count", "2")])
                    .header("accept", "application/scim+json")
            })?;
            let (found, _) = scim_json(response)?;
            let list = found["Resources"].as_array().ok_or_else(remote_error)?;
            if found["totalResults"]
                .as_u64()
                .is_none_or(|n| n != list.len() as u64 || n > 1)
                || list.iter().any(|r| r["externalId"] != external_id)
            {
                return Err(Error::conflict(
                    "Remote external identity is ambiguous; refusing to choose an account",
                ));
            }
            let known = self.store.get::<Link>(
                "provisioning_links",
                &link_key(&job.plan.target, &resource.kind, &resource.local_id),
            )?;
            let remote_id = if let Some(found) = list.first() {
                let id = found["id"]
                    .as_str()
                    .filter(|id| !id.is_empty() && id.len() <= 512)
                    .ok_or_else(remote_error)?;
                if known.as_ref().is_some_and(|l| l.remote_id != id) {
                    return Err(Error::conflict(
                        "Remote account ID changed; review before replacing its binding",
                    ));
                }
                let mut item_url = url::Url::parse(&url).map_err(Error::internal)?;
                item_url
                    .path_segments_mut()
                    .map_err(|_| remote_error())?
                    .push(id);
                let response =
                    authorized(self, &job.plan.target, &target, &http, |http, token| {
                        http.get(item_url.clone()).bearer_auth(token)
                    })?;
                let (current, etag) = scim_json(response)?;
                if current["externalId"] != external_id || current["id"] != id {
                    return Err(Error::conflict("Remote account binding changed"));
                }
                let equal = body
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(key, _)| key.as_str() != "schemas")
                    .all(|(key, value)| managed_member_equal(key, value, current.get(key)));
                if !equal {
                    let etag = etag
                        .or_else(|| current["meta"]["version"].as_str().map(String::from))
                        .ok_or_else(|| {
                            Error::bad("SCIM target must supply an ETag for conditional updates")
                        })?;
                    let changes: serde_json::Map<String, Value> = body
                        .as_object()
                        .unwrap()
                        .iter()
                        .filter(|(key, value)| {
                            !["schemas", "externalId"].contains(&key.as_str())
                                && !managed_member_equal(key, value, current.get(*key))
                        })
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    let patch = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","value":changes}]});
                    let response =
                        authorized(self, &job.plan.target, &target, &http, |http, token| {
                            http.patch(item_url.clone())
                                .bearer_auth(token)
                                .header("if-match", etag.clone())
                                .header(
                                    "idempotency-key",
                                    format!(
                                        "ri-{}-{}",
                                        job.plan.id,
                                        digest(&format!("{}:{}", resource.kind, resource.local_id))
                                    ),
                                )
                                .header("content-type", "application/scim+json")
                                .json(&patch)
                        })?;
                    // RFC 7644 allows a successful PATCH to return 204 with no body.
                    // Read the resource back before advancing the durable job so that
                    // a lost or incomplete update cannot be mistaken for success.
                    let updated = if response.status() == reqwest::StatusCode::NO_CONTENT {
                        let response =
                            authorized(self, &job.plan.target, &target, &http, |http, token| {
                                http.get(item_url.clone()).bearer_auth(token)
                            })?;
                        scim_json(response)?.0
                    } else {
                        scim_json(response)?.0
                    };
                    if updated["id"] != id
                        || updated["externalId"] != external_id
                        || !managed_equal(&body, &updated)
                    {
                        return Err(remote_error());
                    }
                }
                id.to_owned()
            } else {
                if body["active"] == false
                    || resource.kind == "Groups" && resource.member_ids.is_empty()
                {
                    return Ok(None);
                }
                if known.is_some() {
                    return Err(Error::conflict(
                        "Linked remote account is missing; review before recreating it",
                    ));
                }
                let response =
                    authorized(self, &job.plan.target, &target, &http, |http, token| {
                        http.post(&url)
                            .bearer_auth(token)
                            .header(
                                "idempotency-key",
                                format!(
                                    "ri-{}-{}",
                                    job.plan.id,
                                    digest(&format!("{}:{}", resource.kind, resource.local_id))
                                ),
                            )
                            .header("content-type", "application/scim+json")
                            .json(&body)
                    })?;
                let (created, _) = scim_json(response)?;
                if created["externalId"] != external_id || !managed_equal(&body, &created) {
                    return Err(remote_error());
                }
                created["id"]
                    .as_str()
                    .filter(|id| !id.is_empty() && id.len() <= 512)
                    .ok_or_else(remote_error)?
                    .to_owned()
            };
            Ok(Some(Link {
                target: job.plan.target.clone(),
                url: target.url.clone(),
                kind: resource.kind.clone(),
                local_id: resource.local_id.clone(),
                remote_id,
                external_id,
                body,
            }))
        })();
        match result {
            Ok(link) => self.finish_provisioning(&mut job, link),
            Err(error) => self.store.write(|tx| {
                if let Some(mut current) = tx
                    .get::<Job>("provisioning_jobs", &job.plan.id)?
                    .filter(|j| j.lease == job.lease)
                {
                    current.lease = None;
                    current.error = Some(error.code.into());
                    current.next_attempt = now() + 2u64.pow(current.attempts.min(12)).min(3600);
                    tx.put("provisioning_jobs", &job.plan.id, &current)?;
                }
                Ok(())
            }),
        }
    }
    fn finish_provisioning(&self, job: &mut Job, link: Option<Link>) -> Result<()> {
        self.store.write(|tx|{
            let Some(mut current)=tx.get::<Job>("provisioning_jobs",&job.plan.id)?.filter(|j|j.lease==job.lease) else{return Ok(());};
            if let Some(link)=link {tx.put("provisioning_links",&link_key(&link.target,&link.kind,&link.local_id),&link)?;}
            current.cursor=(current.cursor+1).min(current.plan.resources.len());current.completed=current.cursor==current.plan.resources.len();current.lease=None;current.error=None;current.next_attempt=now();current.attempts=0;
            if current.plan.revision!=tx.get::<u64>("meta","revision")?.unwrap_or(0) || actor(tx,&current.plan.actor).and_then(|a|a.require("provisioner.sync",&format!("provisioner/{}",current.plan.target))).is_err() {
                current.stale=true;current.completed=false;current.error=Some("Source or authority changed during delivery; inspect partial results and replan".into());
            }
            compact_terminal_job(&mut current);
            tx.put("provisioning_jobs",&job.plan.id,&current)?;
            if current.completed {audit(tx,&current.plan.actor,"provisioner.complete",&current.plan.target)?;}
            Ok(())
        })
    }
}
fn validate_oauth(oauth: &Oauth) -> Result<()> {
    crate::config::validate_server_url(&oauth.token_url)
        .map_err(|_| Error::bad("SCIM OAuth token URL must be canonical HTTPS or HTTP loopback"))?;
    if oauth.client_id.is_empty()
        || oauth.client_id.len() > 256
        || !oauth.client_id.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(Error::bad(
            "SCIM OAuth client_id must be 1–256 ASCII graphic characters",
        ));
    }
    validate_oauth_parameter(oauth.scope.as_deref(), 1024, "scope")?;
    validate_oauth_parameter(oauth.audience.as_deref(), 512, "audience")?;
    match oauth.grant {
        OauthGrant::ClientCredentials => {
            if oauth.client_secret_file.is_none() || oauth.refresh_token_file.is_some() {
                return Err(Error::bad(
                    "client_credentials requires client_secret_file and does not use refresh_token_file",
                ));
            }
        }
        OauthGrant::RefreshToken => {
            if oauth.refresh_token_file.is_none() {
                return Err(Error::bad(
                    "refresh_token grant requires refresh_token_file; riAuth does not write that file",
                ));
            }
        }
    }
    Ok(())
}
fn validate_oauth_parameter(value: Option<&str>, limit: usize, name: &str) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() || value.len() > limit || value.chars().any(char::is_control) {
        return Err(Error::bad(format!(
            "SCIM OAuth {name} is empty, too long, or contains control characters"
        )));
    }
    Ok(())
}
fn http_client(ca_file: Option<&Path>) -> Result<reqwest::blocking::Client> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none());
    if let Some(path) = ca_file {
        builder = builder.add_root_certificate(
            reqwest::Certificate::from_pem(&std::fs::read(path).map_err(Error::internal)?)
                .map_err(Error::internal)?,
        );
    }
    builder.build().map_err(|_| remote_error())
}
fn read_secret_file(path: &Path) -> Result<Zeroizing<String>> {
    let secret = crate::config::read_private_secret(path, 4096)
        .map_err(|_| Error::bad("SCIM credential must be a private file of at most 4096 bytes"))?;
    let trimmed = secret.trim();
    if trimmed.is_empty()
        || trimmed.len() > 4096
        || !trimmed.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(Error::bad("Invalid SCIM target credential"));
    }
    Ok(Zeroizing::new(trimmed.to_owned()))
}
struct OauthSecrets {
    client_secret: Option<Zeroizing<String>>,
    refresh_token: Option<Zeroizing<String>>,
    fingerprint: String,
}
fn read_oauth_secrets(oauth: &Oauth) -> Result<OauthSecrets> {
    let client_secret = oauth
        .client_secret_file
        .as_deref()
        .map(read_secret_file)
        .transpose()?;
    let refresh_token = oauth
        .refresh_token_file
        .as_deref()
        .map(read_secret_file)
        .transpose()?;
    let mut material = Zeroizing::new(String::new());
    if let Some(secret) = &client_secret {
        material.push_str(secret);
    }
    material.push('\0');
    if let Some(refresh) = &refresh_token {
        material.push_str(refresh);
    }
    Ok(OauthSecrets {
        fingerprint: digest(&material),
        client_secret,
        refresh_token,
    })
}
struct Issued {
    token: Zeroizing<String>,
    expires_at: u64,
    fingerprint: String,
}
struct AttemptError {
    error: Error,
    retry: bool,
}
fn oauth_failure(message: &'static str) -> AttemptError {
    AttemptError {
        retry: false,
        error: Error::new(
            axum::http::StatusCode::BAD_GATEWAY,
            "provisioning_remote_error",
            message,
        ),
    }
}
fn request_token(target: &Target, oauth: &Oauth, secrets: &OauthSecrets) -> Result<Issued> {
    let http = target.token_http()?;
    match post_token(&http, oauth, secrets) {
        Ok(issued) => Ok(issued),
        Err(error) if error.retry => post_token(&http, oauth, secrets).map_err(|error| error.error),
        Err(error) => Err(error.error),
    }
}
fn post_token(
    http: &reqwest::blocking::Client,
    oauth: &Oauth,
    secrets: &OauthSecrets,
) -> std::result::Result<Issued, AttemptError> {
    let mut form = vec![
        ("grant_type".to_owned(), oauth.grant.as_str().to_owned()),
        ("client_id".to_owned(), oauth.client_id.clone()),
    ];
    if let Some(secret) = &secrets.client_secret {
        form.push(("client_secret".to_owned(), secret.as_str().to_owned()));
    }
    if let Some(refresh) = &secrets.refresh_token {
        form.push(("refresh_token".to_owned(), refresh.as_str().to_owned()));
    }
    if let Some(scope) = &oauth.scope {
        form.push(("scope".to_owned(), scope.clone()));
    }
    if let Some(audience) = &oauth.audience {
        form.push(("audience".to_owned(), audience.clone()));
    }
    let response = http
        .post(&oauth.token_url)
        .header("accept", "application/json")
        .form(&form)
        .send();
    for (_, value) in &mut form {
        value.zeroize();
    }
    let response = response.map_err(|_| AttemptError {
        retry: true,
        error: Error::new(
            axum::http::StatusCode::BAD_GATEWAY,
            "provisioning_remote_error",
            "SCIM OAuth token endpoint was unreachable",
        ),
    })?;
    let status = response.status();
    if !status.is_success() {
        discard_body(response);
        return Err(AttemptError {
            retry: status.is_server_error(),
            error: Error::new(
                axum::http::StatusCode::BAD_GATEWAY,
                "provisioning_remote_error",
                format!(
                    "SCIM OAuth token endpoint returned HTTP {}",
                    status.as_u16()
                ),
            ),
        });
    }
    if response
        .content_length()
        .is_some_and(|length| length > 65_536)
    {
        discard_body(response);
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned an invalid token",
        ));
    }
    let mut body = Zeroizing::new(Vec::new());
    response
        .take(65_537)
        .read_to_end(&mut body)
        .map_err(|_| oauth_failure("SCIM OAuth token endpoint returned an invalid token"))?;
    if body.len() > 65_536 {
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned an invalid token",
        ));
    }
    let mut parsed: TokenResponse = serde_json::from_slice(&body)
        .map_err(|_| oauth_failure("SCIM OAuth token endpoint returned an invalid token"))?;
    body.zeroize();
    let token_type = parsed.token_type.as_deref().unwrap_or("").trim();
    let mut raw = Zeroizing::new(parsed.access_token.take().unwrap_or_default());
    if !token_type.eq_ignore_ascii_case("bearer") {
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned a non-bearer token",
        ));
    }
    let access = Zeroizing::new(raw.trim().to_owned());
    raw.zeroize();
    if access.is_empty() {
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned an empty token",
        ));
    }
    if access.len() > 8_192 || !access.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned an invalid token",
        ));
    }
    enforce_token_restrictions(oauth, &access, &parsed)?;
    let lifetime = token_lifetime(parsed.expires_in.as_ref())?;
    // Reuse a token only until 30 seconds before expiry. A missing expires_in lasts 60 seconds.
    let expires_at = now().saturating_add(lifetime.saturating_sub(30));
    let fingerprint = digest(&access);
    Ok(Issued {
        token: access,
        expires_at,
        fingerprint,
    })
}
#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    #[serde(default)]
    expires_in: Option<Value>,
    #[serde(default)]
    scope: Option<Value>,
    #[serde(default)]
    audience: Option<Value>,
    #[serde(default)]
    aud: Option<Value>,
}
fn token_lifetime(value: Option<&Value>) -> std::result::Result<u64, AttemptError> {
    let Some(value) = value else {
        return Ok(60);
    };
    if let Some(seconds) = value.as_u64() {
        return Ok(seconds);
    }
    if let Some(seconds) = value.as_i64()
        && seconds >= 0
    {
        return Ok(seconds as u64);
    }
    if let Some(seconds) = value.as_f64()
        && seconds.is_finite()
        && seconds >= 0.0
        && seconds <= u64::MAX as f64
    {
        return Ok(seconds.floor() as u64);
    }
    Err(oauth_failure(
        "SCIM OAuth token endpoint returned an invalid token",
    ))
}
fn enforce_token_restrictions(
    oauth: &Oauth,
    token: &str,
    body: &TokenResponse,
) -> std::result::Result<(), AttemptError> {
    if let Some(required) = oauth.scope.as_deref() {
        if let Some(scope) = &body.scope
            && !scope_covers(scope, required)
        {
            return Err(oauth_failure(
                "SCIM OAuth token does not cover the configured scope",
            ));
        }
        if let Some(claims) = unverified_claims(token) {
            for key in ["scope", "scp"] {
                if let Some(scope) = claims.get(key)
                    && !scope_covers(scope, required)
                {
                    return Err(oauth_failure(
                        "SCIM OAuth token does not cover the configured scope",
                    ));
                }
            }
        }
    }
    if let Some(required) = oauth.audience.as_deref() {
        for audience in [body.audience.as_ref(), body.aud.as_ref()]
            .into_iter()
            .flatten()
        {
            if !audience_covers(audience, required) {
                return Err(oauth_failure(
                    "SCIM OAuth token does not cover the configured audience",
                ));
            }
        }
        if let Some(claims) = unverified_claims(token)
            && let Some(audience) = claims.get("aud")
            && !audience_covers(audience, required)
        {
            return Err(oauth_failure(
                "SCIM OAuth token does not cover the configured audience",
            ));
        }
    }
    Ok(())
}
fn scope_covers(granted: &Value, required: &str) -> bool {
    let granted = match granted {
        Value::String(text) => text.split_whitespace().collect::<BTreeSet<_>>(),
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>(),
        _ => return false,
    };
    required
        .split_whitespace()
        .all(|scope| granted.contains(scope))
}
fn audience_covers(audience: &Value, required: &str) -> bool {
    match audience {
        Value::String(value) => value == required,
        Value::Array(items) => items.iter().any(|item| item.as_str() == Some(required)),
        _ => false,
    }
}
fn unverified_claims(token: &str) -> Option<Value> {
    let mut parts = token.split('.');
    let (Some(header), Some(payload), Some(signature)) = (parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    if header.is_empty() || payload.is_empty() || signature.is_empty() || parts.next().is_some() {
        return None;
    }
    // Signature verification belongs to the SCIM resource server. This only reads scope/audience claims.
    jsonwebtoken::dangerous::insecure_decode::<Value>(token)
        .ok()
        .map(|data| data.claims)
        .filter(Value::is_object)
}
fn discard_body(response: reqwest::blocking::Response) {
    let mut ignored = Vec::new();
    let _ = response.take(65_537).read_to_end(&mut ignored);
    ignored.zeroize();
}
fn authorized(
    core: &Core,
    name: &str,
    target: &Target,
    http: &reqwest::blocking::Client,
    build: impl Fn(&reqwest::blocking::Client, &str) -> reqwest::blocking::RequestBuilder,
) -> Result<reqwest::blocking::Response> {
    let mut bearer = target.bearer(core, name)?;
    let mut response = build(http, bearer.as_str())
        .send()
        .map_err(|_| remote_error())?;
    if response.status().as_u16() == 401 {
        discard_body(response);
        if target.oauth.is_some() {
            invalidate_generation(
                name,
                &cache_key(name, &target.fingerprint()?),
                bearer.generation,
            );
        }
        bearer = target.bearer(core, name)?;
        response = build(http, bearer.as_str())
            .send()
            .map_err(|_| remote_error())?;
        if response.status().as_u16() == 401 {
            discard_body(response);
            return Err(remote_error());
        }
    }
    Ok(response)
}
#[derive(Serialize, Deserialize)]
struct OauthTokenRecord {
    expires_at: u64,
    fingerprint: String,
}
fn persist_oauth_meta(core: &Core, name: &str, expires_at: u64, fingerprint: &str) -> Result<()> {
    core.store.write(|tx| {
        tx.put(
            "scim_oauth_cache",
            name,
            &OauthTokenRecord {
                expires_at,
                fingerprint: fingerprint.to_owned(),
            },
        )
    })
}
struct Slot {
    generation: u64,
    secret_fingerprint: String,
    expires_at: u64,
    token: Option<Zeroizing<String>>,
}
fn token_cache() -> &'static Mutex<BTreeMap<String, Slot>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, Slot>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn target_locks() -> &'static Mutex<BTreeMap<String, Arc<Mutex<()>>>> {
    static LOCKS: OnceLock<Mutex<BTreeMap<String, Arc<Mutex<()>>>>> = OnceLock::new();
    LOCKS.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn mutex_guard<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
fn target_lock(name: &str) -> Arc<Mutex<()>> {
    let mut locks = mutex_guard(target_locks());
    locks
        .entry(name.to_owned())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}
fn cache_key(name: &str, config_fingerprint: &str) -> String {
    format!("{name}\0{config_fingerprint}")
}
fn cached_bearer(key: &str, secret_fingerprint: &str) -> Option<Bearer> {
    let cache = mutex_guard(token_cache());
    let slot = cache.get(key)?;
    if slot.secret_fingerprint != secret_fingerprint || slot.expires_at <= now() {
        return None;
    }
    slot.token.as_ref().map(|token| Bearer {
        generation: slot.generation,
        token: token.clone(),
    })
}
fn store_bearer(name: &str, key: &str, secret_fingerprint: &str, issued: &Issued) -> u64 {
    let mut cache = mutex_guard(token_cache());
    let generation = match cache.get(key) {
        Some(slot) if slot.token.is_none() => slot.generation.max(1),
        Some(slot) => slot.generation.saturating_add(1),
        None => 1,
    };
    let prefix = format!("{name}\0");
    cache.retain(|existing, _| existing == key || !existing.starts_with(&prefix));
    cache.insert(
        key.to_owned(),
        Slot {
            generation,
            secret_fingerprint: secret_fingerprint.to_owned(),
            expires_at: issued.expires_at,
            token: Some(issued.token.clone()),
        },
    );
    generation
}
fn forget_cache_key(key: &str) {
    mutex_guard(token_cache()).remove(key);
}
fn invalidate_generation(name: &str, key: &str, generation: u64) {
    let lock = target_lock(name);
    let _guard = mutex_guard(&lock);
    let mut cache = mutex_guard(token_cache());
    if let Some(slot) = cache.get_mut(key)
        && slot.generation == generation
        && slot.token.is_some()
    {
        slot.token = None;
        slot.expires_at = 0;
        slot.generation = slot.generation.saturating_add(1);
    }
}
fn remote_error() -> Error {
    Error::new(
        axum::http::StatusCode::BAD_GATEWAY,
        "provisioning_remote_error",
        "SCIM target rejected the request or returned an invalid response",
    )
}
fn scim_json(response: reqwest::blocking::Response) -> Result<(Value, Option<String>)> {
    if !response.status().is_success() || response.content_length().is_some_and(|n| n > 2_097_152) {
        return Err(remote_error());
    }
    let etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let mut bytes = Vec::new();
    response
        .take(2_097_153)
        .read_to_end(&mut bytes)
        .map_err(|_| remote_error())?;
    if bytes.len() > 2_097_152 {
        return Err(remote_error());
    }
    Ok((
        serde_json::from_slice(&bytes).map_err(|_| remote_error())?,
        etag,
    ))
}
pub async fn deliver(core: Core) -> Result<()> {
    if core.config.scim_targets.is_empty() {
        return Ok(());
    }
    tokio::task::spawn_blocking(move || core.provisioning_step())
        .await
        .map_err(Error::internal)?
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, plan) in tx.maintenance_page::<Plan>("provisioning_plans")? {
        if plan.expires_at + 7 * 86400 < at {
            tx.delete("provisioning_plans", &id)?;
        }
    }
    for (id, mut job) in tx.maintenance_page::<Job>("provisioning_jobs")? {
        if (job.completed || job.stale) && job.plan.expires_at + 7 * 86400 < at {
            tx.delete("provisioning_jobs", &id)?;
        } else if (job.completed || job.stale) && !job.plan.resources.is_empty() {
            compact_terminal_job(&mut job);
            tx.put("provisioning_jobs", &id, &job)?;
        }
    }
    Ok(())
}

fn managed_equal(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => expected
            .iter()
            .all(|(key, value)| managed_member_equal(key, value, actual.get(key))),
        (Value::Array(expected), Value::Array(actual)) => {
            expected.len() == actual.len()
                && expected
                    .iter()
                    .all(|value| actual.iter().any(|actual| managed_equal(value, actual)))
        }
        _ => expected == actual,
    }
}

fn managed_member_equal(key: &str, expected: &Value, actual: Option<&Value>) -> bool {
    if ["members", "emails"].contains(&key)
        && expected.as_array().is_some_and(Vec::is_empty)
        && actual.is_none_or(Value::is_null)
    {
        return true;
    }
    actual.is_some_and(|actual| managed_equal(expected, actual))
}

#[cfg(test)]
mod tests {
    use super::managed_equal;
    use serde_json::json;

    #[test]
    fn scim_omitted_empty_multi_value_fields_are_equal() {
        assert!(managed_equal(
            &json!({"id":"group-1","members":[]}),
            &json!({"id":"group-1"})
        ));
        assert!(managed_equal(
            &json!({"id":"user-1","emails":[]}),
            &json!({"id":"user-1","emails":null})
        ));
        assert!(!managed_equal(
            &json!({"id":"group-1","members":[{"value":"user-1"}]}),
            &json!({"id":"group-1"})
        ));
    }
}
