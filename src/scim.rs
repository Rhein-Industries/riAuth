//! SCIM 2.0 user/group provisioning. Each authenticated operator owns its provisioned records.
use crate::{
    agent::Principal,
    core::{Core, audit, make_user, user_by_name, validate_display, validate_email, validate_name},
    crypto,
    error::{Error, Result},
    model::{Group, NewUser, User},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub use crate::scim_shared::{GROUP, USER, response};
const LIST: &str = "urn:ietf:params:scim:api:messages:2.0:ListResponse";
const PATCH: &str = "urn:ietf:params:scim:api:messages:2.0:PatchOp";

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Record {
    owner: String,
    kind: String,
    local_id: String,
    external_id: Option<String>,
    data: Value,
    deleted: bool,
    /// A generation marker prevents a resource from reusing an earlier ETag
    /// after SCIM changes are later reversed. Legacy rows use an empty marker.
    #[serde(default)]
    version: String,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub start_index: Option<usize>,
    pub count: Option<usize>,
    pub filter: Option<String>,
}

pub fn metadata(kind: &str) -> Result<Value> {
    Ok(match kind {
        "ServiceProviderConfig" => {
            json!({"schemas":["urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig"],"patch":{"supported":true},"bulk":{"supported":false,"maxOperations":0,"maxPayloadSize":0},"filter":{"supported":true,"maxResults":1000},"changePassword":{"supported":true},"sort":{"supported":false},"etag":{"supported":true},"authenticationSchemes":[{"type":"oauthbearertoken","name":"Scoped operator credential","description":"Use a dedicated agent credential and resource If-Match for updates and deletes","primary":true}]})
        }
        "ResourceTypes" => {
            json!({"schemas":[LIST],"totalResults":2,"startIndex":1,"itemsPerPage":2,"Resources":[{"schemas":["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],"id":"User","name":"User","endpoint":"/Users","schema":USER},{"schemas":["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],"id":"Group","name":"Group","endpoint":"/Groups","schema":GROUP}]})
        }
        "Schemas" => {
            json!({"schemas":[LIST],"totalResults":2,"startIndex":1,"itemsPerPage":2,"Resources":[schema(USER),schema(GROUP)]})
        }
        USER | GROUP => schema(kind),
        _ => return Err(Error::missing("Unknown SCIM metadata resource")),
    })
}
fn schema(id: &str) -> Value {
    let mut attributes = vec![attribute("externalId", "string", false, false, "readWrite")];
    if id == USER {
        attributes.extend([
            attribute("userName", "string", false, true, "readWrite"),
            attribute("displayName", "string", false, false, "readWrite"),
            attribute("active", "boolean", false, false, "readWrite"),
            attribute("password", "string", false, false, "writeOnly"),
            attribute("name", "complex", false, false, "readWrite"),
            attribute("emails", "complex", true, false, "readWrite"),
            attribute("groups", "complex", true, false, "readOnly"),
        ]);
    } else {
        attributes.extend([
            attribute("displayName", "string", false, true, "readWrite"),
            attribute("members", "complex", true, false, "readWrite"),
        ]);
    }
    for a in &mut attributes {
        match a["name"].as_str().unwrap_or("") {
            "name" => {
                a["subAttributes"] = json!([
                    attribute("formatted", "string", false, false, "readWrite"),
                    attribute("givenName", "string", false, false, "readWrite"),
                    attribute("familyName", "string", false, false, "readWrite")
                ])
            }
            "emails" => {
                a["subAttributes"] = json!([
                    attribute("value", "string", false, true, "readWrite"),
                    attribute("type", "string", false, false, "readWrite"),
                    attribute("primary", "boolean", false, false, "readWrite")
                ])
            }
            "members" | "groups" => {
                a["subAttributes"] = json!([
                    attribute("value", "string", false, true, "readWrite"),
                    attribute("display", "string", false, false, "readOnly")
                ])
            }
            _ => {}
        }
    }
    json!({"schemas":["urn:ietf:params:scim:schemas:core:2.0:Schema"],"id":id,"name":if id==USER{"User"}else{"Group"},"attributes":attributes})
}
fn attribute(name: &str, kind: &str, multi: bool, required: bool, mutability: &str) -> Value {
    json!({"name":name,"type":kind,"multiValued":multi,"required":required,"caseExact":false,"mutability":mutability,"returned":if mutability=="writeOnly"{"never"}else{"default"},"uniqueness":if name=="userName"{"server"}else{"none"}})
}

fn bucket(kind: &str) -> Result<&'static str> {
    match kind {
        "Users" => Ok("scim_users"),
        "Groups" => Ok("scim_groups"),
        _ => Err(Error::missing("SCIM resource type not found")),
    }
}
fn scope(kind: &str) -> &'static str {
    if kind == "Users" { "user" } else { "group" }
}
fn owned(tx: &Tx<'_>, actor: &Principal, kind: &str, id: &str) -> Result<Record> {
    tx.get::<Record>(bucket(kind)?, id)?
        .filter(|r| !r.deleted && r.owner == actor.id)
        .ok_or_else(|| Error::missing("SCIM resource not found"))
}

