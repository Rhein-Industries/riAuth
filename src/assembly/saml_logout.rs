//! Platform SAML logout transaction and read ports.

use crate::{
    core::{Core, audit},
    crypto,
    error::Result,
    model::{Client, Session},
    saml::{
        Reply, RpSession, Settings,
        logout::{self, Flow, Peer, SamlLogoutCore, SamlLogoutTx},
    },
    source::{Source, saml::UpstreamSession},
    store::Tx,
};
use serde_json::Value;

impl SamlLogoutCore for Core {
    fn logout_issuer(&self) -> &str {
        &self.config.issuer
    }

    fn client_issuer(&self, settings: &Settings, client: &Client) -> String {
        settings.issuer(self, client)
    }
}

impl SamlLogoutTx for Tx<'_> {
    fn session_record(&self, id: &str) -> Result<Option<Session>> {
        self.get("sessions", id)
    }

    fn rp_sessions(&self) -> Result<Vec<(String, RpSession)>> {
        self.list("saml_sessions")
    }

    fn upstream_session(&self, id: &str) -> Result<Option<UpstreamSession>> {
        self.get("saml_source_sessions", id)
    }

    fn upstream_sessions(&self) -> Result<Vec<(String, UpstreamSession)>> {
        self.list("saml_source_sessions")
    }

    fn source_record(&self, id: &str) -> Result<Option<Source>> {
        self.get("sources", id)
    }

    fn source_key(&self, source: &Source) -> Result<crypto::SigningKey> {
        source.saml.as_ref().unwrap().key(self)
    }

    fn flow_record(&self, id: &str) -> Result<Option<Flow>> {
        self.get("saml_logout_flows", id)
    }

    fn flows(&self) -> Result<Vec<(String, Flow)>> {
        self.list("saml_logout_flows")
    }

    fn put_flow(&self, id: &str, flow: &Flow) -> Result<()> {
        self.put("saml_logout_flows", id, flow)
    }

    fn flow_page(&self) -> Result<Vec<(String, Flow)>> {
        self.maintenance_page("saml_logout_flows")
    }

    fn delete_flow(&self, id: &str) -> Result<()> {
        self.delete("saml_logout_flows", id)
    }

    fn session_ticket(&self, id: &str) -> Result<Option<String>> {
        self.get("saml_logout_sessions", id)
    }

    fn put_session_ticket(&self, id: &str, ticket: &str) -> Result<()> {
        self.put("saml_logout_sessions", id, &ticket)
    }

    fn ticket_page(&self) -> Result<Vec<(String, String)>> {
        self.maintenance_page("saml_logout_sessions")
    }

    fn delete_session_ticket(&self, id: &str) -> Result<()> {
        self.delete("saml_logout_sessions", id)
    }

    fn replay_record(&self, id: &str) -> Result<Option<u64>> {
        self.get("saml_replays", id)
    }

    fn replay_count(&self) -> Result<usize> {
        Ok(self.list::<u64>("saml_replays")?.len())
    }

    fn put_replay(&self, id: &str, expires_at: u64) -> Result<()> {
        self.put("saml_replays", id, &expires_at)
    }

    fn revoke_source_session(
        &self,
        id: &str,
        mut session: Session,
        issuer: &str,
        source_id: &str,
    ) -> Result<Vec<String>> {
        let fronts = crate::session_protocol::frontchannel_urls(self, id, issuer)?;
        session.revoked = true;
        self.put("sessions", id, &session)?;
        crate::logout::queue_session(self, id)?;
        crate::ssf::enqueue(
            self,
            &session.identity.user_id,
            crate::ssf::SESSION_REVOKED,
            "",
        )?;
        audit(
            self,
            &session.identity.user_id,
            "saml.source.logout",
            source_id,
        )?;
        Ok(fronts)
    }

    fn audit_confirmation(&self, peer_id: &str) -> Result<()> {
        audit(self, "saml-peer", "saml.logout.confirmation", peer_id)
    }
}

impl Core {
    pub fn saml_logout_status(&self, ticket: &str) -> Result<Value> {
        self.store.read(|tx| logout::status(self, tx, ticket))
    }

    pub fn saml_logout_next(&self, ticket: &str) -> Result<Reply> {
        self.store.write(|tx| logout::next(self, tx, ticket))
    }

    pub(crate) fn saml_logout_response_in(
        &self,
        tx: &Tx<'_>,
        peer: Peer,
        raw: &str,
        post: bool,
    ) -> Result<Reply> {
        logout::response_in(self, tx, peer, raw, post)
    }

    pub fn saml_source_logout(&self, id: &str, raw: &str, post: bool) -> Result<Reply> {
        self.store
            .write(|tx| logout::source_logout(self, tx, id, raw, post))
    }
}
