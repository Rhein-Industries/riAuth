//! A bounded, read-only LDAPv3 provider with TLS and scoped service credentials.
use crate::{
    core::{Core, durable_groups_for, validate_name},
    crypto::{digest, now},
    error::{Error, Result},
    model::{Client, Group, User},
    store::Tx,
};
use futures_util::{SinkExt, StreamExt};
use ldap3_proto::{LdapCodec, control::LdapControl, proto::*};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
    path::PathBuf,
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
const STARTTLS: &str = "1.3.6.1.4.1.1466.20037";
const WHOAMI: &str = "1.3.6.1.4.1.4203.1.11.3";
const PAGED: &str = "1.2.840.113556.1.4.319";
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub base_dn: String,
    pub search_groups: BTreeSet<String>,
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
    fn user_dn(&self, name: &str) -> String {
        format!("uid={name},ou=users,{}", self.base_dn)
    }
    fn group_dn(&self, name: &str) -> String {
        format!("cn={name},ou=groups,{}", self.base_dn)
    }
    fn agent_dn(&self) -> String {
        format!("cn=riauth-agent,{}", self.base_dn)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    pub listen: SocketAddr,
    pub client_id: String,
    pub allowed_peers: BTreeSet<IpAddr>,
    pub tls_cert_file: Option<PathBuf>,
    pub tls_key_file: Option<PathBuf>,
    #[serde(default)]
    pub ldaps: bool,
    #[serde(default)]
    pub local_unencrypted: bool,
}
impl Listener {
    pub fn validate(&self) -> Result<()> {
        validate_name(&self.client_id)?;
        if self.allowed_peers.is_empty()
            || self.allowed_peers.len() > 128
            || self
                .allowed_peers
                .iter()
                .any(|ip| ip.is_unspecified() || ip.is_multicast())
        {
            return Err(Error::bad(
                "LDAP listeners require one to 128 explicit peer IPs",
            ));
        }
        if self.local_unencrypted {
            if !self.listen.ip().is_loopback()
                || self.ldaps
                || self.tls_cert_file.is_some()
                || self.tls_key_file.is_some()
            {
                return Err(Error::bad(
                    "Unencrypted LDAP is restricted to an explicit loopback listener",
                ));
            }
        } else if self.tls_cert_file.is_none() || self.tls_key_file.is_none() {
            return Err(Error::bad(
                "LDAP requires certificate and key files for LDAPS or mandatory STARTTLS",
            ));
        }
        Ok(())
    }
}
pub struct Servers {
    pub addresses: Vec<SocketAddr>,
    tasks: Vec<JoinHandle<()>>,
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
    for listener in core.config.ldap_listeners.values() {
        listener.validate()?;
        let tls = tls(&core, listener).await?;
        let socket = TcpListener::bind(listener.listen).await?;
        ready.push((socket, listener.clone(), tls));
    }
    let mut servers = Servers {
        addresses: Vec::new(),
        tasks: Vec::new(),
    };
    for (socket, config, tls) in ready {
        servers.addresses.push(socket.local_addr()?);
        let core = core.clone();
        servers
            .tasks
            .push(tokio::spawn(listen(core, socket, config, tls)));
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
enum Auth {
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
            if tokio::task::spawn_blocking(move || {
                worker.store.shared_rate_limit(peer, &category, 600)
            })
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
fn profile(tx: &Tx<'_>, id: &str) -> Result<(Client, Settings)> {
    let client = tx
        .get::<Client>("clients", id)?
        .filter(|c| c.enabled)
        .ok_or_else(Error::forbidden)?;
    let settings = client.settings.ldap.clone().ok_or_else(Error::forbidden)?;
    settings.validate(&client)?;
    for group in &settings.search_groups {
        if tx.get::<Group>("groups", group)?.is_none() {
            return Err(Error::bad("LDAP search group does not exist"));
        }
    }
    Ok((client, settings))
}
fn bind_user(core: &Core, cid: &str, dn: &str, password: &str) -> Result<Auth> {
    let (settings, username) = core.store.read(|tx| {
        let (_, settings) = profile(tx, cid)?;
        if dn.eq_ignore_ascii_case(&settings.agent_dn()) {
            core.management(tx, password, "ldap.search", &format!("client/{cid}"))?;
            if !password.starts_with("ri_agent_") {
                return Err(Error::forbidden());
            }
            return Ok((settings, None));
        }
        let matches: Vec<_> = tx
            .list::<User>("users")?
            .into_iter()
            .map(|(_, u)| u)
            .filter(|u| settings.user_dn(&u.username).eq_ignore_ascii_case(dn))
            .collect();
        if matches.len() != 1 {
            return Err(Error::unauthorized());
        }
        Ok((
            settings,
            Some((
                matches[0].username.clone(),
                matches[0].totp_secret.is_some(),
            )),
        ))
    })?;
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
    let allowed = core.store.read(|tx| {
        let (client, _) = profile(tx, cid)?;
        let (_, session) = core.session(tx, &token)?;
        core.authorize_identity(tx, &client, &session.identity)?;
        if crate::assurance::needs_step_up(&client, &Default::default(), &session.identity) {
            return Err(Error::forbidden());
        }
        Ok(())
    });
    if let Err(error) = allowed {
        let _ = core.logout(&token);
        return Err(error);
    }
    let _ = settings;
    Ok(Auth::User(token))
}
fn authorize(core: &Core, tx: &Tx<'_>, cid: &str, auth: Option<&Auth>) -> Result<Option<User>> {
    let (client, _) = profile(tx, cid)?;
    match auth {
        Some(Auth::Agent(token)) => {
            core.management(tx, token, "ldap.search", &format!("client/{cid}"))?;
            Ok(None)
        }
        Some(Auth::User(token)) => {
            let (user, session) = core.session(tx, token)?;
            core.authorize_identity(tx, &client, &session.identity)?;
            if crate::assurance::needs_step_up(&client, &Default::default(), &session.identity) {
                return Err(Error::forbidden());
            }
            Ok(Some(user))
        }
        None => Err(Error::unauthorized()),
    }
}
fn whoami(core: &Core, cid: &str, auth: Option<&Auth>) -> Result<String> {
    core.store.read(|tx| {
        let (_, settings) = profile(tx, cid)?;
        if auth.is_none() {
            return Ok(String::new());
        }
        let user = authorize(core, tx, cid, auth)?;
        Ok(format!(
            "dn:{}",
            user.as_ref()
                .map(|u| settings.user_dn(&u.username))
                .unwrap_or_else(|| settings.agent_dn())
        ))
    })
}
fn entry(dn: String, attrs: Vec<(&str, Vec<String>)>) -> LdapSearchResultEntry {
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
    core.store.read(|tx| {
        let (client, settings) = profile(tx, cid)?;
        let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
        if query.base.is_empty() && query.scope == LdapSearchScope::Base {
            let row = entry(
                String::new(),
                vec![
                    ("objectClass", vec!["top".into()]),
                    ("namingContexts", vec![settings.base_dn]),
                    ("supportedLDAPVersion", vec!["3".into()]),
                    (
                        "supportedExtension",
                        if starttls {
                            vec![STARTTLS.into(), WHOAMI.into()]
                        } else {
                            vec![WHOAMI.into()]
                        },
                    ),
                    ("supportedControl", vec![PAGED.into()]),
                ],
            );
            let rows = if matches_filter(&query.filter, &row) {
                vec![row]
            } else {
                Vec::new()
            };
            return Ok((rows, revision));
        }
        let self_user = authorize(core, tx, cid, auth)?;
        let all_groups = tx.list::<Group>("groups")?;
        let selected: BTreeSet<String> = all_groups
            .iter()
            .filter(|(name, _)| settings.search_groups.contains(name))
            .flat_map(|(_, g)| g.members.iter().cloned())
            .collect();
        let users: Vec<_> = tx
            .list::<User>("users")?
            .into_iter()
            .map(|(_, u)| u)
            .filter(|u| {
                u.enabled
                    && selected.contains(&u.id)
                    && self_user.as_ref().is_none_or(|me| me.id == u.id)
            })
            .collect();
        if users.len() > 2000 {
            return Err(Error::bad(
                "LDAP profile supports at most 2000 selected users",
            ));
        }
        let visible: BTreeMap<_, _> = users
            .iter()
            .map(|u| (u.id.clone(), u.username.clone()))
            .collect();
        let mut rows = Vec::new();
        let mut unique = BTreeSet::new();
        rows.push(entry(
            settings.base_dn.clone(),
            vec![
                ("objectClass", vec!["top".into(), "domain".into()]),
                (
                    "dc",
                    vec![
                        settings
                            .base_dn
                            .split(',')
                            .next()
                            .unwrap()
                            .trim_start_matches("dc=")
                            .into(),
                    ],
                ),
            ],
        ));
        for ou in ["users", "groups"] {
            rows.push(entry(
                format!("ou={ou},{}", settings.base_dn),
                vec![
                    (
                        "objectClass",
                        vec!["top".into(), "organizationalUnit".into()],
                    ),
                    ("ou", vec![ou.into()]),
                ],
            ));
        }
        for user in users {
            let dn = settings.user_dn(&user.username);
            if !unique.insert(dn.to_ascii_lowercase()) {
                return Err(Error::conflict(
                    "LDAP DNs collide under case-insensitive matching",
                ));
            }
            let memberships = durable_groups_for(tx, &user.id)?;
            let mut attrs = vec![
                (
                    "objectClass",
                    vec![
                        "top".into(),
                        "person".into(),
                        "organizationalPerson".into(),
                        "inetOrgPerson".into(),
                    ],
                ),
                ("uid", vec![user.username.clone()]),
                ("cn", vec![user.display_name.clone()]),
                ("sn", vec![user.display_name.clone()]),
                ("displayName", vec![user.display_name.clone()]),
                ("entryUUID", vec![user.id.clone()]),
            ];
            if client.scopes.contains("email") {
                attrs.push(("mail", user.email.into_iter().collect()));
            }
            if client.scopes.contains("groups") {
                attrs.push((
                    "memberOf",
                    memberships
                        .into_iter()
                        .map(|g| settings.group_dn(&g))
                        .collect(),
                ));
            }
            rows.push(entry(dn, attrs));
        }
        if client.scopes.contains("groups") {
            for (name, group) in all_groups {
                let members: Vec<_> = group
                    .members
                    .iter()
                    .filter_map(|id| visible.get(id))
                    .map(|name| settings.user_dn(name))
                    .collect();
                if members.is_empty() {
                    continue;
                }
                let dn = settings.group_dn(&name);
                if !unique.insert(dn.to_ascii_lowercase()) {
                    return Err(Error::conflict("LDAP group DNs collide"));
                }
                rows.push(entry(
                    dn,
                    vec![
                        ("objectClass", vec!["top".into(), "groupOfNames".into()]),
                        ("cn", vec![name]),
                        ("member", members),
                    ],
                ));
            }
        }
        let base = query.base.to_ascii_lowercase();
        if !rows.iter().any(|r| r.dn.eq_ignore_ascii_case(&query.base)) {
            return Err(Error::missing("LDAP base not found"));
        }
        rows.retain(|row| {
            let dn = row.dn.to_ascii_lowercase();
            let suffix = format!(",{base}");
            let child = dn.strip_suffix(&suffix);
            let in_scope = match query.scope {
                LdapSearchScope::Base => dn == base,
                LdapSearchScope::OneLevel => child.is_some_and(|s| !s.contains(',')),
                LdapSearchScope::Subtree => dn == base || child.is_some(),
                LdapSearchScope::Children => child.is_some(),
            };
            in_scope && matches_filter(&query.filter, row)
        });
        rows.sort_by(|a, b| a.dn.cmp(&b.dn));
        if rows.iter().map(LdapSearchResultEntry::size).sum::<usize>() > 4 * 1024 * 1024 {
            return Err(Error::bad("LDAP result exceeds provider limit"));
        }
        Ok((rows, revision))
    })
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
fn matches_filter(filter: &LdapFilter, row: &LdapSearchResultEntry) -> bool {
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
