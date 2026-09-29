//! Browser SLO coordinator. Revocation commits before any participant is contacted.
//! Tickets can only finish cleanup of sessions that have already been revoked.
use super::{
    Reply, RpSession, SamlTx, Settings,
    wire::{self, ASSERTION, PROTOCOL},
};
use crate::{
    crypto::{self, SigningKey, digest, now},
    error::{Error, Result},
    model::{Client, Session},
    response::escape,
    session_protocol::PostLogoutReturn,
    source::Source,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// The caller supplies one transaction for replay checks, local revocation and
/// fanout persistence. Assembly maps these operations to concrete collections.
pub(crate) trait SamlLogoutTx: SamlTx {
    fn session_record(&self, id: &str) -> Result<Option<Session>>;
    fn rp_sessions(&self) -> Result<Vec<(String, RpSession)>>;
    fn upstream_session(&self, id: &str) -> Result<Option<crate::source::saml::UpstreamSession>>;
    fn upstream_sessions(&self) -> Result<Vec<(String, crate::source::saml::UpstreamSession)>>;
    fn source_record(&self, id: &str) -> Result<Option<Source>>;
    fn source_key(&self, source: &Source) -> Result<SigningKey>;
    fn flow_record(&self, id: &str) -> Result<Option<Flow>>;
    fn flows(&self) -> Result<Vec<(String, Flow)>>;
    fn put_flow(&self, id: &str, flow: &Flow) -> Result<()>;
    fn flow_page(&self) -> Result<Vec<(String, Flow)>>;
    fn delete_flow(&self, id: &str) -> Result<()>;
    fn session_ticket(&self, id: &str) -> Result<Option<String>>;
    fn put_session_ticket(&self, id: &str, ticket: &str) -> Result<()>;
    fn ticket_page(&self) -> Result<Vec<(String, String)>>;
    fn delete_session_ticket(&self, id: &str) -> Result<()>;
    fn replay_record(&self, id: &str) -> Result<Option<u64>>;
    fn replay_count(&self) -> Result<usize>;
    fn put_replay(&self, id: &str, expires_at: u64) -> Result<()>;
    fn revoke_source_session(
        &self,
        id: &str,
        session: Session,
        issuer: &str,
        source_id: &str,
    ) -> Result<Vec<String>>;
    fn audit_confirmation(&self, peer_id: &str) -> Result<()>;
}

pub(crate) trait SamlLogoutCore {
    fn logout_issuer(&self) -> &str;
    fn client_issuer(&self, settings: &Settings, client: &Client) -> String;
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Peer {
    Client { id: String, fingerprint: String },
    Source { id: String, fingerprint: String },
}
impl Peer {
    fn id(&self) -> &str {
        match self {
            Self::Client { id, .. } | Self::Source { id, .. } => id,
        }
    }
    fn same_entity(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other) && self.id() == other.id()
    }
    fn current(&self, core: &impl SamlLogoutCore, tx: &impl SamlLogoutTx) -> Result<Current> {
        match self {
            Self::Client { id, fingerprint } => {
                let (client, settings) = super::client(tx, id)?;
                if *fingerprint != super::fingerprint(&client)? {
                    return Err(Error::conflict("SAML logout client changed"));
                }
                super::validate_key(tx, &client)?;
                let own = core.client_issuer(&settings, &client);
                Ok(Current {
                    issuer: own.clone(),
                    peer: settings.sp_entity_id.clone(),
                    name_qualifier: own,
                    sp_qualifier: settings.sp_entity_id,
                    callback: format!(
                        "{}/saml/{id}/sso",
                        super::endpoint_base(core.logout_issuer())
                    ),
                    redirect: settings.slo_redirect_url,
                    post: settings.slo_post_url,
                    certificate: settings.idp_certificate_pem,
                    trusted: settings.sp_certificates_pem,
                    key: tx.signing_key(&client)?,
                })
            }
            Self::Source { id, fingerprint } => {
                let source = tx
                    .source_record(id)?
                    .filter(|s| s.enabled)
                    .ok_or_else(Error::forbidden)?;
                if *fingerprint != source.fingerprint()? {
                    return Err(Error::conflict("SAML logout source changed"));
                }
                let settings = source.saml.as_ref().ok_or_else(Error::forbidden)?;
                settings.validate(&source)?;
                Ok(Current {
                    issuer: source.client_id.clone(),
                    peer: source.issuer.clone(),
                    name_qualifier: source.issuer.clone(),
                    sp_qualifier: source.client_id.clone(),
                    callback: format!(
                        "{}/saml/sources/{id}/slo",
                        super::endpoint_base(core.logout_issuer())
                    ),
                    redirect: settings.slo_redirect_url.clone(),
                    post: settings.slo_post_url.clone(),
                    certificate: settings.sp_certificate_pem.clone(),
                    trusted: settings.idp_certificates_pem.clone(),
                    key: tx.source_key(&source)?,
                })
            }
        }
    }
}
struct Current {
    issuer: String,
    peer: String,
    name_qualifier: String,
    sp_qualifier: String,
    callback: String,
    redirect: Option<String>,
    post: Option<String>,
    certificate: String,
    trusted: Vec<String>,
    key: SigningKey,
}
impl Current {
    fn endpoint(&self, post: bool) -> Result<&str> {
        (if post { &self.post } else { &self.redirect })
            .as_deref()
            .ok_or_else(|| Error::bad("SAML logout binding is not registered"))
    }
    fn message(&self, xml: &str, relay: Option<&str>, post: bool, field: &str) -> Result<Reply> {
        let endpoint = self.endpoint(post)?;
        if post {
            let signed = super::sign(xml, &self.key, &self.certificate, true)?;
            let mut fields = vec![(field.into(), STANDARD.encode(signed))];
            if let Some(relay) = relay {
                fields.push(("RelayState".into(), relay.into()));
            }
            Ok(Reply::Post {
                target: endpoint.into(),
                fields,
                cookies: vec![],
            })
        } else {
            Ok(Reply::Redirect(wire::redirect_message(
                endpoint,
                xml,
                relay,
                &self.key.pem,
                field,
            )?))
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Target {
    peer: Peer,
    name_id: String,
    format: String,
    index: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ReturnResponse {
    pub peer: Peer,
    pub request: wire::Logout,
    pub post: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct Finish {
    pub redirect: Option<String>,
    /// Exact OIDC registration for an external return. Old flows lack it and
    /// cannot safely deliver their cached redirect.
    #[serde(default)]
    pub return_binding: Option<PostLogoutReturn>,
    pub response: Option<ReturnResponse>,
    pub frontchannel_urls: BTreeSet<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Pending {
    id: String,
    xml: String,
    post: bool,
    deadline: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Flow {
    id: String,
    expires_at: u64,
    targets: Vec<Target>,
    position: usize,
    pending: Option<Pending>,
    confirmed: usize,
    failed: usize,
    finish: Finish,
}
impl Flow {
    fn status(&self, core: &impl SamlLogoutCore) -> Value {
        let completed = self.position == self.targets.len();
        json!({"protocol":"saml","logged_out":true,"status":if completed {if self.failed==0 {"complete"} else {"partial_logout"}}else if self.expires_at<=now(){"expired"}else{"pending"},"confirmed":self.confirmed,"failed":self.failed,"remaining":self.targets.len()-self.position,"expires_at":self.expires_at,"resume_uri":flow_url(core,&self.id),"retry_after":self.pending.as_ref().map(|p|p.deadline.saturating_sub(now()))})
    }
}
fn flow_url(core: &impl SamlLogoutCore, id: &str) -> String {
    format!(
        "{}/saml/logout/{id}",
        super::endpoint_base(core.logout_issuer())
    )
}

/// Only a stored ticket at our canonical SAML logout URL is an internal
/// continuation. The confirmation may keep this URL when its external return
/// registration has been removed.
pub(crate) fn is_continuation(
    core: &impl SamlLogoutCore,
    tx: &impl SamlLogoutTx,
    sid: &str,
    uri: &str,
) -> Result<bool> {
    let prefix = format!("{}/saml/logout/", super::endpoint_base(core.logout_issuer()));
    let Some(ticket) = uri.strip_prefix(&prefix) else {
        return Ok(false);
    };
    if ticket.len() != 43
        || !ticket
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Ok(false);
    }
    if tx.session_ticket(sid)?.as_deref() != Some(ticket) {
        return Ok(false);
    }
    Ok(tx.flow_record(&digest(ticket))?.is_some_and(|flow| {
        flow.id == ticket
            && flow.expires_at.saturating_add(86400) > now()
            && uri == flow_url(core, &flow.id)
    }))
}

// The optional response sender is excluded, preventing source/SP logout loops.
pub(crate) fn begin(
    core: &impl SamlLogoutCore,
    tx: &impl SamlLogoutTx,
    session_ids: &BTreeSet<String>,
    finish: Finish,
) -> Result<Reply> {
    let exclude = finish.response.as_ref().map(|r| &r.peer);
    let mut targets = Vec::new();
    let mut failed = 0;
    for sid in session_ids {
        if tx.session_record(sid)?.is_some_and(|s| !s.revoked) {
            return Err(Error::internal(
                "SAML propagation requires local revocation",
            ));
        }
    }
    for (_, rp) in tx.rp_sessions()? {
        if !session_ids.contains(&rp.identity.session_id) || rp.expires_at <= now() {
            continue;
        }
        let peer = Peer::Client {
            id: rp.client_id,
            fingerprint: rp.fingerprint,
        };
        if exclude.is_some_and(|p| p.same_entity(&peer)) {
            continue;
        }
        if let Some(index) = rp.index {
            targets.push(Target {
                peer,
                name_id: rp.name_id,
                format: rp.format.uri().into(),
                index,
            });
        } else {
            failed += 1;
        }
    }
    for sid in session_ids {
        let Some(session) = tx.session_record(sid)? else {
            continue;
        };
        let Some(context) = &session.identity.source else {
            continue;
        };
        let Some(upstream) = tx.upstream_session(sid)? else {
            continue;
        };
        if upstream.expires_at.is_some_and(|at| at <= now()) {
            continue;
        }
        let peer = Peer::Source {
            id: context.id.clone(),
            fingerprint: context.fingerprint.clone(),
        };
        if exclude.is_some_and(|p| p.same_entity(&peer)) {
            continue;
        }
        let source = tx.source_record(&context.id)?;
        let settings = source.as_ref().and_then(|s| s.saml.as_ref());
        if let (Some(subject), Some(settings)) = (upstream.subject, settings) {
            targets.push(Target {
                peer,
                name_id: subject,
                format: settings.name_id_format.uri().into(),
                index: upstream.index,
            });
        } else {
            failed += 1;
        }
    }
    let mut unique = BTreeSet::new();
    let mut deduplicated = Vec::new();
    for target in targets {
        if unique.insert(serde_json::to_string(&target).map_err(Error::internal)?) {
            deduplicated.push(target);
        }
    }
    let mut targets = deduplicated;
    if targets.len() > 64 {
        failed += targets.len() - 64;
        targets.truncate(64);
    }
    let mut flow = Flow {
        id: crypto::random_token(""),
        expires_at: now() + 600,
        targets,
        position: 0,
        pending: None,
        confirmed: 0,
        failed,
        finish,
    };
    if flow.targets.is_empty() && flow.finish.frontchannel_urls.is_empty() {
        return finish_reply(core, tx, &flow);
    }
    cleanup(tx, now())?;
    if tx.flows()?.len() >= 5000 {
        // Reaching an outbox limit cannot roll back the already requested logout.
        flow.failed += flow.targets.len();
        flow.position = flow.targets.len();
        return finish_reply(core, tx, &flow);
    }
    tx.put_flow(&digest(&flow.id), &flow)?;
    Ok(Reply::LogoutPage(
        json!({"logged_out":true,"redirect_uri":flow_url(core,&flow.id),"frontchannel_urls":flow.finish.frontchannel_urls}),
    ))
}
pub(crate) fn redirect(
    core: &impl SamlLogoutCore,
    tx: &impl SamlLogoutTx,
    sid: &str,
    original: Option<PostLogoutReturn>,
) -> Result<Value> {
    if let Some(ticket) = tx.session_ticket(sid)?
        && lookup(tx, &ticket).is_ok()
    {
        return Ok(json!({"redirect_uri":flow_url(core,&ticket)}));
    }
    let reply = begin(
        core,
        tx,
        &BTreeSet::from([sid.into()]),
        Finish {
            redirect: original.as_ref().map(|target| target.rendered_uri.clone()),
            return_binding: original,
            ..Default::default()
        },
    )?;
    let value = match reply {
        Reply::Redirect(uri) => json!({"redirect_uri":uri}),
        Reply::LogoutPage(value) => value,
        _ => return Err(Error::internal("Unexpected SAML logout continuation")),
    };
    if let Some(ticket) = value["redirect_uri"].as_str().and_then(|u| {
        u.strip_prefix(&format!(
            "{}/saml/logout/",
            super::endpoint_base(core.logout_issuer())
        ))
    }) && ticket.len() == 43
        && lookup(tx, ticket).is_ok()
    {
        tx.put_session_ticket(sid, ticket)?;
    }
    Ok(value)
}
fn finish_reply(core: &impl SamlLogoutCore, tx: &impl SamlLogoutTx, flow: &Flow) -> Result<Reply> {
    if let Some(end) = &flow.finish.response {
        let current = end.peer.current(core, tx)?;
        let target = current.endpoint(end.post)?;
        let partial = if flow.failed > 0 {
            "<samlp:StatusCode Value=\"urn:oasis:names:tc:SAML:2.0:status:PartialLogout\"/>"
        } else {
            ""
        };
        let xml = format!(
            r#"<samlp:LogoutResponse xmlns:samlp="{PROTOCOL}" xmlns:saml="{ASSERTION}" ID="_{}" Version="2.0" IssueInstant="{}" Destination="{}" InResponseTo="{}"><saml:Issuer>{}</saml:Issuer><samlp:Status><samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success">{partial}</samlp:StatusCode></samlp:Status></samlp:LogoutResponse>"#,
            crypto::id(),
            wire::timestamp(now())?,
            escape(target),
            escape(&end.request.id),
            escape(&current.issuer)
        );
        current.message(
            &xml,
            end.request.relay_state.as_deref(),
            end.post,
            "SAMLResponse",
        )
    } else if let Some(binding) = &flow.finish.return_binding
        && flow.finish.redirect.as_deref() == Some(binding.rendered_uri.as_str())
        && binding.allowed_by(tx.client_record(&binding.client_id)?.as_ref())
    {
        Ok(Reply::Redirect(binding.rendered_uri.clone()))
    } else {
        Ok(Reply::LogoutPage(flow.status(core)))
    }
}
fn lookup(tx: &impl SamlLogoutTx, ticket: &str) -> Result<Flow> {
    if ticket.len() != 43
        || !ticket
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err(Error::missing("SAML logout not found"));
    }
    tx.flow_record(&digest(ticket))?
        .filter(|f| f.expires_at + 86400 > now())
        .ok_or_else(|| Error::missing("SAML logout expired"))
}
pub(crate) fn status(
    core: &impl SamlLogoutCore,
    tx: &impl SamlLogoutTx,
    ticket: &str,
) -> Result<Value> {
    Ok(lookup(tx, ticket)?.status(core))
}

pub(crate) fn next(
    core: &impl SamlLogoutCore,
    tx: &impl SamlLogoutTx,
    ticket: &str,
) -> Result<Reply> {
    let mut flow = lookup(tx, ticket)?;
    while flow.position < flow.targets.len() {
        if flow.expires_at <= now() {
            flow.failed += flow.targets.len() - flow.position;
            flow.position = flow.targets.len();
            flow.pending = None;
            break;
        }
        let target = &flow.targets[flow.position];
        let current = target.peer.current(core, tx);
        if current.is_err() || flow.pending.as_ref().is_some_and(|p| p.deadline <= now()) {
            flow.failed += 1;
            flow.position += 1;
            flow.pending = None;
            continue;
        }
        let current = current?;
        if flow.pending.is_none() {
            let post = current.redirect.is_none();
            let Ok(endpoint) = current.endpoint(post) else {
                flow.failed += 1;
                flow.position += 1;
                continue;
            };
            let id = format!("_{}", crypto::id());
            let xml = format!(
                r#"<samlp:LogoutRequest xmlns:samlp="{PROTOCOL}" xmlns:saml="{ASSERTION}" ID="{id}" Version="2.0" IssueInstant="{}" Destination="{}" NotOnOrAfter="{}" Reason="urn:oasis:names:tc:SAML:2.0:logout:user"><saml:Issuer>{}</saml:Issuer><saml:NameID Format="{}" NameQualifier="{}" SPNameQualifier="{}">{}</saml:NameID><samlp:SessionIndex>{}</samlp:SessionIndex></samlp:LogoutRequest>"#,
                wire::timestamp(now())?,
                escape(endpoint),
                wire::timestamp((now() + 300).min(flow.expires_at))?,
                escape(&current.issuer),
                escape(&target.format),
                escape(&current.name_qualifier),
                escape(&current.sp_qualifier),
                escape(&target.name_id),
                escape(&target.index)
            );
            flow.pending = Some(Pending {
                id,
                xml,
                post,
                deadline: (now() + 30).min(flow.expires_at),
            });
        }
        let pending = flow.pending.as_ref().unwrap();
        match current.message(&pending.xml, Some(&flow.id), pending.post, "SAMLRequest") {
            Ok(reply) => {
                tx.put_flow(&digest(ticket), &flow)?;
                return Ok(reply);
            }
            Err(_) => {
                flow.failed += 1;
                flow.position += 1;
                flow.pending = None;
            }
        }
    }
    tx.put_flow(&digest(ticket), &flow)?;
    finish_reply(core, tx, &flow)
}

pub(crate) fn response_in(
    core: &impl SamlLogoutCore,
    tx: &impl SamlLogoutTx,
    peer: Peer,
    raw: &str,
    post: bool,
) -> Result<Reply> {
    let current = peer.current(core, tx)?;
    let response = wire::receive_logout_response(
        &current.trusted,
        &current.callback,
        &current.peer,
        raw,
        post,
    )?;
    let mut flow = lookup(tx, &response.relay_state)?;
    let pending = flow
        .pending
        .as_ref()
        .ok_or_else(|| Error::conflict("SAML logout response already consumed"))?;
    if flow.expires_at <= now()
        || pending.deadline <= now()
        || pending.id != response.request
        || flow
            .targets
            .get(flow.position)
            .is_none_or(|t| t.peer != peer)
    {
        return Err(Error::forbidden());
    }
    if response.success {
        flow.confirmed += 1;
    } else {
        flow.failed += 1;
    }
    flow.position += 1;
    flow.pending = None;
    tx.put_flow(&digest(&flow.id), &flow)?;
    tx.audit_confirmation(peer.id())?;
    Ok(Reply::Redirect(flow_url(core, &flow.id)))
}

pub(crate) fn source_logout(
    core: &impl SamlLogoutCore,
    tx: &impl SamlLogoutTx,
    id: &str,
    raw: &str,
    post: bool,
) -> Result<Reply> {
    let source = tx
        .source_record(id)?
        .filter(|s| s.enabled)
        .ok_or_else(Error::forbidden)?;
    let fingerprint = source.fingerprint()?;
    let peer = Peer::Source {
        id: id.into(),
        fingerprint: fingerprint.clone(),
    };
    let current = peer.current(core, tx)?;
    if wire::is_response(raw) {
        return response_in(core, tx, peer, raw, post);
    }
    current.endpoint(post)?;
    let request = wire::receive_logout(
        &current.trusted,
        &current.callback,
        &current.peer,
        &current.name_qualifier,
        &current.sp_qualifier,
        raw,
        post,
    )?;
    let replay = digest(&format!("source/{id}\0{}", request.id));
    if tx.replay_record(&replay)?.is_some_and(|e| e > now()) {
        return Err(Error::conflict("SAML logout request replay"));
    }
    let format = source.saml.as_ref().unwrap().name_id_format.uri();
    if request.format.as_deref().is_some_and(|f| f != format) {
        return Err(Error::forbidden());
    }
    let mut sessions = BTreeSet::new();
    let mut found = BTreeSet::new();
    for (sid, upstream) in tx.upstream_sessions()? {
        if upstream.subject.as_deref() != Some(&request.name_id)
            || !request.indices.contains(&upstream.index)
            || upstream.expires_at.is_some_and(|at| at <= now())
        {
            continue;
        }
        // A pinned-certificate change revokes this row and leaves its previous
        // fingerprint. The index still names that source session, so a verified
        // request is consumed when the row is already revoked.
        if let Some(session) = tx.session_record(&sid)?
            && session.expires_at > now()
            && session.identity.source.as_ref().is_some_and(|linked| {
                linked.id == id && (linked.fingerprint == fingerprint || session.revoked)
            })
        {
            sessions.insert(sid);
            found.insert(upstream.index);
        }
    }
    if found.len() != request.indices.len() {
        return Err(Error::forbidden());
    }
    if tx.replay_count()? >= 20000 {
        return Err(Error::conflict("Too many SAML requests"));
    }
    tx.put_replay(&replay, now() + 630)?;
    let mut fronts = BTreeSet::new();
    for sid in &sessions {
        if let Some(session) = tx.session_record(sid)? {
            fronts.extend(tx.revoke_source_session(sid, session, core.logout_issuer(), id)?);
        }
    }
    begin(
        core,
        tx,
        &sessions,
        Finish {
            redirect: None,
            return_binding: None,
            response: Some(ReturnResponse {
                peer,
                request,
                post,
            }),
            frontchannel_urls: fronts,
        },
    )
}

pub(crate) fn cleanup(tx: &impl SamlLogoutTx, at: u64) -> Result<()> {
    for (id, flow) in tx.flow_page()? {
        if flow.expires_at + 86400 <= at {
            tx.delete_flow(&id)?;
        }
    }
    for (sid, ticket) in tx.ticket_page()? {
        if tx.flow_record(&digest(&ticket))?.is_none() {
            tx.delete_session_ticket(&sid)?;
        }
    }
    Ok(())
}
