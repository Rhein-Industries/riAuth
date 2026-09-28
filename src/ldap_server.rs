//! A bounded, read-only LDAPv3 provider with TLS and scoped service credentials.
use crate::{
    core::{Core, validate_name},
    crypto::{digest, now},
    error::{Error, Result},
    model::Client,
};
use futures_util::{SinkExt, StreamExt};
use ldap3_proto::{LdapCodec, control::LdapControl, proto::*};
use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    sync::Semaphore,
    task::{JoinHandle, JoinSet},
};
use tokio_util::codec::Framed;
pub(crate) const STARTTLS: &str = "1.3.6.1.4.1.1466.20037";
pub(crate) const WHOAMI: &str = "1.3.6.1.4.1.4203.1.11.3";
pub(crate) const PAGED: &str = "1.2.840.113556.1.4.319";
pub use crate::model::client_settings::ldap::Settings;

// Reuse the listener's BER decoder and limits without creating a connection or
// invoking bind/search handlers. &[u8] is an always-ready Tokio AsyncRead.
#[cfg(feature = "fuzzing")]
pub(crate) fn fuzz_ber(data: &[u8]) -> std::io::Result<Vec<i32>> {
    use futures_util::FutureExt;
    use std::io;
    use tokio_util::codec::FramedRead;

    if data.len() > 65_536 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "LDAP input limit",
        ));
    }
    let mut wire = FramedRead::new(data, LdapCodec::new(Some(32768), Some(16)));
    let mut ids = Vec::new();
    for _ in 0..1000 {
        match wire.next().now_or_never() {
            Some(Some(Ok(message))) => ids.push(message.msgid),
            Some(Some(Err(error))) => return Err(error),
            Some(None) => return Ok(ids),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "LDAP in-memory read",
                ));
            }
        }
    }
    Ok(ids)
}

#[cfg(all(test, feature = "fuzzing"))]
mod ber_tests {
    use super::fuzz_ber;

    #[test]
    fn listener_codec_parses_complete_frames_and_rejects_truncation() {
        let unbind = include_bytes!("../fuzz/corpus/parsers/ldap-unbind");
        let search = include_bytes!("../fuzz/corpus/parsers/ldap-search-present");
        assert_eq!(fuzz_ber(unbind).unwrap(), vec![1]);
        assert_eq!(fuzz_ber(search).unwrap(), vec![2]);
        assert_eq!(
            fuzz_ber(&[unbind.as_slice(), unbind.as_slice()].concat()).unwrap(),
            vec![1, 1]
        );
        assert!(fuzz_ber(&unbind[..unbind.len() - 1]).is_err());
        assert!(
            fuzz_ber(include_bytes!(
                "../fuzz/corpus/parsers/ldap-truncated-unbind"
            ))
            .is_err()
        );
        assert!(
            fuzz_ber(include_bytes!(
                "../fuzz/corpus/parsers/ldap-indefinite-length"
            ))
            .is_err()
        );
        assert!(fuzz_ber(include_bytes!("../fuzz/corpus/parsers/ldap-deep-filter")).is_err());
        assert!(fuzz_ber(&vec![0; 65_537]).is_err());
    }
}
impl Settings {
    pub fn validate(&self, client: &Client) -> Result<()> {
        if self.base_dn.len() > 253
            || !self.base_dn.split(',').all(|part| {
                part.strip_prefix("dc=").is_some_and(|s| {
                    !s.is_empty()
                        && s.len() <= 63
                        && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
                })
            })
            || self.search_groups.is_empty()
            || self.search_groups.len() > 32
            || client.service
            || !client.scopes.contains("profile")
        {
            return Err(Error::bad(
                "LDAP requires a simple dc=... base, one to 32 search groups and an interactive policy client",
            ));
        }
        for group in &self.search_groups {
            validate_name(group)?;
        }
        Ok(())
    }
    pub(crate) fn user_dn(&self, name: &str) -> String {
        format!("uid={name},ou=users,{}", self.base_dn)
    }
    pub(crate) fn group_dn(&self, name: &str) -> String {
        format!("cn={name},ou=groups,{}", self.base_dn)
    }
    pub(crate) fn agent_dn(&self) -> String {
        format!("cn=riauth-agent,{}", self.base_dn)
    }
}
pub use crate::ldap_listener::Listener;

