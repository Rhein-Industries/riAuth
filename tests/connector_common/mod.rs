//! Shared operator setup for the connector definition tests.
#![allow(dead_code)]

use std::collections::BTreeMap;

/// Every receiving origin the tests send a credential to.
pub const ORIGINS: &[&str] = &[
    "ldaps://a.example.test",
    "ldaps://b.example.test",
    "ldaps://fresh.example.test",
    "ldaps://ldap.example.test",
    "ldap://ldap.example.test",
    "ldaps://moved.example.test",
    "ldaps://other.example.test",
    "ldaps://attacker.example.test",
    "https://scim.example.test",
    "https://attacker.example.test",
    "https://idp.example.test",
    "https://broker.example.test",
    "https://admin.googleapis.com",
    "https://oauth2.googleapis.com",
    "https://graph.microsoft.com",
    "https://login.microsoftonline.com",
];

/// The credential file names the tests use.
pub const NAMES: &[&str] = &[
    "ldap/a",
    "ldap/b",
    "ldap/bind",
    "ldap/legacy-bind",
    "ldap/moved-bind",
    "ldap/other",
    "ldap/shared",
    "ldap/toml-owned",
    "ldap/x",
    "ldap/y",
    "ldap/z",
    "scim/client-secret",
    "scim/refresh-new",
    "scim/refresh-y",
    "scim/refresh-z",
    "scim/secret",
    "scim/secret-new",
    "scim/secret-w",
    "scim/secret-x",
    "scim/token",
    "workspace/secret",
    "workspace/new-secret",
    "workspace/key",
    "entra/secret",
];

/// An operator who pinned every name to every origin: the pin layer passes and
/// the binding layer is what the tests exercise.
pub fn wide_pins() -> BTreeMap<String, String> {
    NAMES
        .iter()
        .map(|name| ((*name).to_owned(), ORIGINS.join(",")))
        .collect()
}
