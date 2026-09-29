use riauth::{
    encryption::EncryptionKey,
    exchange::ExchangePolicy,
    jose::{ClientAuthMethod, MachineTrust, PublicJwk, PublicJwks},
    model::{Client, ProviderSettings},
};
use std::collections::BTreeSet;

pub const ENCRYPTION_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\n\
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAmrAlfbbb4AD7RYRGr23C\n\
Gh6HrQQzDNvG3dmOdAys9fDuDH+6h70MZ0aOjNYdeYWNLZT2RXXZT0/1Xa5zfKux\n\
Axbk1eVF54/fr8iN+tBq3LXh3XxNj4wERFIk47UGhCsT27TK/SnfpZxcz5BzvsW/\n\
lkwpwEW3Rjh8LuClyRqcxeD6W5/R7E/jTH519J/5fVsadb8k2ekninBFvK3XmZe/\n\
+9tEPOxxfZMU99S6CEUpN5F4Uv+UVY9ysZPGy7aD4Hit8NEGP704AKvWTSnJ66Uz\n\
Oy03hBcNP05WHFLSA+n4EHlWyX4OaDmcfd+ck6ce4XuUnyDXuSmVO2ZCBSbKI9j8\n\
VQIDAQAB\n\
-----END PUBLIC KEY-----\n";

pub fn stored_client() -> Client {
    let jwks = PublicJwks {
        keys: vec![PublicJwk {
            kty: "OKP".into(),
            kid: "workload-key".into(),
            alg: "EdDSA".into(),
            usage: Some("sig".into()),
            key_ops: vec!["verify".into()],
            n: None,
            e: None,
            crv: Some("Ed25519".into()),
            x: Some("uu8qb9E3_CijMPu6SWZQaASIiVQhe55YoL24WZ94Cd0".into()),
            y: None,
        }],
    };
    Client {
        id: "config-boundary".into(),
        name: "Configuration boundary client".into(),
        secret_hash: Some("stored-hash-fixture".into()),
        redirect_uris: Vec::new(),
        scopes: BTreeSet::from(["read".into()]),
        allowed_groups: BTreeSet::new(),
        require_mfa: false,
        enabled: true,
        service: true,
        settings: ProviderSettings {
            token_endpoint_auth_method: Some(ClientAuthMethod::ClientSecretBasic),
            jwks: Some(jwks.clone()),
            machine_trust: vec![MachineTrust {
                issuer: "https://workload.example".into(),
                subject: "worker".into(),
                jwks,
                scopes: BTreeSet::from(["read".into()]),
            }],
            exchange: Some(ExchangePolicy {
                subject_clients: BTreeSet::from(["subject-app".into()]),
                target_clients: BTreeSet::from(["target-app".into()]),
                scopes: BTreeSet::from(["read".into()]),
                allow_impersonation: true,
                allow_delegation: false,
            }),
            id_token_encryption: Some(EncryptionKey {
                content_encryption: "A256GCM".into(),
                kid: "rsa-encryption".into(),
                public_key_pem: ENCRYPTION_PUBLIC_KEY.into(),
            }),
            ..Default::default()
        },
    }
}
