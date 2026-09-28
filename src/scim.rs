//! SCIM 2.0 user/group provisioning. Each authenticated operator owns its provisioned records.
use crate::{
    agent::Principal,
    core::{Core, audit, make_user, validate_display, validate_email, validate_name},
    crypto,
    error::{Error, Result},
    model::{Group, NewUser, User},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

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
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub attributes: Option<String>,
    pub excluded_attributes: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectionQuery {
    pub attributes: Option<String>,
    pub excluded_attributes: Option<String>,
}

pub fn metadata(kind: &str) -> Result<Value> {
    Ok(match kind {
        "ServiceProviderConfig" => {
            json!({"schemas":["urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig"],"patch":{"supported":true},"bulk":{"supported":false,"maxOperations":0,"maxPayloadSize":0},"filter":{"supported":true,"maxResults":1000},"changePassword":{"supported":true},"sort":{"supported":true},"etag":{"supported":true},"authenticationSchemes":[{"type":"oauthbearertoken","name":"Scoped operator credential","description":"Use a dedicated agent credential and resource If-Match for updates and deletes","primary":true}]})
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
            let effective = [
                "username",
                "display_name",
                "email",
                "enabled",
                "password_hash",
            ];
            if !effective.iter().any(|field| {
                before.and_then(|v| v.get(*field)) != after.and_then(|v| v.get(*field))
            }) {
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
                    .map(|value| {
                        serde_json::from_value::<Group>(value.clone())
                            .map(|group| group.members)
                            .map_err(Error::internal)
                    })
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
    Present(&'static str),
    EmailPresent,
    EmailPath(Box<FilterExpr>),
    EmailText {
        field: &'static str,
        needle: String,
    },
    EmailPrimary(bool),
    MemberValue(String),
}

enum FilterExpr {
    Predicate(Filter),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl FilterExpr {
    fn matches(&self, value: &Value) -> bool {
        match self {
            Self::Predicate(predicate) => predicate.matches(value),
            Self::And(left, right) => left.matches(value) && right.matches(value),
            Self::Or(left, right) => left.matches(value) || right.matches(value),
        }
    }
}

impl Filter {
    fn matches(&self, value: &Value) -> bool {
        match self {
            Self::Text {
                field,
                needle,
                exact,
            } => value[*field].as_str().is_some_and(|actual| {
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
            Self::Present(field) => match &value[*field] {
                Value::Null => false,
                Value::String(s) => !s.is_empty(),
                Value::Array(items) => !items.is_empty(),
                Value::Object(fields) => !fields.is_empty(),
                _ => true, // An assigned boolean is present even when false.
            },
            Self::EmailPresent => value["emails"].as_array().is_some_and(|emails| {
                emails.iter().any(|email| {
                    email["value"]
                        .as_str()
                        .is_some_and(|actual| !actual.is_empty())
                })
            }),
            Self::EmailPath(expr) => value["emails"]
                .as_array()
                .is_some_and(|emails| emails.iter().any(|email| expr.matches(email))),
            Self::EmailText { field, needle } => value[*field]
                .as_str()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(needle)),
            Self::EmailPrimary(expected) => value["primary"].as_bool() == Some(*expected),
            Self::MemberValue(needle) => value["value"].as_str() == Some(needle),
        }
    }
}

const MAX_FILTER_EXPRESSIONS: usize = 8;
const MAX_FILTER_DEPTH: usize = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterScope {
    Users,
    Groups,
    Email,
    Member,
}

fn invalid_filter(message: &'static str) -> Error {
    Error::oauth("invalid_filter", message)
}

fn parse_filter_predicate(
    scope: FilterScope,
    field: &str,
    op: &str,
    literal: Option<&str>,
) -> Result<Filter> {
    let field = field.to_ascii_lowercase();
    if op.eq_ignore_ascii_case("pr") {
        return match (scope, field.as_str()) {
            (FilterScope::Users, "username") => Ok(Filter::Present("userName")),
            (FilterScope::Users | FilterScope::Groups, "displayname") => {
                Ok(Filter::Present("displayName"))
            }
            (FilterScope::Users | FilterScope::Groups, "externalid") => {
                Ok(Filter::Present("externalId"))
            }
            (FilterScope::Users | FilterScope::Groups, "id") => Ok(Filter::Present("id")),
            (FilterScope::Users, "active") => Ok(Filter::Present("active")),
            (FilterScope::Users, "emails") => Ok(Filter::Present("emails")),
            (FilterScope::Users, "emails.value") => Ok(Filter::EmailPresent),
            (FilterScope::Email, "value") => Ok(Filter::Present("value")),
            (FilterScope::Email, "type") => Ok(Filter::Present("type")),
            (FilterScope::Email, "primary") => Ok(Filter::Present("primary")),
            _ => Err(invalid_filter("Unsupported presence attribute")),
        };
    }
    if !op.eq_ignore_ascii_case("eq") {
        return Err(invalid_filter("Unsupported filter operator"));
    }
    let literal = literal.ok_or_else(|| invalid_filter("Expected filter value"))?;
    if matches!(
        (scope, field.as_str()),
        (FilterScope::Users, "active") | (FilterScope::Email, "primary")
    ) {
        let active = serde_json::from_str::<bool>(literal)
            .map_err(|_| invalid_filter("Boolean filter requires true or false"))?;
        return Ok(if scope == FilterScope::Email {
            Filter::EmailPrimary(active)
        } else {
            Filter::Active(active)
        });
    }
    let needle = serde_json::from_str::<String>(literal)
        .map_err(|_| invalid_filter("Filter value must be one quoted JSON string"))?;
    match (scope, field.as_str()) {
        (FilterScope::Users, "username") => Ok(Filter::Text {
            field: "userName",
            needle,
            exact: false,
        }),
        (FilterScope::Users | FilterScope::Groups, "displayname") => Ok(Filter::Text {
            field: "displayName",
            needle,
            exact: false,
        }),
        (FilterScope::Users | FilterScope::Groups, "externalid") => Ok(Filter::Text {
            field: "externalId",
            needle,
            exact: true,
        }),
        (FilterScope::Users | FilterScope::Groups, "id") => Ok(Filter::Text {
            field: "id",
            needle,
            exact: true,
        }),
        (FilterScope::Users, "emails.value") => Ok(Filter::Email(needle)),
        (FilterScope::Email, "value") => Ok(Filter::EmailText {
            field: "value",
            needle,
        }),
        (FilterScope::Email, "type") => Ok(Filter::EmailText {
            field: "type",
            needle,
        }),
        (FilterScope::Member, "value") => Ok(Filter::MemberValue(needle)),
        _ => Err(invalid_filter("Unsupported equality attribute")),
    }
}

struct FilterParser<'a> {
    input: &'a str,
    pos: usize,
    expressions: usize,
}

impl<'a> FilterParser<'a> {
    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn spaces(&mut self) -> bool {
        let start = self.pos;
        while self.peek() == Some(b' ') {
            self.pos += 1;
        }
        self.pos != start
    }

    fn word(&mut self) -> &'a str {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|byte| !matches!(byte, b' ' | b'(' | b')' | b'[' | b']' | b'"'))
        {
            self.pos += 1;
        }
        &self.input[start..self.pos]
    }

    fn literal(&mut self) -> Result<&'a str> {
        if self.peek() != Some(b'"') {
            return Ok(self.word());
        }
        let start = self.pos;
        self.pos += 1;
        let mut escaped = false;
        while let Some(byte) = self.peek() {
            self.pos += 1;
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                return Ok(&self.input[start..self.pos]);
            }
        }
        Err(invalid_filter("Unterminated filter string"))
    }

    // Recursive descent gives parentheses the highest precedence, then `and`,
    // then `or`. The shared predicate counter also includes email valuePaths.
    fn or(&mut self, scope: FilterScope, depth: usize) -> Result<FilterExpr> {
        let mut left = self.and(scope, depth)?;
        loop {
            let before = self.pos;
            if !self.spaces() || !self.word().eq_ignore_ascii_case("or") {
                self.pos = before;
                break;
            }
            if !self.spaces() {
                return Err(invalid_filter("Expected expression after or"));
            }
            let right = self.and(scope, depth)?;
            left = FilterExpr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn and(&mut self, scope: FilterScope, depth: usize) -> Result<FilterExpr> {
        let mut left = self.atom(scope, depth)?;
        loop {
            let before = self.pos;
            if !self.spaces() || !self.word().eq_ignore_ascii_case("and") {
                self.pos = before;
                break;
            }
            if !self.spaces() {
                return Err(invalid_filter("Expected expression after and"));
            }
            let right = self.atom(scope, depth)?;
            left = FilterExpr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn atom(&mut self, scope: FilterScope, depth: usize) -> Result<FilterExpr> {
        self.spaces();
        if self.peek() == Some(b'(') {
            if depth == MAX_FILTER_DEPTH {
                return Err(invalid_filter("Filter nesting too deep"));
            }
            self.pos += 1;
            self.spaces();
            let expr = self.or(scope, depth + 1)?;
            self.spaces();
            if self.peek() != Some(b')') {
                return Err(invalid_filter("Expected closing parenthesis"));
            }
            self.pos += 1;
            return Ok(expr);
        }
        let field = self.word();
        if field.is_empty() {
            return Err(invalid_filter("Expected filter attribute"));
        }
        if self.peek() == Some(b'[') {
            if scope != FilterScope::Users || !field.eq_ignore_ascii_case("emails") {
                return Err(invalid_filter("Unsupported valuePath attribute"));
            }
            if depth == MAX_FILTER_DEPTH {
                return Err(invalid_filter("Filter nesting too deep"));
            }
            self.pos += 1;
            self.spaces();
            let expr = self.or(FilterScope::Email, depth + 1)?;
            self.spaces();
            if self.peek() != Some(b']') {
                return Err(invalid_filter("Expected closing valuePath bracket"));
            }
            self.pos += 1;
            return Ok(FilterExpr::Predicate(Filter::EmailPath(Box::new(expr))));
        }
        if !self.spaces() {
            return Err(invalid_filter("Expected filter operator"));
        }
        let op = self.word();
        let literal = if op.eq_ignore_ascii_case("eq") {
            if !self.spaces() {
                return Err(invalid_filter("Expected filter value"));
            }
            Some(self.literal()?)
        } else {
            None
        };
        if self.expressions == MAX_FILTER_EXPRESSIONS {
            return Err(invalid_filter("Too many filter expressions"));
        }
        self.expressions += 1;
        Ok(FilterExpr::Predicate(parse_filter_predicate(
            scope, field, op, literal,
        )?))
    }
}

fn parse_filter(kind: &str, filter: Option<&str>) -> Result<Option<FilterExpr>> {
    let Some(filter) = filter else {
        return Ok(None);
    };
    if filter.len() > 1024 {
        return Err(Error::oauth("invalid_filter", "Filter too long"));
    }
    if filter.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(Error::oauth(
            "invalid_filter",
            "Control characters are not allowed in filters",
        ));
    }
    let scope = match kind {
        "Users" => FilterScope::Users,
        "Groups" => FilterScope::Groups,
        _ => return Err(invalid_filter("Unsupported resource type")),
    };
    let mut parser = FilterParser {
        input: filter,
        pos: 0,
        expressions: 0,
    };
    parser.spaces();
    let expr = parser.or(scope, 0)?;
    parser.spaces();
    if parser.pos != filter.len() {
        return Err(invalid_filter("Unsupported filter syntax"));
    }
    Ok(Some(expr))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortField {
    Id,
    ExternalId,
    UserName,
    DisplayName,
    Active,
    EmailValue,
    NameFormatted,
    NameGiven,
    NameFamily,
}

#[derive(Clone, Copy)]
struct SortSpec {
    field: SortField,
    descending: bool,
}

fn parse_sort(kind: &str, by: Option<&str>, order: Option<&str>) -> Result<Option<SortSpec>> {
    let Some(by) = by else {
        if order.is_some() {
            return Err(Error::bad("sortOrder requires sortBy"));
        }
        return Ok(None);
    };
    if by.len() > 64 || order.is_some_and(|order| order.len() > 16) {
        return Err(Error::bad("Unsupported SCIM sort parameters"));
    }
    let field = match (kind, by.to_ascii_lowercase().as_str()) {
        ("Users" | "Groups", "id") => SortField::Id,
        ("Users" | "Groups", "externalid") => SortField::ExternalId,
        ("Users" | "Groups", "displayname") => SortField::DisplayName,
        ("Users", "username") => SortField::UserName,
        ("Users", "active") => SortField::Active,
        ("Users", "emails.value") => SortField::EmailValue,
        ("Users", "name.formatted") => SortField::NameFormatted,
        ("Users", "name.givenname") => SortField::NameGiven,
        ("Users", "name.familyname") => SortField::NameFamily,
        _ => return Err(Error::bad("Unsupported SCIM sortBy attribute")),
    };
    let descending = match order {
        None => false,
        Some(order) if order.eq_ignore_ascii_case("ascending") => false,
        Some(order) if order.eq_ignore_ascii_case("descending") => true,
        _ => return Err(Error::bad("Unsupported SCIM sortOrder")),
    };
    Ok(Some(SortSpec { field, descending }))
}

enum SortKey<'a> {
    Text(&'a str),
    Bool(bool),
}

fn sort_key(value: &Value, field: SortField) -> Option<SortKey<'_>> {
    if field == SortField::Active {
        return value["active"].as_bool().map(SortKey::Bool);
    }
    let text = match field {
        SortField::Id => value["id"].as_str(),
        SortField::ExternalId => value["externalId"].as_str(),
        SortField::UserName => value["userName"].as_str(),
        SortField::DisplayName => value["displayName"].as_str(),
        SortField::EmailValue => value["emails"].as_array().and_then(|emails| {
            emails
                .iter()
                .find(|email| email["primary"] == true)
                .or_else(|| emails.first())
                .and_then(|email| email["value"].as_str())
        }),
        SortField::NameFormatted => value["name"]["formatted"].as_str(),
        SortField::NameGiven => value["name"]["givenName"].as_str(),
        SortField::NameFamily => value["name"]["familyName"].as_str(),
        SortField::Active => unreachable!(),
    };
    text.filter(|text| !text.is_empty()).map(SortKey::Text)
}

fn compare_sort(left: &Value, right: &Value, spec: SortSpec) -> Ordering {
    let primary = match (sort_key(left, spec.field), sort_key(right, spec.field)) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => {
            if spec.descending {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Some(_), None) => {
            if spec.descending {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (Some(SortKey::Bool(a)), Some(SortKey::Bool(b))) => {
            if spec.descending {
                b.cmp(&a)
            } else {
                a.cmp(&b)
            }
        }
        (Some(SortKey::Text(a)), Some(SortKey::Text(b))) => {
            let order = if matches!(spec.field, SortField::Id | SortField::ExternalId) {
                a.cmp(b)
            } else {
                a.to_lowercase().cmp(&b.to_lowercase())
            };
            if spec.descending {
                order.reverse()
            } else {
                order
            }
        }
        _ => Ordering::Equal,
    };
    // IDs break case-folded and missing-value ties in either direction.
    primary.then_with(|| left["id"].as_str().cmp(&right["id"].as_str()))
}

const MAX_PROJECTION_SELECTORS: usize = 32;

enum FieldSelection {
    Whole,
    Subfields(BTreeSet<&'static str>),
}

enum Projection {
    Full,
    Include(BTreeMap<&'static str, FieldSelection>),
    Exclude(BTreeMap<&'static str, FieldSelection>),
}

fn projection_path(kind: &str, raw: &str) -> Result<(&'static str, Option<&'static str>)> {
    let schema = if kind == "Users" { USER } else { GROUP };
    let path = if raw
        .get(..schema.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(schema))
        && raw.as_bytes().get(schema.len()) == Some(&b':')
    {
        &raw[schema.len() + 1..]
    } else {
        raw
    };
    match (kind, path.to_ascii_lowercase().as_str()) {
        (_, "schemas") => Ok(("schemas", None)),
        (_, "id") => Ok(("id", None)),
        (_, "externalid") => Ok(("externalId", None)),
        (_, "meta") => Ok(("meta", None)),
        (_, "meta.resourcetype") => Ok(("meta", Some("resourceType"))),
        (_, "meta.location") => Ok(("meta", Some("location"))),
        (_, "meta.version") => Ok(("meta", Some("version"))),
        (_, "displayname") => Ok(("displayName", None)),
        ("Users", "username") => Ok(("userName", None)),
        ("Users", "active") => Ok(("active", None)),
        ("Users", "name") => Ok(("name", None)),
        ("Users", "name.formatted") => Ok(("name", Some("formatted"))),
        ("Users", "name.givenname") => Ok(("name", Some("givenName"))),
        ("Users", "name.familyname") => Ok(("name", Some("familyName"))),
        ("Users", "emails") => Ok(("emails", None)),
        ("Users", "emails.value") => Ok(("emails", Some("value"))),
        ("Users", "emails.type") => Ok(("emails", Some("type"))),
        ("Users", "emails.primary") => Ok(("emails", Some("primary"))),
        ("Users", "groups") => Ok(("groups", None)),
        ("Users", "groups.value") => Ok(("groups", Some("value"))),
        ("Users", "groups.display") => Ok(("groups", Some("display"))),
        ("Groups", "members") => Ok(("members", None)),
        ("Groups", "members.value") => Ok(("members", Some("value"))),
        ("Groups", "members.display") => Ok(("members", Some("display"))),
        ("Users", "password") => Err(Error::bad("Write-only SCIM attribute cannot be projected")),
        _ => Err(Error::bad("Unsupported SCIM projection attribute")),
    }
}

fn parse_projection(
    kind: &str,
    attributes: Option<&str>,
    excluded: Option<&str>,
) -> Result<Projection> {
    let (include, raw) = match (attributes, excluded) {
        (None, None) => return Ok(Projection::Full),
        (Some(_), Some(_)) => {
            return Err(Error::bad(
                "attributes and excludedAttributes are mutually exclusive",
            ));
        }
        (Some(raw), None) => (true, raw),
        (None, Some(raw)) => (false, raw),
    };
    if raw.len() > 1024 || raw.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(Error::bad("Invalid SCIM projection list"));
    }
    let mut fields = BTreeMap::new();
    let mut count = 0;
    for item in raw.split(',') {
        count += 1;
        if count > MAX_PROJECTION_SELECTORS {
            return Err(Error::bad("Too many SCIM projection attributes"));
        }
        let item = item.trim_matches(' ');
        if item.is_empty() || item.len() > 128 {
            return Err(Error::bad("Invalid SCIM projection attribute"));
        }
        let (root, sub) = projection_path(kind, item)?;
        if let Some(sub) = sub {
            match fields
                .entry(root)
                .or_insert_with(|| FieldSelection::Subfields(BTreeSet::new()))
            {
                FieldSelection::Whole => {}
                FieldSelection::Subfields(selected) => {
                    selected.insert(sub);
                }
            }
        } else {
            fields.insert(root, FieldSelection::Whole);
        }
    }
    Ok(if include {
        Projection::Include(fields)
    } else {
        Projection::Exclude(fields)
    })
}

fn projection_subfields(root: &str) -> &'static [&'static str] {
    match root {
        "meta" => &["resourceType", "location", "version"],
        "name" => &["formatted", "givenName", "familyName"],
        "emails" => &["value", "type", "primary"],
        "groups" | "members" => &["value", "display"],
        _ => &[],
    }
}

fn project_complex(value: Value, root: &str, selected: Option<&BTreeSet<&str>>) -> Option<Value> {
    let fields = projection_subfields(root);
    if fields.is_empty() {
        return Some(value);
    }
    let project_object = |source: Map<String, Value>| {
        let mut projected = Map::new();
        for field in fields
            .iter()
            .copied()
            .filter(|field| selected.is_none_or(|set| set.contains(field)))
        {
            if let Some(value) = source.get(field).or_else(|| {
                source
                    .iter()
                    .find(|(key, _)| key.eq_ignore_ascii_case(field))
                    .map(|(_, value)| value)
            }) {
                projected.insert(field.to_owned(), value.clone());
            }
        }
        projected
    };
    match value {
        Value::Object(source) => {
            let projected = project_object(source);
            if projected.is_empty() && selected.is_some() {
                None
            } else {
                Some(Value::Object(projected))
            }
        }
        Value::Array(items) => {
            let projected: Vec<_> = items
                .into_iter()
                .filter_map(|item| match item {
                Value::Object(source) => {
                    let projected = project_object(source);
                        if projected.is_empty() {
                            None
                        } else {
                            Some(Value::Object(projected))
                        }
                }
                _ => None,
                })
                .collect();
            if projected.is_empty() && selected.is_some() {
                None
            } else {
                Some(Value::Array(projected))
            }
        }
        _ => None,
    }
}

impl Projection {
    fn apply(&self, kind: &str, mut value: Value) -> Value {
        let fields = match self {
            Self::Full => return value,
            Self::Include(fields) | Self::Exclude(fields) => fields,
        };
        let source = value.as_object_mut().expect("SCIM view is an object");
        source.retain(|key, _| match (kind, key.as_str()) {
            (_, "schemas" | "id" | "externalId" | "meta" | "displayName") => true,
            ("Users", "userName" | "active" | "name" | "emails" | "groups") => true,
            ("Groups", "members") => true,
            _ => false,
        });
        for root in ["meta", "name", "emails", "groups", "members"] {
            if let Some(original) = source.remove(root) {
                if let Some(safe) = project_complex(original, root, None) {
                    source.insert(root.to_owned(), safe);
                }
            }
        }
        match self {
            Self::Include(_) => {
                let mut projected = Map::new();
                for root in ["schemas", "id"] {
                    if let Some(required) = source.remove(root) {
                        projected.insert(root.to_owned(), required);
                    }
                }
                for (root, selection) in fields {
                    if matches!(*root, "schemas" | "id") {
                        continue;
                    }
                    if let Some(original) = source.remove(*root) {
                        let selected = match selection {
                            FieldSelection::Whole => Some(original),
                            FieldSelection::Subfields(subfields) => {
                                project_complex(original, root, Some(subfields))
                            }
                        };
                        if let Some(selected) = selected {
                            projected.insert((*root).to_owned(), selected);
                        }
                    }
                }
                Value::Object(projected)
            }
            Self::Exclude(_) => {
                for (root, selection) in fields {
                    if matches!(*root, "schemas" | "id") {
                        continue;
                    }
                    if let Some(original) = source.remove(*root) {
                        if let FieldSelection::Subfields(excluded) = selection {
                            let remaining = projection_subfields(root)
                                .iter()
                                .copied()
                                .filter(|field| !excluded.contains(field))
                                .collect();
                            if let Some(selected) =
                                project_complex(original, root, Some(&remaining))
                            {
                                source.insert((*root).to_owned(), selected);
                            }
                        }
                    }
                }
                value
            }
            Self::Full => unreachable!(),
        }
    }
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
                return Err(Error::new(
                    StatusCode::PRECONDITION_FAILED,
                    "precondition_failed",
                    "No resource exists for If-Match",
                ));
            }
            return Ok(());
        };
        let record = owned(tx, actor, kind, id)?;
        require(actor, &record, "write")?;
        if let Some(context) = context {
            let supplied = context.if_match.as_deref();
            if actor.agent && supplied.is_none() {
                return Err(Error::new(
                    StatusCode::PRECONDITION_REQUIRED,
                    "precondition_required",
                    "Agent SCIM updates and deletes require resource If-Match",
                ));
            }
            if let Some(supplied) = supplied {
                let current = self.scim_view(tx, id, &record)?;
                let version = current["meta"]["version"].as_str().unwrap_or("");
                if !crypto::constant_eq(supplied, version) {
                    return Err(Error::new(
                        StatusCode::PRECONDITION_FAILED,
                        "precondition_failed",
                        "SCIM resource version changed",
                    ));
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
    pub fn scim_get_projected(
        &self,
        token: &str,
        kind: &str,
        id: &str,
        query: ProjectionQuery,
    ) -> Result<(Value, String, String)> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let record = owned(tx, &actor, kind, id)?;
            require(&actor, &record, "read")?;
            let projection = parse_projection(
                kind,
                query.attributes.as_deref(),
                query.excluded_attributes.as_deref(),
            )?;
            let full = self.scim_view(tx, id, &record)?;
            let version = full["meta"]["version"].as_str().unwrap_or("").to_owned();
            let location = full["meta"]["location"].as_str().unwrap_or("").to_owned();
            Ok((projection.apply(kind, full), version, location))
        })
    }
    pub fn scim_list(&self, token: &str, kind: &str, query: Query) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let bucket = bucket(kind)?;
            // Validate once, including when the collection is empty.
            let filter = parse_filter(kind, query.filter.as_deref())?;
            let sort = parse_sort(kind, query.sort_by.as_deref(), query.sort_order.as_deref())?;
            let projection = parse_projection(
                kind,
                query.attributes.as_deref(),
                query.excluded_attributes.as_deref(),
            )?;
            let start = query.start_index.unwrap_or(1).max(1);
            let count = query.count.unwrap_or(100).min(1000);
            let mut total = 0;
            let mut values = Vec::new();
            let mut page_records = Vec::new();
            let mut after = None;
            loop {
                // The read transaction holds one snapshot across all pages on
                // both backends. Never decode the whole SCIM bucket at once.
                let records = tx.scan::<Record>(bucket, after.as_deref(), 128)?;
                if records.is_empty() {
                    break;
                }
                let last_page = records.len() < 128;
                after = Some(records.last().unwrap().0.clone());
                for (id, record) in records {
                    if record.deleted
                        || record.owner != actor.id
                        || require(&actor, &record, "read").is_err()
                    {
                        continue;
                    }
                    if filter.is_none() && sort.is_none() {
                        total += 1;
                        if total >= start && page_records.len() < count {
                            page_records.push((id, record));
                        }
                    } else {
                        let value = self.scim_view(tx, &id, &record)?;
                        if filter.as_ref().is_none_or(|filter| filter.matches(&value)) {
                            total += 1;
                            if sort.is_some() || (total >= start && values.len() < count) {
                                values.push(value);
                            }
                        }
                    }
                }
                if last_page {
                    break;
                }
            }
            // A plain key-order page needs full views only for its returned
            // resources. Those views retain the resource-level ETags and live
            // memberships used by GET, filters, and sorted list requests.
            if filter.is_none() && sort.is_none() {
                for (id, record) in page_records {
                    values.push(self.scim_view(tx, &id, &record)?);
                }
            }
            let page: Vec<_> = if let Some(spec) = sort {
                values.sort_by(|left, right| compare_sort(left, right, spec));
                values.into_iter().skip(start - 1).take(count).collect()
            } else {
                values
            };
            let page: Vec<_> = page
                .into_iter()
                .map(|value| projection.apply(kind, value))
                .collect();
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
        self.mutation_checked(
            token,
            |tx, actor, context| self.scim_precondition(tx, actor, context, kind, id),
            |tx| {
            let actor = self.principal(tx, token)?;
            let existing = id.map(|id| owned(tx, &actor, kind, id)).transpose()?;
                let before_version = existing
                    .as_ref()
                    .map(|record| {
                self.scim_view(tx, id.unwrap(), record)
                    .map(|value| value["meta"]["version"].clone())
                    })
                    .transpose()?;
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
                let exposure_baseline = user.clone();
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
                let email_changed = user.email != email;
                if email_changed {
                    user.email_verified = false;
                }
                user.email = email;
                user.display_name = display;
                let password = data.as_object_mut().unwrap().remove("password");
                // A new passwordless SCIM identity has no verified recovery
                // address or operator-known local credential to carry forward.
                let credential_change = password.is_some() || existing.is_some() && email_changed;
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
                if actor.agent && credential_change {
                    crate::delegation::mark_credential_exposure(
                        tx,
                        &actor,
                        &exposure_baseline,
                    )?;
                }
                if revoke {
                    user.epoch += 1;
                    crate::logout::queue_user(tx, &user.id)?;
                }
                crate::management::write_scim_user(
                    tx,
                    &actor,
                    existing.as_ref().map(|record| record.local_id.as_str()),
                    &user,
                )?;
                if existing.is_none() && !actor.agent && !actor.delegated {
                    crate::delegation::record_elevation_provenance(
                        tx,
                        &user,
                        crate::delegation::ProvenanceBasis::HumanScim,
                    )?;
                }
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
                    version: existing
                        .as_ref()
                        .map_or_else(crypto::id, |record| record.version.clone()),
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
            if kind != "Users" && (record_changed || group_changed) {
                audit(tx, &actor.id, &format!("{}.scim", scope(kind)), &label)?;
            }
            Ok(view)
            },
        )
    }
    pub fn scim_delete(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        self.mutation_checked(
            token,
            |tx, actor, context| self.scim_precondition(tx, actor, context, kind, Some(id)),
            |tx| {
            let actor = self.principal(tx, token)?;
            let mut record = owned(tx, &actor, kind, id)?;
            require(&actor, &record, "write")?;
            if kind == "Users" {
                let user = crate::management::disable_scim_user(
                    tx,
                    &actor,
                    name(&record),
                    &record.local_id,
                )?;
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
            if kind != "Users" {
                audit(
                    tx,
                    &actor.id,
                    &format!("{}.scim_delete", scope(kind)),
                    name(&record),
                )?;
            }
            Ok(json!({}))
            },
        )
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
fn validate_email_entries(data: &Value) -> Result<()> {
    let Some(emails) = data.get("emails") else { return Ok(()); };
    let emails = emails.as_array().filter(|v| v.len() <= 8)
        .ok_or_else(|| Error::bad("emails must be an array of at most eight values"))?;
    let mut seen = BTreeSet::new();
    for email in emails {
        let value = email["value"].as_str().ok_or_else(|| Error::bad("Email value missing"))?;
        validate_email(value)?;
        if !seen.insert(value.to_ascii_lowercase()) { return Err(Error::bad("Duplicate email value")); }
    }
    read_email(data)?;
    Ok(())
}
fn patches_members(input: &Value) -> bool {
    input["Operations"].as_array().is_some_and(|operations| {
        operations.iter().any(|op| {
            op["path"].as_str().is_some_and(|path| {
                path.eq_ignore_ascii_case("members")
                    || path
                        .get(..8)
                        .is_some_and(|head| head.eq_ignore_ascii_case("members["))
            }) || op["path"].is_null()
                && op["value"].as_object().is_some_and(|value| {
                    value.keys().any(|key| key.eq_ignore_ascii_case("members"))
                })
        })
    })
}

// PATCH valuePaths use the same bounded expression parser as list filters, but
// match each element in isolation. Brackets inside a JSON string are consumed
// by FilterParser, so they cannot change the selected sub-attribute.
fn patch_value_path(
    path: &str,
) -> Result<Option<(&'static str, FilterExpr, Option<&'static str>)>> {
    if path.len() > 1024 || path.bytes().any(|b| b.is_ascii_control()) {
        return Err(Error::oauth("invalidPath", "Invalid PATCH path"));
    }
    let Some((root, _)) = path.split_once('[') else {
        return Ok(None);
    };
    let (field, scope) = if root.eq_ignore_ascii_case("emails") {
        ("emails", FilterScope::Email)
    } else if root.eq_ignore_ascii_case("members") {
        ("members", FilterScope::Member)
    } else {
        return Err(Error::oauth("invalidPath", "Unsupported PATCH valuePath"));
    };
    let mut parser = FilterParser {
        input: path,
        pos: root.len() + 1,
        expressions: 0,
    };
    parser.spaces();
    let filter = parser
        .or(scope, 0)
        .map_err(|_| Error::oauth("invalidPath", "Invalid PATCH valuePath filter"))?;
    parser.spaces();
    if parser.peek() != Some(b']') {
        return Err(Error::oauth(
            "invalidPath",
            "PATCH valuePath is missing a closing bracket",
        ));
    }
    parser.pos += 1;
    let tail = &path[parser.pos..];
    let sub = if tail.is_empty() {
        None
    } else if let Some(tail) = tail.strip_prefix('.') {
        match (field, tail.to_ascii_lowercase().as_str()) {
            ("emails", "value") | ("members", "value") => Some("value"),
            ("emails", "type") => Some("type"),
            ("emails", "primary") => Some("primary"),
            ("members", "display") => {
                return Err(Error::oauth("mutability", "Member display is read-only"));
            }
            _ => {
                return Err(Error::oauth(
                    "invalidPath",
                    "Unsupported PATCH sub-attribute",
                ));
            }
        }
    } else {
        return Err(Error::oauth("invalidPath", "Unsupported PATCH path suffix"));
    };
    Ok(Some((field, filter, sub)))
}

fn patch_entry(field: &str, value: &Value) -> Result<Value> {
    let source = value
        .as_object()
        .ok_or_else(|| Error::bad("Complex PATCH value must be an object"))?;
    let mut result = Map::new();
    for (key, value) in source {
        let canonical = match (field, key.to_ascii_lowercase().as_str()) {
            (_, "value") => "value",
            ("emails", "type") => "type",
            ("emails", "primary") => "primary",
            ("members", "display") => {
                if !value.is_string() {
                    return Err(Error::bad("Member display must be a string"));
                }
                // A GET-shaped member may echo display. Membership writes
                // resolve it again from the owned User record.
                continue;
            }
            _ => return Err(Error::bad("Unsupported complex PATCH sub-attribute")),
        };
        if field == "emails" {
            match canonical {
                "value" => {
                    let value = value.as_str().ok_or_else(|| Error::bad("Email value must be a string"))?;
                    validate_email(value)?;
                }
                "type" => {
                    let kind = value.as_str().ok_or_else(|| Error::bad("Email type must be a string"))?;
                    if kind.is_empty() || kind.len() > 64 || kind.chars().any(char::is_control) {
                        return Err(Error::bad("Invalid email type"));
                    }
                }
                "primary" if !value.is_boolean() => {
                    return Err(Error::bad("Email primary must be a boolean"));
                }
                _ => {}
            }
        }
        if result.insert(canonical.into(), value.clone()).is_some() {
            return Err(Error::bad("Duplicate complex PATCH sub-attribute"));
        }
    }
    if field == "members"
        && result.get("value").is_some_and(|value| {
            value.as_str().is_none_or(|id| {
                id.is_empty() || id.len() > 256 || id.chars().any(char::is_control)
            })
        })
    {
        return Err(Error::bad("Invalid member ID"));
    }
    Ok(Value::Object(result))
}

fn patch_entries(field: &str, value: &Value) -> Result<Vec<Value>> {
    let raw: Vec<&Value> = match value {
        Value::Array(values) => values.iter().collect(),
        Value::Object(_) => vec![value],
        _ => return Err(Error::bad("Complex PATCH value must be an object or array")),
    };
    if raw.len() > if field == "emails" { 8 } else { 1000 } {
        return Err(Error::bad("Too many complex PATCH values"));
    }
    raw.into_iter()
        .map(|value| patch_entry(field, value))
        .collect()
}

fn set_email_primary(items: &mut [Value], winner: usize) {
    for (index, email) in items.iter_mut().enumerate() {
        if index != winner {
            email["primary"] = json!(false);
        }
    }
}

fn patch_multi_root(data: &mut Value, field: &str, operation: &str, value: &Value) -> Result<()> {
    let incoming = patch_entries(field, value)?;
    if operation == "replace" {
        data[field] = json!(incoming);
        return Ok(());
    }
    if data.get(field).is_none() {
        data[field] = json!([]);
    }
    let current = data[field]
        .as_array_mut()
        .ok_or_else(|| Error::bad("Complex attribute must be an array"))?;
    for item in incoming {
        let key = item["value"]
            .as_str()
            .ok_or_else(|| Error::bad("Complex value is required"))?;
        let duplicate = current.iter().any(|old| {
            old["value"].as_str().is_some_and(|v| {
                if field == "emails" {
                    v.eq_ignore_ascii_case(key)
                } else {
                    v == key
                }
            })
        });
        if duplicate {
            continue;
        }
        let primary = field == "emails" && item["primary"] == true;
        current.push(item);
        if primary {
            let winner = current.len() - 1;
            set_email_primary(current, winner);
        }
    }
    Ok(())
}

fn patch_multi_selected(
    data: &mut Value,
    field: &str,
    filter: &FilterExpr,
    sub: Option<&str>,
    operation: &str,
    value: &Value,
) -> Result<()> {
    let current = data.get_mut(field).and_then(Value::as_array_mut);
    let Some(current) = current else {
        if operation == "remove" {
            return Ok(());
        }
        return Err(Error::oauth("noTarget", "PATCH valuePath matched no value"));
    };
    let selected: Vec<usize> = current
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| filter.matches(entry).then_some(i))
        .collect();
    if selected.is_empty() {
        if operation == "remove" {
            return Ok(());
        }
        return Err(Error::oauth("noTarget", "PATCH valuePath matched no value"));
    }
    if operation == "remove" {
        if let Some(sub) = sub {
            if sub == "value" {
                return Err(Error::oauth("mutability", "Cannot remove a required value"));
            }
            for &i in &selected {
                current[i].as_object_mut().unwrap().remove(sub);
            }
        } else {
            current.retain(|entry| !filter.matches(entry));
        }
        return Ok(());
    }
    let replacement = if let Some(sub) = sub {
        if field == "members" && sub != "value" {
            return Err(Error::oauth("mutability", "Member display is read-only"));
        }
        let valid = match sub {
            "value" if field == "emails" => {
                value.as_str().is_some_and(|v| validate_email(v).is_ok())
            }
            "value" => value.as_str().is_some_and(|v| {
                !v.is_empty() && v.len() <= 256 && !v.chars().any(char::is_control)
            }),
            "type" => value.as_str().is_some_and(|v| {
                !v.is_empty() && v.len() <= 64 && !v.chars().any(char::is_control)
            }),
            "primary" => value.is_boolean(),
            _ => false,
        };
        if !valid {
            return Err(Error::bad("Invalid complex PATCH sub-attribute value"));
        }
        None
    } else {
        Some(patch_entry(field, value)?)
    };
    let promote = if let Some(sub) = sub {
        sub == "primary" && value == true
    } else {
        replacement
            .as_ref()
            .is_some_and(|value| value["primary"] == true)
    };
    if promote && selected.len() != 1 {
        return Err(Error::bad("Primary email target is ambiguous"));
    }
    for &i in &selected {
        if let Some(sub) = sub {
            current[i][sub] = value.clone();
        } else {
            for (key, value) in replacement.as_ref().unwrap().as_object().unwrap() {
                current[i][key] = value.clone();
            }
        }
    }
    if promote {
        set_email_primary(current, selected[0]);
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum NamePath {
    Whole,
    Sub(&'static str),
}

fn name_path(kind: &str, path: &str) -> Result<Option<NamePath>> {
    let prefix = format!("{USER}:");
    let path = if path.get(..prefix.len()).is_some_and(|head| head.eq_ignore_ascii_case(&prefix)) {
        &path[prefix.len()..]
    } else if path.contains(':') {
        return Err(Error::oauth("invalidPath", "Unsupported PATCH schema path"));
    } else {
        path
    };
    let found = if path.eq_ignore_ascii_case("name") {
        Some(NamePath::Whole)
    } else if let Some((root, sub)) = path.split_once('.') {
        if root.eq_ignore_ascii_case("name") {
            let sub = match sub.to_ascii_lowercase().as_str() {
                "formatted" => "formatted",
                "givenname" => "givenName",
                "familyname" => "familyName",
                _ => return Err(Error::oauth("invalidPath", "Unsupported name sub-attribute")),
            };
            Some(NamePath::Sub(sub))
        } else {
            None
        }
    } else {
        None
    };
    if found.is_some() && kind != "Users" {
        return Err(Error::oauth("invalidPath", "Name is not a Group attribute"));
    }
    Ok(found)
}

fn name_text(value: &Value) -> Result<()> {
    let text = value.as_str().ok_or_else(|| Error::bad("Name sub-attribute must be a string"))?;
    if text.is_empty() || text.len() > 200 || text.chars().any(char::is_control) {
        return Err(Error::bad("Name sub-attribute must be 1–200 bytes without control characters"));
    }
    Ok(())
}

fn name_fields(value: &Value) -> Result<Map<String, Value>> {
    let source = value.as_object().ok_or_else(|| Error::bad("name must be a complex object"))?;
    let mut fields = Map::new();
    for (key, value) in source {
        let canonical = match key.to_ascii_lowercase().as_str() {
            "formatted" => "formatted",
            "givenname" => "givenName",
            "familyname" => "familyName",
            _ => return Err(Error::bad("Unsupported name sub-attribute")),
        };
        name_text(value)?;
        if fields.insert(canonical.to_owned(), value.clone()).is_some() {
            return Err(Error::bad("Duplicate name sub-attribute"));
        }
    }
    Ok(fields)
}

fn stored_name_fields(value: &Value) -> Result<Map<String, Value>> {
    value.as_object().cloned().ok_or_else(|| Error::bad("name must be a complex object"))
}

fn remove_name_key(fields: &mut Map<String, Value>, sub: &str) -> bool {
    let keys: Vec<_> = fields.keys().filter(|key| key.eq_ignore_ascii_case(sub)).cloned().collect();
    for key in &keys { fields.remove(key); }
    !keys.is_empty()
}

fn patch_name(data: &mut Value, path: NamePath, operation: &str, value: &Value) -> Result<()> {
    if operation == "remove" {
        match path {
            NamePath::Whole => {
                if data.get("name").is_none_or(Value::is_null) {
                    return Err(Error::oauth("noTarget", "Name path is not assigned"));
                }
                data.as_object_mut().unwrap().remove("name");
            }
            NamePath::Sub(sub) => {
                let mut fields = data.get("name").filter(|value| !value.is_null())
                    .map(stored_name_fields).transpose()?
                    .ok_or_else(|| Error::oauth("noTarget", "Name path is not assigned"))?;
                if !remove_name_key(&mut fields, sub) {
                    return Err(Error::oauth("noTarget", "Name path is not assigned"));
                }
                if fields.is_empty() {
                    data.as_object_mut().unwrap().remove("name");
                } else {
                    data["name"] = Value::Object(fields);
                }
            }
        }
        return Ok(());
    }
    let updates = match path {
        NamePath::Whole => {
            let fields = name_fields(value)?;
            if fields.is_empty() { return Err(Error::bad("name PATCH value needs a sub-attribute")); }
            fields
        }
        NamePath::Sub(sub) => {
            name_text(value)?;
            Map::from_iter([(sub.to_owned(), value.clone())])
        }
    };
    let mut fields = data.get("name").filter(|value| !value.is_null())
        .map(stored_name_fields).transpose()?.unwrap_or_default();
    for (key, value) in updates {
        remove_name_key(&mut fields, &key);
        fields.insert(key, value);
    }
    data["name"] = Value::Object(fields);
    Ok(())
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
    let mut email_changed = false;
    for op in operations {
        let operation = op["op"].as_str().unwrap_or("").to_ascii_lowercase();
        if !["add", "replace", "remove"].contains(&operation.as_str()) {
            return Err(Error::bad("Unsupported patch operation"));
        }
        if op["path"].is_null() {
            if operation == "remove" {
                return Err(Error::oauth("noTarget", "remove requires a path"));
            }
            let value = normalize(op["value"].clone())?;
            for (key, value) in value.as_object().unwrap() {
                if ["id", "meta", "schemas", "groups"].contains(&key.as_str()) {
                    return Err(Error::oauth(
                        "mutability",
                        "Cannot patch read-only attributes",
                    ));
                }
                if ["emails", "members"].contains(&key.as_str()) {
                    if key == "emails" {
                        email_changed = true;
                    }
                    patch_multi_root(&mut data, key, &operation, value)?;
                } else if key == "name" {
                    if old["schemas"] != json!([USER]) {
                        return Err(Error::oauth("invalidPath", "Name is not a Group attribute"));
                    }
                    patch_name(&mut data, NamePath::Whole, &operation, value)?;
                } else {
                    data[key] = value.clone();
                }
            }
            continue;
        }
        let path = op["path"]
            .as_str()
            .ok_or_else(|| Error::bad("Invalid patch path"))?;
        if let Some((field, filter, sub)) = patch_value_path(path)? {
            let resource_field = if old["schemas"] == json!([USER]) {
                "emails"
            } else {
                "members"
            };
            if field != resource_field {
                return Err(Error::oauth(
                    "invalidPath",
                    "ValuePath is not valid for this resource",
                ));
            }
            if field == "emails" {
                email_changed = true;
            }
            patch_multi_selected(&mut data, field, &filter, sub, &operation, &op["value"])?;
            continue;
        }
        let kind = if old["schemas"] == json!([USER]) { "Users" } else { "Groups" };
        if let Some(name_path) = name_path(kind, path)? {
            patch_name(&mut data, name_path, &operation, &op["value"])?;
            continue;
        }
        if path.contains('.') {
            return Err(Error::oauth("invalidPath", "Unsupported PATCH complex path"));
        }
        let canonical = normalize(json!({path:op["value"]}))?;
        let (path, value) = canonical.as_object().unwrap().iter().next().unwrap();
        if ["id", "meta", "schemas", "groups", "userName"].contains(&path.as_str()) {
            return Err(Error::oauth("mutability", "Cannot patch this attribute"));
        }
        if path == "emails" { email_changed = true; }
        if operation == "remove" {
            data.as_object_mut().unwrap().remove(path);
        } else if ["members", "emails"].contains(&path.as_str()) {
            let resource_field = if old["schemas"] == json!([USER]) {
                "emails"
            } else {
                "members"
            };
            if path != resource_field {
                return Err(Error::oauth(
                    "invalidPath",
                    "Attribute is not valid for this resource",
                ));
            }
            patch_multi_root(&mut data, path, &operation, value)?;
        } else {
            data[path] = value.clone();
        }
    }
    if email_changed && old["schemas"] == json!([USER]) {
        validate_email_entries(&data)?;
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