pub struct Servers {
    pub addresses: Vec<SocketAddr>,
    tasks: Vec<JoinHandle<()>>,
    leases: Vec<crate::capability::ListenerLease>,
}
impl Drop for Servers {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}
async fn tls(
    _core: &Core,
    listener: &Listener,
) -> anyhow::Result<Option<Arc<rustls::ServerConfig>>> {
    if listener.local_unencrypted {
        return Ok(None);
    }
    let config = crate::api::tls_files(
        listener.tls_cert_file.clone().unwrap(),
        listener.tls_key_file.clone().unwrap(),
    )
    .await?;
    Ok(Some(Arc::new(config)))
}
pub async fn start(core: Core) -> anyhow::Result<Servers> {
    let mut ready = Vec::new();
    for (id, listener) in &core.config.ldap_listeners {
        listener.validate()?;
        let tls = tls(&core, listener).await?;
        let socket = TcpListener::bind(listener.listen).await?;
        ready.push((id.clone(), socket, listener.clone(), tls));
    }
    let mut servers = Servers {
        addresses: Vec::new(),
        tasks: Vec::new(),
        leases: Vec::new(),
    };
    for (id, socket, config, tls) in ready {
        servers.addresses.push(socket.local_addr()?);
        let core = core.clone();
        let lease = core.runtime.bind_ldap(&id, &config);
        let worker_lease = lease.clone();
        servers.leases.push(lease);
        servers.tasks.push(tokio::spawn(async move {
            let _lease = worker_lease;
            _lease.running();
            listen(core, socket, config, tls).await;
        }));
    }
    Ok(servers)
}
async fn listen(
    core: Core,
    socket: TcpListener,
    config: Listener,
    mut tls_config: Option<Arc<rustls::ServerConfig>>,
) {
    let slots = Arc::new(Semaphore::new(128));
    let mut peers: BTreeMap<IpAddr, Arc<Semaphore>> = BTreeMap::new();
    let mut connections = JoinSet::new();
    let mut reload = tokio::time::interval(Duration::from_secs(60));
    loop {
        tokio::select! {
            accepted=socket.accept()=>{
                let Ok((stream,peer))=accepted else{break};if !config.allowed_peers.contains(&peer.ip()){continue;}
                let Ok(slot)=slots.clone().try_acquire_owned()else{continue};peers.retain(|_,s|s.available_permits()<8||Arc::strong_count(s)>1);let per_peer=peers.entry(peer.ip()).or_insert_with(||Arc::new(Semaphore::new(8))).clone();let Ok(peer_slot)=per_peer.try_acquire_owned()else{continue};
                let core=core.clone();let config=config.clone();let tls=tls_config.clone();connections.spawn(async move{let _slots=(slot,peer_slot);let _=connection(core,stream,peer.ip(),config,tls).await;});
            },
            _=reload.tick(),if !config.local_unencrypted=>{match tls(&core,&config).await{Ok(next)=>tls_config=next,Err(_)=>tracing::warn!("LDAP certificate reload failed; retaining current certificate")}},
            _=connections.join_next(),if !connections.is_empty()=>{},
        }
    }
}
trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}
type Wire = Framed<Box<dyn Io>, LdapCodec>;
#[derive(Clone)]
pub(crate) enum Auth {
    Agent(zeroize::Zeroizing<String>),
    User(zeroize::Zeroizing<String>),
}
#[derive(Default)]
struct Connection {
    auth: Option<Auth>,
    page: Option<Page>,
}
struct Page {
    cookie: Vec<u8>,
    fingerprint: String,
    revision: u64,
    offset: usize,
    expires_at: u64,
}
fn result(code: LdapResultCode) -> LdapResult {
    LdapResult {
        code,
        matcheddn: String::new(),
        message: String::new(),
        referral: Vec::new(),
    }
}
fn msg(id: i32, op: LdapOp) -> LdapMsg {
    LdapMsg {
        msgid: id,
        op,
        ctrl: Vec::new(),
    }
}
async fn send(wire: &mut Wire, message: LdapMsg) -> anyhow::Result<()> {
    tokio::time::timeout(Duration::from_secs(5), wire.send(message)).await??;
    Ok(())
}
fn mapped(error: Error) -> LdapResultCode {
    match error.status.as_u16() {
        400 => LdapResultCode::InappropriateMatching,
        401 | 403 => LdapResultCode::InsufficentAccessRights,
        404 => LdapResultCode::NoSuchObject,
        409 | 429 => LdapResultCode::Busy,
        _ => LdapResultCode::Unavailable,
    }
}
async fn release(core: &Core, auth: Option<Auth>) {
    if let Some(Auth::User(token)) = auth {
        let core = core.clone();
        let _ = tokio::task::spawn_blocking(move || core.logout(&token)).await;
    }
}
async fn connection(
    core: Core,
    stream: tokio::net::TcpStream,
    peer: IpAddr,
    config: Listener,
    tls_config: Option<Arc<rustls::ServerConfig>>,
) -> anyhow::Result<()> {
    let mut secured = config.local_unencrypted;
    let stream: Box<dyn Io> = if config.ldaps {
        secured = true;
        Box::new(
            tokio::time::timeout(
                Duration::from_secs(5),
                tokio_rustls::TlsAcceptor::from(tls_config.clone().unwrap()).accept(stream),
            )
            .await??,
        )
    } else {
        Box::new(stream)
    };
    let mut wire = Framed::new(stream, LdapCodec::new(Some(32768), Some(16)));
    let mut state = Connection::default();
    let result = async {
        for _ in 0..1000 {
            let Some(request) = tokio::time::timeout(Duration::from_secs(60), wire.next()).await?
            else {
                break;
            };
            let request = request?;
            let id = request.msgid;
            if id <= 0 {
                break;
            }
            let worker = core.clone();
            let category = format!("ldap:{}", config.client_id);
            if tokio::task::spawn_blocking(move || worker.ldap_rate_limit(peer, &category))
                .await??
            {
                break;
            }
            match request.op {
                LdapOp::UnbindRequest => break,
                LdapOp::AbandonRequest(_) => {}
                LdapOp::ExtendedRequest(ref req) if req.name == STARTTLS => {
                    release(&core, state.auth.take()).await;
                    state.page = None;
                    if secured
                        || tls_config.is_none()
                        || req.value.is_some()
                        || !request.ctrl.is_empty()
                    {
                        send(
                            &mut wire,
                            msg(
                                id,
                                LdapOp::ExtendedResponse(LdapExtendedResponse {
                                    res: result(LdapResultCode::OperationsError),
                                    name: None,
                                    value: None,
                                }),
                            ),
                        )
                        .await?;
                        continue;
                    }
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::ExtendedResponse(LdapExtendedResponse {
                                res: result(LdapResultCode::Success),
                                name: Some(STARTTLS.into()),
                                value: None,
                            }),
                        ),
                    )
                    .await?;
                    let parts = wire.into_parts();
                    if !parts.read_buf.is_empty() || !parts.write_buf.is_empty() {
                        return Err(anyhow::anyhow!("Pipelined plaintext after STARTTLS"));
                    }
                    let tls = tokio::time::timeout(
                        Duration::from_secs(5),
                        tokio_rustls::TlsAcceptor::from(tls_config.clone().unwrap())
                            .accept(parts.io),
                    )
                    .await??;
                    wire = Framed::new(
                        Box::new(tls) as Box<dyn Io>,
                        LdapCodec::new(Some(32768), Some(16)),
                    );
                    secured = true;
                }
                LdapOp::BindRequest(bind) => {
                    release(&core, state.auth.take()).await;
                    state.page = None;
                    let mut code = LdapResultCode::ConfidentialityRequired;
                    if secured {
                        code = LdapResultCode::InvalidCredentials;
                        if request.ctrl.is_empty()
                            && let LdapBindCred::Simple(password) = bind.cred
                            && !password.is_empty()
                            && password.len() <= 4096
                        {
                            let worker = core.clone();
                            let cid = config.client_id.clone();
                            let password = zeroize::Zeroizing::new(password);
                            match tokio::task::spawn_blocking(move || {
                                bind_user(&worker, &cid, &bind.dn, &password)
                            })
                            .await?
                            {
                                Ok(auth) => {
                                    state.auth = Some(auth);
                                    code = LdapResultCode::Success;
                                }
                                Err(error) => {
                                    if error.status.as_u16() >= 500 {
                                        code = LdapResultCode::Unavailable;
                                    }
                                }
                            }
                        }
                    }
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::BindResponse(LdapBindResponse {
                                res: result(code),
                                saslcreds: None,
                            }),
                        ),
                    )
                    .await?;
                }
                LdapOp::SearchRequest(search) => {
                    let mut page_control = None;
                    let mut invalid = false;
                    for control in request.ctrl {
                        match control {
                            LdapControl::SimplePagedResults { size, cookie }
                                if page_control.is_none() =>
                            {
                                page_control = Some((size, cookie))
                            }
                            LdapControl::Unknown {
                                criticality: false, ..
                            }
                            | LdapControl::ManageDsaIT { criticality: false } => {}
                            _ => invalid = true,
                        }
                    }
                    if invalid {
                        send(
                            &mut wire,
                            msg(
                                id,
                                LdapOp::SearchResultDone(result(
                                    LdapResultCode::UnavailableCriticalExtension,
                                )),
                            ),
                        )
                        .await?;
                        continue;
                    }
                    if !secured && !search.base.is_empty() {
                        send(
                            &mut wire,
                            msg(
                                id,
                                LdapOp::SearchResultDone(result(
                                    LdapResultCode::ConfidentialityRequired,
                                )),
                            ),
                        )
                        .await?;
                        continue;
                    }
                    let worker = core.clone();
                    let cid = config.client_id.clone();
                    let auth = state.auth.clone();
                    let query = search.clone();
                    let starttls = tls_config.is_some() && !config.ldaps;
                    let found = tokio::task::spawn_blocking(move || {
                        search_entries(&worker, &cid, auth.as_ref(), &query, starttls)
                    })
                    .await?;
                    let (mut entries, revision) = match found {
                        Ok(v) => v,
                        Err(error) => {
                            state.page = None;
                            send(
                                &mut wire,
                                msg(id, LdapOp::SearchResultDone(result(mapped(error)))),
                            )
                            .await?;
                            continue;
                        }
                    };
                    let limited = search.sizelimit > 0 && entries.len() > search.sizelimit as usize;
                    if limited {
                        entries.truncate(search.sizelimit as usize);
                    }
                    let total = entries.len();
                    let code = if limited {
                        LdapResultCode::SizeLimitExceeded
                    } else {
                        LdapResultCode::Success
                    };
                    let mut controls = Vec::new();
                    if let Some((size, cookie)) = page_control {
                        let fingerprint = digest(&serde_json::to_string(&search)?);
                        let previous = state.page.take();
                        let offset = if cookie.is_empty() {
                            0
                        } else {
                            match previous {
                                Some(p)
                                    if p.cookie == cookie
                                        && p.fingerprint == fingerprint
                                        && p.revision == revision
                                        && p.expires_at > now() =>
                                {
                                    p.offset
                                }
                                _ => {
                                    send(
                                        &mut wire,
                                        msg(
                                            id,
                                            LdapOp::SearchResultDone(result(
                                                LdapResultCode::UnwillingToPerform,
                                            )),
                                        ),
                                    )
                                    .await?;
                                    continue;
                                }
                            }
                        };
                        if !(0..=500).contains(&size) || offset > total {
                            send(
                                &mut wire,
                                msg(
                                    id,
                                    LdapOp::SearchResultDone(result(
                                        LdapResultCode::AdminLimitExceeded,
                                    )),
                                ),
                            )
                            .await?;
                            continue;
                        }
                        let end = (offset + size as usize).min(total);
                        entries = entries
                            .into_iter()
                            .skip(offset)
                            .take(size as usize)
                            .collect();
                        let next = if size > 0 && end < total {
                            let cookie = crypto_token();
                            state.page = Some(Page {
                                cookie: cookie.clone(),
                                fingerprint,
                                revision,
                                offset: end,
                                expires_at: now() + 300,
                            });
                            cookie
                        } else {
                            vec![]
                        };
                        controls.push(LdapControl::SimplePagedResults {
                            size: total as i64,
                            cookie: next,
                        });
                    } else {
                        state.page = None;
                    }
                    if limited {
                        state.page = None;
                        controls.clear();
                    }
                    for mut entry in entries {
                        let all = search.attrs.is_empty() || search.attrs.iter().any(|s| s == "*");
                        let operational = search.attrs.iter().any(|s| s == "+");
                        entry.attributes.retain(|a| {
                            (all && a.atype != "entryUUID")
                                || (operational && a.atype == "entryUUID")
                                || search
                                    .attrs
                                    .iter()
                                    .any(|s| s.eq_ignore_ascii_case(&a.atype))
                        });
                        if search.typesonly {
                            for attr in &mut entry.attributes {
                                attr.vals.clear();
                            }
                        }
                        send(&mut wire, msg(id, LdapOp::SearchResultEntry(entry))).await?;
                    }
                    send(
                        &mut wire,
                        LdapMsg {
                            msgid: id,
                            op: LdapOp::SearchResultDone(result(code)),
                            ctrl: controls,
                        },
                    )
                    .await?;
                }
                LdapOp::ExtendedRequest(req)
                    if req.name == WHOAMI && req.value.is_none() && request.ctrl.is_empty() =>
                {
                    let worker = core.clone();
                    let cid = config.client_id.clone();
                    let auth = state.auth.clone();
                    let who =
                        tokio::task::spawn_blocking(move || whoami(&worker, &cid, auth.as_ref()))
                            .await?;
                    let (code, value) = match who {
                        Ok(s) => (LdapResultCode::Success, Some(s.into_bytes())),
                        Err(e) => (mapped(e), None),
                    };
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::ExtendedResponse(LdapExtendedResponse {
                                res: result(code),
                                name: None,
                                value,
                            }),
                        ),
                    )
                    .await?;
                }
                LdapOp::ModifyRequest(_) => {
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::ModifyResponse(result(LdapResultCode::UnwillingToPerform)),
                        ),
                    )
                    .await?
                }
                LdapOp::AddRequest(_) => {
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::AddResponse(result(LdapResultCode::UnwillingToPerform)),
                        ),
                    )
                    .await?
                }
                LdapOp::DelRequest(_) => {
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::DelResponse(result(LdapResultCode::UnwillingToPerform)),
                        ),
                    )
                    .await?
                }
                LdapOp::ModifyDNRequest(_) => {
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::ModifyDNResponse(result(LdapResultCode::UnwillingToPerform)),
                        ),
                    )
                    .await?
                }
                LdapOp::CompareRequest(_) => {
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::CompareResult(result(LdapResultCode::UnwillingToPerform)),
                        ),
                    )
                    .await?
                }
                LdapOp::ExtendedRequest(_) => {
                    send(
                        &mut wire,
                        msg(
                            id,
                            LdapOp::ExtendedResponse(LdapExtendedResponse {
                                res: result(LdapResultCode::UnavailableCriticalExtension),
                                name: None,
                                value: None,
                            }),
                        ),
                    )
                    .await?
                }
                _ => break,
            }
        }
        anyhow::Ok(())
    }
    .await;
    release(&core, state.auth.take()).await;
    result
}
fn crypto_token() -> Vec<u8> {
    crate::crypto::random_token("").into_bytes()
}
fn bind_user(core: &Core, cid: &str, dn: &str, password: &str) -> Result<Auth> {
    let (settings, username) = core.ldap_bind_target(cid, dn, password)?;
    let Some((username, mfa)) = username else {
        return Ok(Auth::Agent(zeroize::Zeroizing::new(password.into())));
    };
    let (password, otp) = password
        .rsplit_once(';')
        .filter(|(_, otp)| {
            mfa && (otp.starts_with("ri_recovery_")
                || otp.bytes().all(|b| b.is_ascii_digit()) && matches!(otp.len(), 6 | 8))
        })
        .map(|(p, o)| (p, Some(o.to_owned())))
        .unwrap_or((password, None));
    let login = core.login(username, password.into(), otp)?;
    let token = zeroize::Zeroizing::new(
        login["session_token"]
            .as_str()
            .ok_or_else(Error::unauthorized)?
            .to_owned(),
    );
    let allowed = core.ldap_bind_authorized(cid, &token);
    if let Err(error) = allowed {
        let _ = core.logout(&token);
        return Err(error);
    }
    let _ = settings;
    Ok(Auth::User(token))
}
fn whoami(core: &Core, cid: &str, auth: Option<&Auth>) -> Result<String> {
    core.ldap_whoami(cid, auth)
}
pub(crate) fn entry(dn: String, attrs: Vec<(&str, Vec<String>)>) -> LdapSearchResultEntry {
    LdapSearchResultEntry {
        dn,
        attributes: attrs
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(a, v)| LdapPartialAttribute {
                atype: a.into(),
                vals: v.into_iter().map(String::into_bytes).collect(),
            })
            .collect(),
    }
}
fn search_entries(
    core: &Core,
    cid: &str,
    auth: Option<&Auth>,
    query: &LdapSearchRequest,
    starttls: bool,
) -> Result<(Vec<LdapSearchResultEntry>, u64)> {
    if query.sizelimit < 0
        || query.timelimit < 0
        || query.attrs.len() > 64
        || query.attrs.iter().any(|s| s.len() > 128)
        || query.aliases != LdapDerefAliases::Never
    {
        return Err(Error::bad("Unsupported search options"));
    }
    validate_filter(&query.filter, 0, &mut 0)?;
    core.ldap_search_entries(cid, auth, query, starttls)
}
fn validate_filter(filter: &LdapFilter, depth: usize, nodes: &mut usize) -> Result<()> {
    *nodes += 1;
    if depth > 12 || *nodes > 128 {
        return Err(Error::bad("LDAP filter exceeds complexity limits"));
    }
    match filter {
        LdapFilter::And(v) | LdapFilter::Or(v) => {
            if v.is_empty() {
                return Err(Error::bad("Empty LDAP boolean filter"));
            }
            for f in v {
                validate_filter(f, depth + 1, nodes)?;
            }
        }
        LdapFilter::Not(f) => validate_filter(f, depth + 1, nodes)?,
        LdapFilter::Equality(a, v) => {
            if a.len() > 128 || v.len() > 2048 {
                return Err(Error::bad("Oversized LDAP assertion"));
            }
        }
        LdapFilter::Present(a) => {
            if a.len() > 128 {
                return Err(Error::bad("Oversized LDAP attribute"));
            }
        }
        LdapFilter::Substring(a, s) => {
            if a.len() > 128
                || s.any.len() > 16
                || s.initial
                    .iter()
                    .chain(&s.any)
                    .chain(s.final_.iter())
                    .any(|v| v.len() > 2048)
            {
                return Err(Error::bad("Oversized LDAP substring filter"));
            }
        }
        _ => {
            return Err(Error::bad(
                "This LDAP profile supports equality, presence, substring and boolean filters",
            ));
        }
    }
    Ok(())
}
pub(crate) fn matches_filter(filter: &LdapFilter, row: &LdapSearchResultEntry) -> bool {
    let values = |name: &str| {
        row.attributes
            .iter()
            .filter(move |a| a.atype.eq_ignore_ascii_case(name))
            .flat_map(|a| a.vals.iter())
            .filter_map(|v| std::str::from_utf8(v).ok())
            .collect::<Vec<_>>()
            .into_iter()
    };
    match filter {
        LdapFilter::And(v) => v.iter().all(|f| matches_filter(f, row)),
        LdapFilter::Or(v) => v.iter().any(|f| matches_filter(f, row)),
        LdapFilter::Not(f) => !matches_filter(f, row),
        LdapFilter::Equality(a, expected) => {
            values(a).any(|v| v.to_lowercase() == expected.to_lowercase())
        }
        LdapFilter::Present(a) => values(a).next().is_some(),
        LdapFilter::Substring(a, s) => values(a).any(|v| {
            let v = v.to_lowercase();
            let mut rest = v.as_str();
            if let Some(first) = &s.initial {
                let first = first.to_lowercase();
                let Some(next) = rest.strip_prefix(&first) else {
                    return false;
                };
                rest = next;
            }
            for middle in &s.any {
                let middle = middle.to_lowercase();
                let Some(offset) = rest.find(&middle) else {
                    return false;
                };
                rest = &rest[offset + middle.len()..];
            }
            s.final_
                .as_ref()
                .is_none_or(|last| rest.ends_with(&last.to_lowercase()))
        }),
        _ => false,
    }
}
