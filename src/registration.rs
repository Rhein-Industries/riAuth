//! RFC 7591 registration using bounded, revocable initial access tokens.
use crate::{
    core::{Core, audit, validate_client, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    jose::{ClientAuthMethod, PublicJwks},
    model::{Client, ProviderSettings},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationTemplate {
    pub id: String,
    pub redirect_uris: Vec<String>,
    pub scopes: BTreeSet<String>,
    pub grant_types: BTreeSet<String>,
    pub auth_methods: BTreeSet<String>,
    pub settings: ProviderSettings,
    #[serde(default)]
    pub allowed_groups: BTreeSet<String>,
    #[serde(default)]
    pub require_mfa: bool,
    pub ttl: u64,
    pub max_uses: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct InitialAccess {
    template: RegistrationTemplate,
    token_hash: String,
    created_by: String,
    creator_agent: bool,
    expires_at: u64,
    used: u32,
    enabled: bool,
}
impl InitialAccess {
    fn view(&self) -> Value {
        json!({"template": self.template, "created_by": self.created_by, "expires_at": self.expires_at, "used": self.used, "enabled": self.enabled})
    }
}

#[derive(schemars::JsonSchema, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub scope: Option<String>,
    pub grant_types: Option<BTreeSet<String>>,
    pub response_types: Option<BTreeSet<String>>,
    pub token_endpoint_auth_method: Option<String>,
    pub application_type: Option<String>,
    pub jwks: Option<PublicJwks>,
    pub post_logout_redirect_uris: Option<Vec<String>>,
}

fn metadata(message: &str) -> Error {
    Error::oauth("invalid_client_metadata", message)
}

impl Core {
    pub fn registration_template(
        &self,
        token: &str,
        template: RegistrationTemplate,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(
                tx,
                token,
                "registration.write",
                &format!("registration/{}", template.id),
            )?;
            actor.require("client.write", "*")?;
            validate_name(&template.id)?;
            if !(60..=2_592_000).contains(&template.ttl)
                || !(1..=1000).contains(&template.max_uses)
                || template.redirect_uris.is_empty()
                || template.auth_methods.is_empty()
                || template.auth_methods.iter().any(|m| {
                    ![
                        "none",
                        "client_secret_basic",
                        "client_secret_post",
                        "private_key_jwt",
                    ]
                    .contains(&m.as_str())
                })
                || template.grant_types.is_empty()
                || template.grant_types.iter().any(|g| {
                    ![
                        "authorization_code",
                        "refresh_token",
                        crate::oidc::DEVICE_GRANT,
                    ]
                    .contains(&g.as_str())
                })
            {
                return Err(Error::bad(
                    "Invalid registration template limits, grants or authentication methods",
                ));
            }
            // Registration cannot create machine trusts, exchange permissions or inherit keys.
            if template.settings.exchange.is_some()
                || !template.settings.exchange_from.is_empty()
                || !template.settings.machine_trust.is_empty()
                || template.settings.jwks.is_some()
                || template.settings.token_endpoint_auth_method.is_some()
            {
                return Err(Error::bad(
                    "Registration templates cannot delegate machine/exchange trust or client keys",
                ));
            }
            if template.settings.implicit_consent {
                return Err(Error::bad(
                    "Registration templates cannot skip browser consent",
                ));
            }
            let mut sample = Client {
                id: "registration-validation".into(),
                name: "Registration validation".into(),
                secret_hash: None,
                redirect_uris: template.redirect_uris.clone(),
                scopes: template.scopes.clone(),
                allowed_groups: template.allowed_groups.clone(),
                require_mfa: template.require_mfa,
                enabled: true,
                service: false,
                settings: template.settings.clone(),
            };
            sample.settings.allowed_grants = template.grant_types.clone();
            validate_client(tx, &sample)?;
            if tx
                .get::<InitialAccess>("registrations", &template.id)?
                .is_some()
            {
                return Err(Error::conflict("Registration template already exists"));
            }
            let credential = crypto::random_token("ri_register_");
            let record = InitialAccess {
                token_hash: digest(&credential),
                created_by: actor.id.clone(),
                creator_agent: actor.agent,
                expires_at: now() + template.ttl,
                used: 0,
                enabled: true,
                template,
            };
            tx.put("registrations", &record.template.id, &record)?;
            tx.put(
                "registration_tokens",
                &record.token_hash,
                &record.template.id,
            )?;
            audit(tx, &actor.id, "registration.create", &record.template.id)?;
            Ok(json!({"registration": record.view(), "initial_access_token": credential}))
        })
    }
    pub fn registration_templates(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<InitialAccess>("registrations")?
                    .iter()
                    .filter(|(_, r)| actor.allows(
                        "registration.read",
                        &format!("registration/{}", r.template.id)
                    ))
                    .map(|(_, r)| r.view())
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn revoke_registration(&self, token: &str, id: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(
                tx,
                token,
                "registration.write",
                &format!("registration/{id}"),
            )?;
            let mut record = tx
                .get::<InitialAccess>("registrations", id)?
                .ok_or_else(|| Error::missing("Template not found"))?;
            record.enabled = false;
            tx.put("registrations", id, &record)?;
            audit(tx, &actor.id, "registration.revoke", id)?;
            Ok(record.view())
        })
    }
    pub fn dynamic_register(
        &self,
        initial_token: &str,
        request: RegistrationRequest,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let hash = digest(initial_token);
            let id = tx.get::<String>("registration_tokens", &hash)?.ok_or_else(Error::unauthorized)?;
            let mut record = tx.get::<InitialAccess>("registrations", &id)?.filter(|r| r.enabled && r.expires_at > now() && r.used < r.template.max_uses)
                .ok_or_else(Error::unauthorized)?;
            check_creator(tx, &record)?;
            let template = &record.template;
            if request.redirect_uris.is_empty() || request.redirect_uris.iter().any(|uri| !template.redirect_uris.contains(uri)) {
                return Err(Error::oauth("invalid_redirect_uri", "Redirects must be an exact subset of the registration template"));
            }
            let method = request.token_endpoint_auth_method.as_deref().unwrap_or("client_secret_basic");
            if !template.auth_methods.contains(method) { return Err(metadata("Authentication method is outside the template")); }
            let grants = request.grant_types.unwrap_or_else(|| BTreeSet::from(["authorization_code".into()]));
            if grants.is_empty() || !grants.is_subset(&template.grant_types)
                || request.response_types.as_ref().is_some_and(|r| r != &BTreeSet::from(["code".into()])) {
                return Err(metadata("Unsupported grant or response type"));
            }
            if request.application_type.as_deref().is_some_and(|t| t != if template.settings.native { "native" } else { "web" }) {
                return Err(metadata("Application type is fixed by the template"));
            }
            let secret = ["client_secret_basic", "client_secret_post"].contains(&method).then(|| crypto::random_token("ri_client_"));
            let mut settings = template.settings.clone();
            // Only client.write holders may waive consent, never a self-registered client.
            settings.implicit_consent = false;
            settings.allowed_grants = grants;
            settings.token_endpoint_auth_method = Some(match method {
                "none" => ClientAuthMethod::None, "client_secret_basic" => ClientAuthMethod::ClientSecretBasic,
                "client_secret_post" => ClientAuthMethod::ClientSecretPost, "private_key_jwt" => ClientAuthMethod::PrivateKeyJwt,
                _ => return Err(metadata("Unsupported authentication method")),
            });
            if let Some(uris) = request.post_logout_redirect_uris {
                if uris.iter().any(|uri| !settings.post_logout_redirect_uris.contains(uri)) { return Err(metadata("Logout redirects are outside the template")); }
                settings.post_logout_redirect_uris = uris;
            }
            settings.jwks = request.jwks;
            let cid = format!("{}-{}", template.id, crypto::id());
            validate_name(&cid).map_err(|_| metadata("Template id leaves insufficient space for a generated client id"))?;
            let mut client = Client { id: cid, name: request.client_name.unwrap_or_else(|| template.id.clone()), secret_hash: secret.as_deref().map(digest),
                redirect_uris: request.redirect_uris, scopes: template.scopes.clone(), allowed_groups: template.allowed_groups.clone(),
                require_mfa: template.require_mfa, enabled: true, service: false, settings };
            if let Some(scope) = request.scope { client.scopes = crate::oidc::scope_request(&scope, &client).map_err(|_| metadata("Scopes are outside the template"))?; }
            crate::core::validate_display(&client.name).map_err(|_| metadata("Invalid client name"))?;
            validate_client(tx, &client).map_err(|_| metadata("Client metadata conflicts with provider policy"))?;
            tx.put("clients", &client.id, &client)?;
            record.used += 1; tx.put("registrations", &id, &record)?;
            audit(tx, &format!("registration:{id}"), "client.register", &client.id)?;
            let mut response = json!({"client_id": client.id, "client_id_issued_at": now(), "client_name": client.name,
                "redirect_uris": client.redirect_uris, "scope": client.scopes.iter().cloned().collect::<Vec<_>>().join(" "),
                "grant_types": client.settings.allowed_grants, "response_types": ["code"], "token_endpoint_auth_method": method,
                "application_type": if client.settings.native { "native" } else { "web" },
                "post_logout_redirect_uris": client.settings.post_logout_redirect_uris});
            if let Some(jwks) = &client.settings.jwks { response["jwks"] = json!(jwks); }
            if let Some(secret) = secret { response["client_secret"] = json!(secret); response["client_secret_expires_at"] = json!(0); }
            Ok(response)
        })
    }
}

fn check_creator(tx: &Tx<'_>, record: &InitialAccess) -> Result<()> {
    if record.creator_agent {
        let agent = tx
            .get::<crate::agent::Agent>(
                "agents",
                record
                    .created_by
                    .strip_prefix("agent:")
                    .unwrap_or(&record.created_by),
            )?
            .ok_or_else(Error::unauthorized)?;
        if !crate::agent::authority_active(tx, &agent)? {
            return Err(Error::unauthorized());
        }
        let actor = crate::agent::Principal {
            id: record.created_by.clone(),
            agent: true,
            permissions: agent.permissions,
        };
        actor.require("client.write", "*")?;
        actor.require(
            "registration.write",
            &format!("registration/{}", record.template.id),
        )?;
    } else {
        tx.get::<crate::model::User>("users", &record.created_by)?
            .filter(|u| u.enabled && u.admin)
            .ok_or_else(Error::unauthorized)?;
    }
    Ok(())
}
