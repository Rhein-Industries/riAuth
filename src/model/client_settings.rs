//! Stored client protocol settings shared with the identity model.
//!
//! These are data definitions only. Each adapter retains its existing
//! validation and runtime behavior, and re-exports the former public paths.

pub mod portal {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeSet;

    #[derive(
        schemars::JsonSchema, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq,
    )]
    #[serde(default, deny_unknown_fields)]
    pub struct Settings {
        pub description: String,
        pub category: String,
        /// Application home/login URL, never an OAuth callback URL.
        pub launch_url: Option<String>,
        pub hidden: bool,
        /// Built-in icon; no remote image requests are made by the portal.
        pub icon: String,
        pub accent: String,
        /// Scopes the application's login requests, in addition to protocol minimums.
        pub launch_scopes: BTreeSet<String>,
    }
}

pub mod saml {
    use serde::{Deserialize, Serialize};

    #[derive(
        schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default,
    )]
    #[serde(rename_all = "snake_case")]
    pub enum NameIdFormat {
        #[default]
        Persistent,
        Transient,
        Email,
        Unspecified,
    }
    #[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Attribute {
        pub name: String,
        pub claim: String,
        pub friendly_name: Option<String>,
        #[serde(default)]
        pub required: bool,
    }
    #[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Settings {
        pub sp_entity_id: String,
        pub acs_urls: Vec<String>,
        #[serde(default)]
        pub acs_indices: std::collections::BTreeMap<u16, String>,
        pub idp_entity_id: Option<String>,
        pub idp_certificate_pem: String,
        pub sp_certificates_pem: Vec<String>,
        pub encryption_certificate_pem: Option<String>,
        #[serde(default)]
        pub name_id_format: NameIdFormat,
        #[serde(default)]
        pub attributes: Vec<Attribute>,
        pub slo_redirect_url: Option<String>,
        pub slo_post_url: Option<String>,
        #[serde(default)]
        pub idp_initiated: bool,
        pub default_relay_state: Option<String>,
        #[serde(default = "assertion_ttl")]
        pub assertion_ttl: u64,
    }
    fn assertion_ttl() -> u64 {
        120
    }
}

pub mod radius {
    use serde::{Deserialize, Serialize};
    use std::net::Ipv4Addr;

    #[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(tag = "type", content = "value", rename_all = "snake_case")]
    pub enum ReplyValue {
        Text(String),
        Integer(u32),
        Ipv4(Ipv4Addr),
        Attribute(String),
    }
    #[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    pub enum Attribute {
        Standard {
            code: u8,
            value: ReplyValue,
        },
        Vendor {
            vendor: u32,
            code: u8,
            value: ReplyValue,
        },
    }
    #[derive(
        schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default,
    )]
    #[serde(deny_unknown_fields)]
    pub struct Settings {
        #[serde(default)]
        pub eap_tls: bool,
        #[serde(default)]
        pub reply: Vec<Attribute>,
    }
}

pub mod ldap {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeSet;

    #[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Settings {
        pub base_dn: String,
        pub search_groups: BTreeSet<String>,
    }
}

pub mod proxy {
    use serde::{Deserialize, Serialize};

    #[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Settings {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub domain: Option<Domain>,
        pub external_origin: String,
        #[serde(default = "default_ttl")]
        pub session_ttl: u64,
    }
    #[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Domain {
        pub cookie_domain: String,
        pub application_origins: std::collections::BTreeSet<String>,
    }
    fn default_ttl() -> u64 {
        3600
    }
}
