//! LDAP listener Core bindings. Protocol parsing and response handling stay in ldap_server.

use crate::{
    core::Core,
    error::Result,
    ldap_server::{Auth, LdapPort, Listener, Servers, Settings},
};
use ldap3_proto::proto::{LdapSearchRequest, LdapSearchResultEntry};
use serde_json::Value;
use std::{collections::BTreeMap, net::IpAddr};

pub async fn ldap_start(core: Core) -> anyhow::Result<Servers> {
    crate::ldap_server::start_with_port(core).await
}

impl LdapPort for Core {
    fn listeners(&self) -> &BTreeMap<String, Listener> {
        &self.config.ldap_listeners
    }

    fn bind_listener(&self, id: &str, listener: &Listener) -> crate::capability::ListenerLease {
        self.runtime.bind_ldap(id, listener)
    }

    fn ldap_rate_limit(&self, peer: IpAddr, category: &str) -> Result<bool> {
        Core::ldap_rate_limit(self, peer, category)
    }

    fn ldap_bind_target(
        &self,
        cid: &str,
        dn: &str,
        password: &str,
    ) -> Result<(Settings, Option<(String, bool)>)> {
        Core::ldap_bind_target(self, cid, dn, password)
    }

    fn login(&self, username: String, password: String, otp: Option<String>) -> Result<Value> {
        Core::login(self, username, password, otp)
    }

    fn ldap_bind_authorized(&self, cid: &str, token: &str) -> Result<()> {
        Core::ldap_bind_authorized(self, cid, token)
    }

    fn logout(&self, token: &str) -> Result<Value> {
        Core::logout(self, token)
    }

    fn ldap_whoami(&self, cid: &str, auth: Option<&Auth>) -> Result<String> {
        Core::ldap_whoami(self, cid, auth)
    }

    fn ldap_search_entries(
        &self,
        cid: &str,
        auth: Option<&Auth>,
        query: &LdapSearchRequest,
        starttls: bool,
    ) -> Result<(Vec<LdapSearchResultEntry>, u64)> {
        Core::ldap_search_entries(self, cid, auth, query, starttls)
    }
}
