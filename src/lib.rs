#![recursion_limit = "256"]
#[cfg(not(feature = "essentials"))]
compile_error!("Select --features essentials or --features platform for a server build");
pub mod agent;
pub mod api;
pub mod application_setup;
mod assembly;
pub mod assurance;
pub mod authenticator;
pub mod authorization;
mod background;
pub mod bootstrap;
pub mod browser;
pub mod capability;
pub mod claims;
pub mod cli;
#[cfg(feature = "platform")]
pub mod cloud_directory;
#[cfg(not(feature = "platform"))]
#[path = "cloud_directory_essentials.rs"]
pub mod cloud_directory;
mod cloud_directory_types;
#[cfg(feature = "platform")]
mod cloud_operations;
pub mod config;
pub mod connector_guard;
pub mod context;
pub mod core;
pub mod crypto;
pub mod delegation;
#[cfg(feature = "platform")]
pub mod device_trust;
#[cfg(not(feature = "platform"))]
#[path = "device_trust_essentials.rs"]
pub mod device_trust;
mod device_trust_types;
pub mod directory;
pub mod dpop;
pub mod edition;
pub mod encryption;
pub mod error;
#[cfg(feature = "platform")]
pub mod event_map;
pub mod exchange;
pub mod identity;
pub mod issuer;
pub mod jose;
pub mod keyring;
#[cfg(feature = "platform")]
pub mod kms;
#[cfg(not(feature = "platform"))]
#[path = "kms_essentials.rs"]
pub mod kms;
mod kms_types;
mod ldap_listener;
#[cfg(feature = "platform")]
pub mod ldap_server;
#[cfg(not(feature = "platform"))]
#[path = "ldap_server_essentials.rs"]
pub mod ldap_server;
pub mod lifecycle;
pub mod logout;
pub(crate) mod management;
pub mod migration;
pub mod model;
#[cfg(feature = "platform")]
pub mod mtls;
#[cfg(not(feature = "platform"))]
#[path = "mtls_essentials.rs"]
pub mod mtls;
mod mtls_config;
pub mod node_security;
#[cfg(feature = "platform")]
pub mod offboarding;
#[cfg(not(feature = "platform"))]
#[path = "offboarding_essentials.rs"]
pub mod offboarding;
mod offboarding_types;
pub mod oidc;
pub mod operations;
#[cfg(feature = "platform")]
pub mod outpost;
#[cfg(not(feature = "platform"))]
#[path = "outpost_essentials.rs"]
pub mod outpost;
#[cfg(feature = "platform")]
pub mod pam;
#[cfg(not(feature = "platform"))]
#[path = "pam_essentials.rs"]
pub mod pam;
mod pam_types;
pub mod passkey;
pub(crate) mod password;
pub mod portal;
pub mod postgres_store;
pub mod process_role;
pub mod provider;
pub mod provisioning;
mod proxy_listener;
#[cfg(feature = "platform")]
pub mod radius;
#[cfg(not(feature = "platform"))]
#[path = "radius_essentials.rs"]
pub mod radius;
mod radius_eap_types;
mod radius_listener;
pub mod reconciliation;
pub mod recovery;
pub mod registration;
pub mod reports;
pub mod resource;
pub mod response;
#[cfg(feature = "platform")]
pub mod saml;
#[cfg(not(feature = "platform"))]
#[path = "saml_essentials.rs"]
pub mod saml;
pub mod schema;
#[cfg(feature = "platform")]
pub mod scim;
#[cfg(not(feature = "platform"))]
#[path = "scim_essentials.rs"]
pub mod scim;
mod scim_shared;
pub mod session_protocol;
pub mod signin;
pub mod source;
#[cfg(feature = "platform")]
pub mod ssf;
#[cfg(not(feature = "platform"))]
#[path = "ssf_essentials.rs"]
pub mod ssf;
pub mod state;
pub mod store;
pub mod telemetry;
pub mod upgrade;
mod user_listing;
mod validation;
#[cfg(feature = "platform")]
pub mod windows_login;
pub mod workflow;

#[cfg(feature = "fuzzing")]
pub mod fuzzing;
#[cfg(feature = "platform")]
pub mod proxy_server;
#[cfg(not(feature = "platform"))]
#[path = "proxy_server_essentials.rs"]
pub mod proxy_server;