/// Track effective changes made outside inbound SCIM as part of the same
/// storage transaction. This keeps an old ETag stale even when a later write
/// restores the same projected content. Server assembly calls this only in
/// Platform builds; Essentials has no inbound SCIM transition work.
pub(crate) fn record_transition(
    tx: &Tx<'_>,
    bucket: &str,
    key: &str,
    before: Option<&Value>,
    after: Option<&Value>,
) -> Result<()> {
    match bucket {
        "users" => {
            let effective = ["username", "display_name", "email", "enabled", "password_hash"];
            if !effective.iter().any(|field| before.and_then(|v| v.get(*field)) != after.and_then(|v| v.get(*field))) {
                return Ok(());
            }
            for (id, mut record) in tx.list::<Record>("scim_users")? {
                if !record.deleted && record.local_id == key {
                    record.version = crypto::id();
                    tx.put("scim_users", &id, &record)?;
                }
            }
        }
        "groups" => {
            let members = |value: Option<&Value>| -> Result<BTreeSet<String>> {
                value
                    .map(|value| serde_json::from_value::<Group>(value.clone()).map(|group| group.members).map_err(Error::internal))
                    .transpose()
                    .map(|members| members.unwrap_or_default())
            };
            let old = members(before)?;
            let new = members(after)?;
            if old == new {
                return Ok(());
            }
            let users = tx.list::<Record>("scim_users")?;
            for (id, mut group) in tx.list::<Record>("scim_groups")? {
                if group.deleted || group.local_id != key {
                    continue;
                }
                let mut visible_change = false;
                for (user_id, mut user) in users.iter().cloned() {
                    if user.deleted || user.owner != group.owner {
                        continue;
                    }
                    if old.contains(&user.local_id) != new.contains(&user.local_id) {
                        visible_change = true;
                        user.version = crypto::id();
                        tx.put("scim_users", &user_id, &user)?;
                    }
                }
                if visible_change {
                    group.version = crypto::id();
                    tx.put("scim_groups", &id, &group)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
fn name(record: &Record) -> &str {
    record.data[if record.kind == "Users" {
        "userName"
    } else {
        "displayName"
    }]
    .as_str()
    .unwrap_or("")
}
fn require(actor: &Principal, record: &Record, action: &str) -> Result<()> {
    let kind = scope(&record.kind);
    actor.require(
        &format!("{kind}.{action}"),
        &format!("{kind}/{}", name(record)),
    )
}
fn owner_scoped_group_members(
    tx: &Tx<'_>,
    actor: &Principal,
    group_name: &str,
    mut requested: BTreeSet<String>,
) -> Result<BTreeSet<String>> {
    let current = tx
        .get::<Group>("groups", group_name)?
        .ok_or_else(|| Error::missing("Group not found"))?;
    // Group edges have no source tag. Live SCIM User records owned by this
    // operator are the ownership boundary; preserve all other members.
    let owned_members: BTreeSet<_> = tx
        .list::<Record>("scim_users")?
        .into_iter()
        .filter(|(_, user)| !user.deleted && user.owner == actor.id)
        .map(|(_, user)| user.local_id)
        .collect();
    requested.extend(current.members.difference(&owned_members).cloned());
    Ok(requested)
}
fn normalize(mut value: Value) -> Result<Value> {
    let map = value
        .as_object_mut()
        .ok_or_else(|| Error::bad("SCIM resource must be an object"))?;
    let mut normalized = serde_json::Map::new();
    for (k, v) in std::mem::take(map) {
        let canonical = match k.to_ascii_lowercase().as_str() {
            "username" => "userName",
            "displayname" => "displayName",
            "externalid" => "externalId",
            "schemas" => "schemas",
            "name" => "name",
            "active" => "active",
            "emails" => "emails",
            "password" => "password",
            "members" => "members",
            "id" => "id",
            "meta" => "meta",
            "groups" => "groups",
            _ => return Err(Error::bad("Unsupported SCIM attribute")),
        }
        .to_owned();
        if normalized.insert(canonical, v).is_some() {
            return Err(Error::bad("Duplicate case-insensitive SCIM attribute"));
        }
    }
    Ok(Value::Object(normalized))
}
enum Filter {
    Text {
        field: &'static str,
        needle: String,
        exact: bool,
    },
    Email(String),
    Active(bool),
}

impl Filter {
    fn matches(&self, value: &Value) -> bool {
        match self {
            Self::Text { field, needle, exact } => value[*field].as_str().is_some_and(|actual| {
                if *exact {
                    actual == needle
                } else {
                    actual.eq_ignore_ascii_case(needle)
                }
            }),
            Self::Email(needle) => value["emails"].as_array().is_some_and(|emails| {
                emails.iter().any(|email| {
                    email["value"]
                        .as_str()
                        .is_some_and(|actual| actual.eq_ignore_ascii_case(needle))
                })
            }),
            Self::Active(expected) => value["active"].as_bool() == Some(*expected),
        }
    }
}

fn parse_filter(kind: &str, filter: Option<&str>) -> Result<Option<Filter>> {
    let Some(filter) = filter else {
        return Ok(None);
    };
    if filter.len() > 1024 {
        return Err(Error::oauth("invalid_filter", "Filter too long"));
    }
    if filter.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(Error::oauth("invalid_filter", "Control characters are not allowed in filters"));
    }
    // One attribute, one `eq`, and one literal. Only ASCII spaces may separate
    // tokens; serde_json enforces complete JSON string/boolean literal parsing.
    let filter = filter.trim_matches(' ');
    let (field, rest) = filter
        .split_once(' ')
        .ok_or_else(|| Error::oauth("invalid_filter", "Expected attribute eq value"))?;
    let (op, literal) = rest
        .trim_start_matches(' ')
        .split_once(' ')
        .ok_or_else(|| Error::oauth("invalid_filter", "Expected attribute eq value"))?;
    let literal = literal.trim_matches(' ');
    if !op.eq_ignore_ascii_case("eq") || literal.is_empty() {
        return Err(Error::oauth("invalid_filter", "Only one eq expression is supported"));
    }
    let field = field.to_ascii_lowercase();
    if field == "active" {
        if kind != "Users" {
            return Err(Error::oauth("invalid_filter", "active is a User filter"));
        }
        let active = serde_json::from_str::<bool>(literal)
            .map_err(|_| Error::oauth("invalid_filter", "active requires true or false"))?;
        return Ok(Some(Filter::Active(active)));
    }
    let needle = serde_json::from_str::<String>(literal)
        .map_err(|_| Error::oauth("invalid_filter", "Filter value must be one quoted JSON string"))?;
    let parsed = match field.as_str() {
        "username" if kind == "Users" => Filter::Text {
            field: "userName", needle, exact: false,
        },
        "displayname" => Filter::Text {
            field: "displayName", needle, exact: false,
        },
        "externalid" => Filter::Text {
            field: "externalId", needle, exact: true,
        },
        "id" => Filter::Text {
            field: "id", needle, exact: true,
        },
        "emails.value" if kind == "Users" => Filter::Email(needle),
        _ => return Err(Error::oauth(
            "invalid_filter",
            "Supported fields: displayName, externalId, id, and User userName, emails.value, active",
        )),
    };
    Ok(Some(parsed))
}

impl Core {
    fn scim_view(&self, tx: &Tx<'_>, id: &str, record: &Record) -> Result<Value> {
        let mut value = record.data.clone();
        value["id"] = json!(id);
        value.as_object_mut().unwrap().remove("password");
        if record.kind == "Users" {
            let user = tx
                .get::<User>("users", &record.local_id)?
                .ok_or_else(|| Error::missing("User missing"))?;
            value["active"] = json!(user.enabled);
            value["displayName"] = json!(user.display_name);
            let stored_email = read_email(&value)?;
            if stored_email != user.email {
                value["emails"] = json!(
                    user.email
                        .iter()
                        .map(|email| json!({"value":email,"primary":true}))
                        .collect::<Vec<_>>()
                );
            }
            let groups = crate::core::durable_groups_for(tx, &user.id)?;
            value["groups"] = json!(
                tx.list::<Record>("scim_groups")?
                    .into_iter()
                    .filter(|(_, g)| !g.deleted
                        && g.owner == record.owner
                        && groups.contains(&g.local_id))
                    .map(|(id, g)| json!({"value":id,"display":name(&g)}))
                    .collect::<Vec<_>>()
            );
        } else {
            let group = tx
                .get::<Group>("groups", &record.local_id)?
                .ok_or_else(|| Error::missing("Group missing"))?;
            value["members"] = json!(
                tx.list::<Record>("scim_users")?
                    .into_iter()
                    .filter(|(_, u)| !u.deleted
                        && u.owner == record.owner
                        && group.members.contains(&u.local_id))
                    .map(|(id, u)| json!({"value":id,"display":name(&u)}))
                    .collect::<Vec<_>>()
            );
        }
        // Include the effective projection so direct management and directory
        // changes to this resource also invalidate its ETag. Hidden password
        // changes affect the user version without revealing the password hash.
        let credential = if record.kind == "Users" {
            tx.get::<User>("users", &record.local_id)?
                .map(|user| user.password_hash)
        } else {
            None
        };
        let fingerprint = json!([record.version, value, credential]);
        let serialized = serde_json::to_string(&fingerprint).map_err(Error::internal)?;
        let version = format!("\"{}\"", crypto::digest(&serialized));
        value["meta"] = json!({"resourceType":if record.kind=="Users"{"User"}else{"Group"},"location":format!("{}/scim/v2/{}/{id}",self.config.issuer.trim_end_matches('/'),record.kind),"version":version});
        Ok(value)
    }
    fn scim_precondition(
        &self,
        tx: &Tx<'_>,
        actor: &Principal,
        context: Option<&crate::context::RequestContext>,
        kind: &str,
        id: Option<&str>,
    ) -> Result<()> {
        let Some(id) = id else {
            if context.and_then(|c| c.if_match.as_ref()).is_some() {
                return Err(Error::new(StatusCode::PRECONDITION_FAILED, "precondition_failed", "No resource exists for If-Match"));
            }
            return Ok(());
        };
        let record = owned(tx, actor, kind, id)?;
        require(actor, &record, "write")?;
        if let Some(context) = context {
            let supplied = context.if_match.as_deref();
            if actor.agent && supplied.is_none() {
                return Err(Error::new(StatusCode::PRECONDITION_REQUIRED, "precondition_required", "Agent SCIM updates and deletes require resource If-Match"));
            }
            if let Some(supplied) = supplied {
                let current = self.scim_view(tx, id, &record)?;
                let version = current["meta"]["version"].as_str().unwrap_or("");
                if !crypto::constant_eq(supplied, version) {
                    return Err(Error::new(StatusCode::PRECONDITION_FAILED, "precondition_failed", "SCIM resource version changed"));
                }
            }
        }
        Ok(())
    }
    pub fn scim_get(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let record = owned(tx, &actor, kind, id)?;
            require(&actor, &record, "read")?;
            self.scim_view(tx, id, &record)
        })
    }
    pub fn scim_list(&self, token: &str, kind: &str, query: Query) -> Result<Value> {
        self.store.read(|tx|{
        let actor=self.principal(tx,token)?;
        bucket(kind)?;
        // Parse once, including when the collection is empty.
        let filter=parse_filter(kind,query.filter.as_deref())?;
        let start=query.start_index.unwrap_or(1).max(1);let count=query.count.unwrap_or(100).min(1000);
        let mut values=Vec::new();
        for (id,record) in tx.list::<Record>(bucket(kind)?)? {
            if record.deleted || record.owner!=actor.id || require(&actor,&record,"read").is_err(){continue;}
            let value=self.scim_view(tx,&id,&record)?;
            if filter.as_ref().is_none_or(|filter| filter.matches(&value)){values.push(value);}
        }
        let total=values.len();let page:Vec<_>=values.into_iter().skip(start-1).take(count).collect();
        Ok(json!({"schemas":[LIST],"totalResults":total,"startIndex":start,"itemsPerPage":page.len(),"Resources":page}))
    })
    }
    pub fn scim_write(
        &self,
        token: &str,
        kind: &str,
        id: Option<&str>,
        input: Value,
        patch: bool,
    ) -> Result<Value> {
        bucket(kind)?;
        self.mutation_checked(token, |tx, actor, context| self.scim_precondition(tx, actor, context, kind, id), |tx| {
            let actor = self.principal(tx, token)?;
            let existing = id.map(|id| owned(tx, &actor, kind, id)).transpose()?;
            let before_version = existing.as_ref().map(|record| {
                self.scim_view(tx, id.unwrap(), record)
                    .map(|value| value["meta"]["version"].clone())
            }).transpose()?;
            let member_patch = kind == "Groups" && patch && patches_members(&input);
            let mut data = if patch {
                let mut base = existing
                    .as_ref()
                    .ok_or_else(|| Error::bad("Patch requires a resource"))?
                    .data
                    .clone();
                if member_patch {
                    // A PATCH starts from live SCIM-visible membership. Stored
                    // metadata may lag direct or directory group changes.
                    base["members"] =
                        self.scim_view(tx, id.unwrap(), existing.as_ref().unwrap())?["members"]
                            .clone();
                }
                patch_resource(&base, input)?
            } else {
                normalize(input)?
            };
            let schema = if kind == "Users" { USER } else { GROUP };
            if !data["schemas"]
                .as_array()
                .is_some_and(|a| a.len() == 1 && a[0] == schema)
            {
                return Err(Error::bad("Unsupported or missing SCIM resource schema"));
            }
            let name_field = if kind == "Users" {
                "userName"
            } else {
                "displayName"
            };
            let label = data[name_field]
                .as_str()
                .ok_or_else(|| Error::bad("Required SCIM name missing"))?
                .to_owned();
            validate_name(&label)?;
            let resource = format!("{}/{}", scope(kind), label);
            actor.require(&format!("{}.write", scope(kind)), &resource)?;
            if existing.as_ref().is_some_and(|r| name(r) != label) {
                return Err(Error::oauth(
                    "mutability",
                    "Resource names are immutable; create another identity",
                ));
            }
            let external_id = data
                .get("externalId")
                .map(|v| {
                    v.as_str()
                        .filter(|s| !s.is_empty() && s.len() <= 256)
                        .map(String::from)
                        .ok_or_else(|| Error::bad("Invalid externalId"))
                })
                .transpose()?;
            if let Some(external) = &external_id
                && tx.list::<Record>(bucket(kind)?)?.iter().any(|(other, r)| {
                    !r.deleted
                        && r.owner == actor.id
                        && r.external_id.as_ref() == Some(external)
                        && Some(other.as_str()) != id
                })
            {
                return Err(Error::conflict(
                    "externalId already belongs to another resource",
                ));
            }
            let id = id.map(String::from).unwrap_or_else(crypto::id);
            let mut group_changed = false;
            let local_id = if kind == "Users" {
                if data.get("members").is_some() {
                    return Err(Error::bad("Users cannot supply group members"));
                }
                let mut user = if let Some(old) = &existing {
                    tx.get::<User>("users", &old.local_id)?
                        .ok_or_else(|| Error::missing("User missing"))?
                } else {
                    if tx.get::<String>("usernames", &label)?.is_some()
                        || tx
                            .list::<User>("users")?
                            .iter()
                            .any(|(_, u)| u.username.eq_ignore_ascii_case(&label))
                    {
                        return Err(Error::conflict(
                            "Username already exists; provisioning cannot take ownership",
                        ));
                    }
                    let mut u = make_user(NewUser {
                        username: label.clone(),
                        password: crypto::random_token(""),
                        email: None,
                        display_name: label.clone(),
                        admin: false,
                    })?;
                    u.password_hash.clear();
                    u
                };
                if user.admin {
                    return Err(Error::forbidden());
                }
                let enabled = data
                    .get("active")
                    .map(|v| {
                        v.as_bool()
                            .ok_or_else(|| Error::bad("active must be a boolean"))
                    })
                    .transpose()?
                    .unwrap_or(true);
                let display = data["displayName"]
                    .as_str()
                    .or(data["name"]["formatted"].as_str())
                    .unwrap_or(&label)
                    .to_owned();
                validate_display(&display)?;
                let email = read_email(&data)?;
                if user.email != email {
                    user.email_verified = false;
                }
                user.email = email;
                user.display_name = display;
                let password = data.as_object_mut().unwrap().remove("password");
                let revoke = user.enabled != enabled || password.is_some();
                user.enabled = enabled;
                if let Some(password) = password {
                    let plaintext = password
                        .as_str()
                        .ok_or_else(|| Error::bad("password must be a string"))?;
                    let hashed = crypto::password_hash(plaintext)?;
                    crate::identity::password_history::accept(
                        tx,
                        self.config.password_history,
                        &user.id,
                        &user.password_hash,
                        plaintext,
                        &hashed,
                    )?;
                    user.password_hash = hashed;
                    tx.delete("credential_versions", &format!("user/{label}"))?;
                }
                if revoke {
                    user.epoch += 1;
                    crate::logout::queue_user(tx, &user.id)?;
                }
                tx.put("users", &user.id, &user)?;
                tx.put("usernames", &label, &user.id)?;
                data.as_object_mut().unwrap().remove("groups");
                user.id
            } else {
                if data.as_object().unwrap().keys().any(|k| {
                    ![
                        "schemas",
                        "id",
                        "meta",
                        "externalId",
                        "displayName",
                        "members",
                    ]
                    .contains(&k.as_str())
                }) {
                    return Err(Error::bad("Unsupported group attribute"));
                }
                actor.require("group.members", &resource)?;
                if existing.is_none() && tx.get::<Group>("groups", &label)?.is_some() {
                    return Err(Error::conflict(
                        "Group already exists; provisioning cannot take ownership",
                    ));
                }
                if !patch || member_patch {
                    let mut members = BTreeSet::new();
                    let mut public = Vec::new();
                    let array = data
                        .get("members")
                        .map(|v| {
                            v.as_array()
                                .ok_or_else(|| Error::bad("members must be an array"))
                        })
                        .transpose()?;
                    for member in array.into_iter().flatten() {
                        let id = member["value"]
                            .as_str()
                            .ok_or_else(|| Error::bad("Member ID required"))?;
                        let member = owned(tx, &actor, "Users", id)?;
                        if !members.insert(member.local_id.clone()) {
                            return Err(Error::bad("Duplicate group member"));
                        }
                        public.push(json!({"value":id,"display":name(&member)}));
                    }
                    if existing.is_some() {
                        members = owner_scoped_group_members(tx, &actor, &label, members)?;
                    }
                    if members.len() > 1000 {
                        return Err(Error::bad("Too many group members"));
                    }
                    data["members"] = json!(public);
                    let intent = if existing.is_none() {
                        crate::management::GroupIntent::Create(&members)
                    } else {
                        crate::management::GroupIntent::ReplaceMembers(&members)
                    };
                    group_changed = crate::management::write_group(
                        tx,
                        &actor,
                        &label,
                        intent,
                        crate::management::GroupAudit::Deferred,
                    )?
                    .changed;
                }
                label.clone()
            };
            data.as_object_mut().unwrap().remove("meta");
            data.as_object_mut().unwrap().remove("id");
            let mut record = Record {
                owner: actor.id.clone(),
                kind: kind.into(),
                local_id,
                external_id,
                data,
                deleted: false,
                version: existing.as_ref().map_or_else(crypto::id, |record| record.version.clone()),
            };
            let record_changed = existing.as_ref() != Some(&record);
            if kind == "Users" || record_changed {
                tx.put(bucket(kind)?, &id, &record)?;
            }
            let mut view = self.scim_view(tx, &id, &record)?;
            if before_version.is_some_and(|before| before != view["meta"]["version"]) {
                record.version = crypto::id();
                tx.put(bucket(kind)?, &id, &record)?;
                view = self.scim_view(tx, &id, &record)?;
            }
            if kind == "Users" || record_changed || group_changed {
                audit(tx, &actor.id, &format!("{}.scim", scope(kind)), &label)?;
            }
            Ok(view)
        })
    }
    pub fn scim_delete(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        self.mutation_checked(token, |tx, actor, context| self.scim_precondition(tx, actor, context, kind, Some(id)), |tx| {
            let actor = self.principal(tx, token)?;
            let mut record = owned(tx, &actor, kind, id)?;
            require(&actor, &record, "write")?;
            if kind == "Users" {
                let mut user = user_by_name(tx, name(&record))?;
                if user.id != record.local_id || user.username != name(&record) {
                    return Err(Error::conflict("SCIM user identity does not match"));
                }
                if user.admin {
                    return Err(Error::forbidden());
                }
                user.enabled = false;
                user.epoch += 1;
                tx.put("users", &user.id, &user)?;
                crate::logout::queue_user(tx, &user.id)?;
                for (group_id, mut group) in tx.list::<Record>("scim_groups")? {
                    if !group.deleted && group.owner == actor.id {
                        if let Some(members) = group.data["members"].as_array_mut() {
                            members.retain(|m| m["value"] != id);
                        }
                        tx.put("scim_groups", &group_id, &group)?;
                    }
                }
                for (key, group) in tx.list::<Group>("groups")? {
                    if group.members.contains(&user.id) {
                        crate::management::write_group(
                            tx,
                            &actor,
                            &key,
                            crate::management::GroupIntent::OffboardMember {
                                user_id: &user.id,
                                username: &user.username,
                            },
                            crate::management::GroupAudit::Deferred,
                        )?;
                    }
                }
            } else {
                require(&actor, &record, "members")?;
                let members =
                    owner_scoped_group_members(tx, &actor, &record.local_id, BTreeSet::new())?;
                crate::management::write_group(
                    tx,
                    &actor,
                    &record.local_id,
                    crate::management::GroupIntent::ReplaceMembers(&members),
                    crate::management::GroupAudit::Deferred,
                )?;
            }
            record.deleted = true;
            tx.put(bucket(kind)?, id, &record)?;
            audit(
                tx,
                &actor.id,
                &format!("{}.scim_delete", scope(kind)),
                name(&record),
            )?;
            Ok(json!({}))
        })
    }
}
fn read_email(data: &Value) -> Result<Option<String>> {
    let Some(emails) = data.get("emails") else {
        return Ok(None);
    };
    let emails = emails
        .as_array()
        .filter(|v| v.len() <= 8)
        .ok_or_else(|| Error::bad("emails must be an array of at most eight values"))?;
    let mut primary = None;
    let mut first = None;
    for email in emails {
        let value = email["value"]
            .as_str()
            .ok_or_else(|| Error::bad("Email value missing"))?;
        validate_email(value)?;
        if first.is_none() {
            first = Some(value.to_owned());
        }
        if email["primary"].as_bool() == Some(true) {
            if primary.is_some() {
                return Err(Error::bad("At most one primary email is allowed"));
            }
            primary = Some(value.to_owned());
        }
    }
    Ok(primary.or(first))
}
fn patches_members(input: &Value) -> bool {
    input["Operations"].as_array().is_some_and(|operations| {
        operations.iter().any(|op| {
            op["path"].as_str().is_some_and(|path| {
                path.eq_ignore_ascii_case("members") || path.starts_with("members[")
            }) || op["path"].is_null()
                && op["value"].as_object().is_some_and(|value| {
                    value.keys().any(|key| key.eq_ignore_ascii_case("members"))
                })
        })
    })
}

fn patch_resource(old: &Value, input: Value) -> Result<Value> {
    if input["schemas"] != json!([PATCH]) {
        return Err(Error::bad("Invalid SCIM patch schema"));
    }
    let operations = input["Operations"]
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= 100)
        .ok_or_else(|| Error::bad("Patch needs 1–100 operations"))?;
    let mut data = old.clone();
    for op in operations {
        let operation = op["op"].as_str().unwrap_or("").to_ascii_lowercase();
        if !["add", "replace", "remove"].contains(&operation.as_str()) {
            return Err(Error::bad("Unsupported patch operation"));
        }
        if op["path"].is_null() {
            if operation == "remove" {
                return Err(Error::bad("remove requires a path"));
            }
            let value = normalize(op["value"].clone())?;
            for (key, value) in value.as_object().unwrap() {
                if ["id", "meta", "schemas", "groups"].contains(&key.as_str()) {
                    return Err(Error::oauth(
                        "mutability",
                        "Cannot patch read-only attributes",
                    ));
                }
                data[key] = value.clone();
            }
            continue;
        }
        let path = op["path"]
            .as_str()
            .ok_or_else(|| Error::bad("Invalid patch path"))?;
        if let Some(filter) = path
            .strip_prefix("members[")
            .and_then(|p| p.strip_suffix(']'))
        {
            if operation != "remove" {
                return Err(Error::bad("Filtered members paths support remove"));
            }
            let needle = filter
                .strip_prefix("value eq ")
                .ok_or_else(|| Error::bad("Invalid member value filter"))?;
            let id: String =
                serde_json::from_str(needle).map_err(|_| Error::bad("Invalid member ID filter"))?;
            if let Some(members) = data["members"].as_array_mut() {
                members.retain(|m| m["value"] != id);
            }
            continue;
        }
        let canonical = normalize(json!({path:op["value"]}))?;
        let (path, value) = canonical.as_object().unwrap().iter().next().unwrap();
        if ["id", "meta", "schemas", "groups", "userName"].contains(&path.as_str()) {
            return Err(Error::oauth("mutability", "Cannot patch this attribute"));
        }
        if operation == "remove" {
            data.as_object_mut().unwrap().remove(path);
        } else if operation == "add" && path == "members" {
            let members = value
                .as_array()
                .ok_or_else(|| Error::bad("members must be an array"))?;
            if data.get(path).is_none() {
                data[path] = json!([]);
            }
            let current = data[path]
                .as_array_mut()
                .ok_or_else(|| Error::bad("members must be an array"))?;
            for m in members {
                if !current.iter().any(|old| old["value"] == m["value"]) {
                    current.push(m.clone());
                }
            }
        } else {
            data[path] = value.clone();
        }
    }
    Ok(data)
}

// Exercise pure parsers with a valid starting resource; never open a store or
// grant authority from a fuzz input. src/fuzzing.rs bounds bytes/depth/nodes.
#[cfg(feature = "fuzzing")]
pub(crate) fn fuzz_resource(input: Value) {
    let _ = normalize(input.clone()).and_then(|value| read_email(&value));
    let old =
        json!({"schemas":[GROUP],"displayName":"fuzz-group","members":[{"value":"fuzz-user"}]});
    let _ = patch_resource(&old, input);
}

#[cfg(feature = "fuzzing")]
pub(crate) fn fuzz_filter(filter: &str) {
    let _ = parse_filter("Users", Some(filter)).map(|parsed| {
        parsed.is_none_or(|parsed| parsed.matches(&json!({"userName":"fuzz-user","id":"fuzz-id"})))
    });
}

#[cfg(test)]
mod patch_tests {
    use super::*;

    fn patch(operations: Value) -> Value {
        json!({"schemas":[PATCH],"Operations":operations})
    }

    #[test]
    fn members_add_rejects_invalid_intermediate_shapes() {
        let old = json!({"schemas":[GROUP],"displayName":"test-group","members":[]});
        let before = old.clone();
        for shape in [
            json!(0),
            Value::Null,
            json!(true),
            json!("member"),
            json!({}),
        ] {
            let input = patch(json!([
                {"op":"replace","path":"members","value":shape},
                {"op":"add","path":"members","value":[]}
            ]));
            let error = patch_resource(&old, input).unwrap_err();
            assert_eq!(error.status, StatusCode::BAD_REQUEST, "{shape}");
            assert_eq!(error.message, "members must be an array", "{shape}");
            assert_eq!(old, before);
        }
    }

    #[test]
    fn members_add_preserves_absent_initialization_and_valid_sequences() {
        for old in [
            json!({"schemas":[GROUP],"displayName":"test-group"}),
            json!({"schemas":[GROUP],"displayName":"test-group","members":[{"value":"original"}]}),
        ] {
            let before = old.clone();
            let added = patch_resource(
                &old,
                patch(json!([
                    {"op":"add","path":"members","value":[{"value":"first"}]}
                ])),
            )
            .unwrap();
            let mut expected = old.get("members").cloned().unwrap_or_else(|| json!([]));
            expected
                .as_array_mut()
                .unwrap()
                .push(json!({"value":"first"}));
            assert_eq!(added["members"], expected);
            let updated = patch_resource(
                &added,
                patch(json!([
                    {"op":"replace","path":"members","value":[{"value":"third"}]},
                    {"op":"add","path":"members","value":[{"value":"second"},{"value":"third"}]}
                ])),
            )
            .unwrap();
            assert_eq!(
                updated["members"],
                json!([{"value":"third"},{"value":"second"}])
            );
            let reinitialized = patch_resource(&updated, patch(json!([
                {"op":"remove","path":"members"},
                {"op":"add","path":"members","value":[{"value":"third"},{"value":"second"},{"value":"third"}]}
            ]))).unwrap();
            assert_eq!(reinitialized, updated);
            assert_eq!(old, before);
        }
    }
}
