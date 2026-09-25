//! SCIM 2.0 user/group provisioning. Each authenticated operator owns its provisioned records.
use crate::{
    agent::Principal,
    core::{Core, audit, make_user, user_by_name, validate_display, validate_email, validate_name},
    crypto,
    error::{Error, Result},
    model::{Group, NewUser, User},
    store::Tx,
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub const USER: &str = "urn:ietf:params:scim:schemas:core:2.0:User";
pub const GROUP: &str = "urn:ietf:params:scim:schemas:core:2.0:Group";
const LIST: &str = "urn:ietf:params:scim:api:messages:2.0:ListResponse";
const PATCH: &str = "urn:ietf:params:scim:api:messages:2.0:PatchOp";

#[derive(Clone, Serialize, Deserialize)]
struct Record {
    owner: String,
    kind: String,
    local_id: String,
    external_id: Option<String>,
    data: Value,
    deleted: bool,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub start_index: Option<usize>,
    pub count: Option<usize>,
    pub filter: Option<String>,
}

pub fn response(result: Result<Value>, status: StatusCode) -> Response {
    let mut response = match result {
        Ok(value) => {
            let location = value["meta"]["location"].as_str().map(String::from);
            let etag = value["meta"]["version"].as_str().map(String::from);
            let mut out = if status == StatusCode::NO_CONTENT {
                status.into_response()
            } else {
                (status, Json(value)).into_response()
            };
            if let Some(location) = location.and_then(|v| v.parse().ok()) {
                out.headers_mut().insert("location", location);
            }
            if let Some(etag) = etag.and_then(|v| v.parse().ok()) {
                out.headers_mut().insert("etag", etag);
            }
            out
        }
        Err(mut error) => {
            if error.code == "conflict" && error.message == "Configuration revision changed" {
                error.status = StatusCode::PRECONDITION_FAILED;
                error.code = "precondition_failed";
            }
            let typ = match error.code {
                "conflict" => "uniqueness",
                "invalid_filter" => "invalidFilter",
                "mutability" => "mutability",
                _ => "invalidValue",
            };
            let mut body = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:Error"],"status":error.status.as_u16().to_string(),"detail":error.message});
            if error.code != "precondition_failed" {
                body["scimType"] = json!(typ);
            }
            (error.status, Json(body)).into_response()
        }
    };
    if status != StatusCode::NO_CONTENT || response.status() != StatusCode::NO_CONTENT {
        response
            .headers_mut()
            .insert("content-type", "application/scim+json".parse().unwrap());
    }
    response
}

pub fn metadata(kind: &str) -> Result<Value> {
    Ok(match kind {
        "ServiceProviderConfig" => {
            json!({"schemas":["urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig"],"patch":{"supported":true},"bulk":{"supported":false,"maxOperations":0,"maxPayloadSize":0},"filter":{"supported":true,"maxResults":1000},"changePassword":{"supported":true},"sort":{"supported":false},"etag":{"supported":true},"authenticationSchemes":[{"type":"oauthbearertoken","name":"Scoped operator credential","description":"Use a dedicated agent credential and If-Match revision for mutations","primary":true}]})
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
fn filter_matches(value: &Value, filter: Option<&str>) -> Result<bool> {
    let Some(filter) = filter else {
        return Ok(true);
    };
    if filter.len() > 1024 {
        return Err(Error::oauth("invalid_filter", "Filter too long"));
    }
    let mut parts = filter.splitn(3, ' ');
    let field = parts.next().unwrap_or("");
    let op = parts.next().unwrap_or("");
    let needle = parts.next().unwrap_or("");
    let field = match field.to_ascii_lowercase().as_str() {
        "username" => "userName",
        "externalid" => "externalId",
        "displayname" => "displayName",
        "id" => "id",
        _ => {
            return Err(Error::oauth(
                "invalid_filter",
                "Supported filters: userName/externalId/displayName/id eq JSON-string",
            ));
        }
    };
    if !op.eq_ignore_ascii_case("eq") {
        return Err(Error::oauth(
            "invalid_filter",
            "Only eq filters are supported",
        ));
    }
    let needle: String = serde_json::from_str(needle).map_err(|_| {
        Error::oauth(
            "invalid_filter",
            "Filter value must be a quoted JSON string",
        )
    })?;
    Ok(value[field].as_str().is_some_and(|v| {
        if field == "externalId" || field == "id" {
            v == needle
        } else {
            v.eq_ignore_ascii_case(&needle)
        }
    }))
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
        value["meta"] = json!({"resourceType":if record.kind=="Users"{"User"}else{"Group"},"location":format!("{}/scim/v2/{}/{id}",self.config.issuer.trim_end_matches('/'),record.kind),"version":format!("\"{}\"",tx.get::<u64>("meta","revision")?.unwrap_or(0))});
        Ok(value)
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
        // Validate unsupported filters even when the collection is empty.
        filter_matches(&json!({}),query.filter.as_deref())?;
        let start=query.start_index.unwrap_or(1).max(1);let count=query.count.unwrap_or(100).min(1000);
        let mut values=Vec::new();
        for (id,record) in tx.list::<Record>(bucket(kind)?)? {
            if record.deleted || record.owner!=actor.id || require(&actor,&record,"read").is_err(){continue;}
            let value=self.scim_view(tx,&id,&record)?;
            if filter_matches(&value,query.filter.as_deref())?{values.push(value);}
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
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            let existing = id.map(|id| owned(tx, &actor, kind, id)).transpose()?;
            let mut data = if patch {
                patch_resource(
                    &existing
                        .as_ref()
                        .ok_or_else(|| Error::bad("Patch requires a resource"))?
                        .data,
                    input,
                )?
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
                    crate::password_history::accept(
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
                if members.len() > 1000 {
                    return Err(Error::bad("Too many group members"));
                }
                data["members"] = json!(public);
                tx.put(
                    "groups",
                    &label,
                    &Group {
                        name: label.clone(),
                        members,
                    },
                )?;
                label.clone()
            };
            data.as_object_mut().unwrap().remove("meta");
            data.as_object_mut().unwrap().remove("id");
            let record = Record {
                owner: actor.id.clone(),
                kind: kind.into(),
                local_id,
                external_id,
                data,
                deleted: false,
            };
            tx.put(bucket(kind)?, &id, &record)?;
            audit(tx, &actor.id, &format!("{}.scim", scope(kind)), &label)?;
            self.scim_view(tx, &id, &record)
        })
    }
    pub fn scim_delete(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            let mut record = owned(tx, &actor, kind, id)?;
            require(&actor, &record, "write")?;
            if kind == "Users" {
                let mut user = user_by_name(tx, name(&record))?;
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
                for (key, mut group) in tx.list::<Group>("groups")? {
                    if group.members.remove(&user.id) {
                        tx.put("groups", &key, &group)?;
                    }
                }
            } else {
                require(&actor, &record, "members")?;
                tx.put(
                    "groups",
                    &record.local_id,
                    &Group {
                        name: record.local_id.clone(),
                        members: Default::default(),
                    },
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
            let current = data[path].as_array_mut().unwrap();
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
