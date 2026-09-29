use super::*;
use axum::Extension;
#[cfg(feature = "platform")]
use axum_server::accept::Accept;
#[cfg(feature = "platform")]
use futures_util::future::BoxFuture;
use std::io;
#[cfg(feature = "platform")]
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "platform")]
use tokio_rustls::server::TlsStream;
#[cfg(feature = "platform")]
use tower::Layer;

struct AbortTasks(Vec<tokio::task::JoinHandle<()>>);
impl Drop for AbortTasks {
    fn drop(&mut self) {
        for task in &self.0 {
            task.abort();
        }
    }
}
#[cfg(feature = "platform")]
struct Listeners {
    _ldap: crate::ldap_server::Servers,
    _radius: crate::radius::Servers,
    _proxy: crate::proxy_server::Servers,
}
struct RoleRuntime {
    #[cfg(feature = "platform")]
    _listeners: Option<Listeners>,
    _tasks: AbortTasks,
}
#[cfg(feature = "platform")]
async fn start_listeners(core: Core) -> anyhow::Result<Listeners> {
    Ok(Listeners {
        _ldap: crate::ldap_server::start(core.clone()).await?,
        _radius: crate::radius::start(core.clone()).await?,
        _proxy: crate::proxy_server::start(core).await?,
    })
}
async fn start_background(core: Core) -> anyhow::Result<AbortTasks> {
    use crate::background::{Background, Job};
    let background = Background::shared(&core.store);
    background.initialize()?;
    let reconciliation_core = core.clone();
    let reconciliation_worker = background.spawn(Job::Reconciliation, move || {
        let core = reconciliation_core.clone();
        async move {
            tokio::task::spawn_blocking(move || core.reconciliation_process().map(drop))
                .await
                .map_err(Error::internal)?
        }
    });
    let [provisioning_worker, deactivation_worker] = background.spawn_provisioning(core.clone());
    let mail_core = core.clone();
    let mail_worker = background.spawn(Job::Mail, move || {
        crate::lifecycle::deliver(mail_core.clone())
    });
    let delivery_core = core.clone();
    let delivery_worker = background.spawn(Job::Delivery, move || {
        let core = delivery_core.clone();
        async move {
            let logout = crate::logout::deliver(core.clone()).await;
            #[cfg(feature = "platform")]
            let ssf = crate::ssf::deliver(core).await;
            logout?;
            #[cfg(feature = "platform")]
            ssf?;
            Ok(())
        }
    });
    let maintenance_core = core.clone();
    let maintenance = background.spawn(Job::Maintenance, move || {
        let core = maintenance_core.clone();
        async move {
            tokio::task::spawn_blocking(move || core.cleanup())
                .await
                .map_err(Error::internal)?
        }
    });
    // Slow alert receivers cannot hold up local scheduled revocation/cleanup.
    let alerts = background.spawn(Job::Alerts, move || {
        let core = core.clone();
        async move { crate::operations::dispatch_alerts(&core).await.map(drop) }
    });
    Ok(AbortTasks(vec![
        maintenance,
        alerts,
        delivery_worker,
        mail_worker,
        provisioning_worker,
        deactivation_worker,
        reconciliation_worker,
    ]))
}
async fn start_role(core: Core) -> anyhow::Result<RoleRuntime> {
    let duties = core.config.process.role.duties();
    #[cfg(feature = "platform")]
    let listeners = if duties.protocol_listeners {
        Some(start_listeners(core.clone()).await?)
    } else {
        None
    };
    let tasks = if duties.background_jobs {
        start_background(core).await?
    } else {
        drop(core);
        AbortTasks(Vec::new())
    };
    Ok(RoleRuntime {
        #[cfg(feature = "platform")]
        _listeners: listeners,
        _tasks: tasks,
    })
}

async fn serving_preflight(core: &Core) -> anyhow::Result<()> {
    // No protocol adapter or background worker starts on unreconciled restored state.
    let store = core.store.clone();
    let config = core.config.clone();
    tokio::task::spawn_blocking(move || {
        if let Some(mail) = &config.mail {
            mail.require_local_material().map_err(|error| {
                Error::bad(format!("SMTP configuration unusable: {}", error.message))
            })?;
        }
        crate::edition::validate_store(&store)?;
        crate::capability::validate_store(&config, &store)?;
        store.ready()
    })
    .await??;
    Ok(())
}

pub async fn serve(core: Core) -> anyhow::Result<()> {
    serving_preflight(&core).await?;
    let config = core.config.clone();
    let authentication = config.process.role.duties().authentication;
    let _runtime = start_role(core.clone()).await?;
    let routes = if authentication {
        router(core)
    } else {
        super::worker_router(core)
    };
    serve_http(config, routes).await
}

pub(crate) async fn serve_bootstrap(setup: crate::bootstrap::Bootstrap) -> anyhow::Result<()> {
    let config = setup.config.clone();
    let (signal, ready) = tokio::sync::oneshot::channel();
    let routes = crate::bootstrap::router_with_signal(setup, Some(signal));
    let serving = serve_http(config, routes);
    tokio::pin!(serving);
    tokio::select! {
        result = &mut serving => result,
        core = ready => {
            let core = core.map_err(|_| anyhow::anyhow!("Setup runtime closed"))?;
            serving_preflight(&core).await?;
            let _runtime = start_role(core).await?;
            serving.await
        }
    }
}

/// Graceful shutdown of one HTTP server, handed to its requests as an
/// extension. Work that watches it, such as a streamed backup export, stops
/// instead of holding the shutdown open. Each server has its own, so stopping
/// one never reaches another server in the same process.
#[derive(Clone)]
pub struct Shutdown(Arc<tokio::sync::watch::Sender<bool>>);

impl Default for Shutdown {
    fn default() -> Self {
        Self(Arc::new(tokio::sync::watch::Sender::new(false)))
    }
}

impl Shutdown {
    pub fn begin(&self) {
        self.0.send_replace(true);
    }
    pub fn started(&self) -> bool {
        *self.0.borrow()
    }
    /// Resolves once `begin` was called, at once if it already was.
    pub async fn wait(&self) {
        // `self` holds the sender, so the channel cannot close first.
        let _ = self.0.subscribe().wait_for(|started| *started).await;
    }
}

async fn serve_http(config: crate::config::Config, routes: axum::Router) -> anyhow::Result<()> {
    let stopping = Shutdown::default();
    // Applied last, so every route, fallback and the bootstrap hand-off sees it.
    let routes = routes.layer(Extension(stopping.clone()));
    let tls = tls_configuration(&config).await?;
    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    tracing::info!(
        listen = %listener.local_addr()?,
        issuer = %config.issuer,
        role = config.process.role.as_str(),
        "riAuth listening"
    );
    let tls_worker = if let Some(tls) = tls.clone() {
        let config = config.clone();
        Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            interval.tick().await;
            loop {
                interval.tick().await;
                match tls_configuration(&config).await {
                    Ok(Some(replacement)) => tls.reload_from_config(replacement.get_inner()),
                    _ => tracing::warn!(
                        "TLS certificate reload failed; retaining the active configuration"
                    ),
                }
            }
        }))
    } else {
        None
    };
    let _tls_worker = AbortTasks(tls_worker.into_iter().collect());
    let result = if let Some(tls) = tls {
        let handle = axum_server::Handle::new();
        let signal = handle.clone();
        let shutdown_worker = tokio::spawn(async move {
            shutdown().await;
            stopping.begin();
            signal.graceful_shutdown(Some(Duration::from_secs(30)));
        });
        let _shutdown_worker = AbortTasks(vec![shutdown_worker]);
        into_rustls_server(listener.into_std()?, tls)?
            .handle(handle)
            .serve(routes.into_make_service_with_connect_info::<SocketAddr>())
            .await
    } else {
        axum::serve(
            listener,
            routes.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            shutdown().await;
            stopping.begin();
        })
        .await
    };
    result?;
    Ok(())
}
pub async fn tls_configuration(
    config: &crate::config::Config,
) -> anyhow::Result<Option<axum_server::tls_rustls::RustlsConfig>> {
    config.validate()?;
    match (&config.tls_cert_file, &config.tls_key_file) {
        (Some(cert), Some(key)) => {
            let mut server = tls_server(
                cert.clone(),
                key.clone(),
                config.client_certificates.clone(),
            )
            .await?;
            server.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            Ok(Some(axum_server::tls_rustls::RustlsConfig::from_config(
                std::sync::Arc::new(server),
            )))
        }
        _ => {
            #[cfg(feature = "platform")]
            if let Some(profile) = &config.client_certificates {
                let profile = profile.clone();
                tokio::task::spawn_blocking(move || profile.material().map(|_| ()))
                    .await?
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
            }
            Ok(None)
        }
    }
}
#[cfg(feature = "platform")]
pub(crate) async fn tls_files(
    cert: std::path::PathBuf,
    key: std::path::PathBuf,
) -> anyhow::Result<rustls::ServerConfig> {
    tls_server(cert, key, None).await
}
async fn tls_server(
    cert: std::path::PathBuf,
    key: std::path::PathBuf,
    client: Option<crate::mtls::ClientCertAuth>,
) -> anyhow::Result<rustls::ServerConfig> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
        let certificates =
            CertificateDer::pem_file_iter(cert)?.collect::<std::result::Result<Vec<_>, _>>()?;
        let private_key = PrivateKeyDer::from_pem_file(key)?;
        let builder = rustls::ServerConfig::builder_with_provider(std::sync::Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_safe_default_protocol_versions()?;
        #[cfg(feature = "platform")]
        let config = if let Some(profile) = client {
            let material = profile
                .material()
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let verifier = profile
                .verifier(&material)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let mut config = builder
                .with_client_cert_verifier(verifier)
                .with_single_cert(certificates, private_key)?;
            // Resumption can omit the certificate the login handler needs.
            config.send_tls13_tickets = 0;
            config.session_storage = std::sync::Arc::new(rustls::server::NoServerSessionStorage {});
            config
        } else {
            builder
                .with_no_client_auth()
                .with_single_cert(certificates, private_key)?
        };
        #[cfg(not(feature = "platform"))]
        let config = {
            if client.is_some() {
                anyhow::bail!("Client-certificate login requires the Platform build");
            }
            builder
                .with_no_client_auth()
                .with_single_cert(certificates, private_key)?
        };
        Ok(config)
    })
    .await?
}

/// TLS service that records the peer certificate after the handshake.
/// Deployments without `client_certificates` still use `with_no_client_auth`.
#[cfg(feature = "platform")]
#[derive(Clone)]
pub struct ClientCertAcceptor {
    inner: axum_server::tls_rustls::RustlsAcceptor,
}

#[cfg(feature = "platform")]
impl ClientCertAcceptor {
    pub fn new(inner: axum_server::tls_rustls::RustlsAcceptor) -> Self {
        Self { inner }
    }
}

#[cfg(feature = "platform")]
impl<I, S> Accept<I, S> for ClientCertAcceptor
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    S: Send + 'static,
{
    type Stream = TlsStream<I>;
    type Service = axum::middleware::AddExtension<S, crate::mtls::TlsClientCerts>;
    type Future = BoxFuture<'static, io::Result<(Self::Stream, Self::Service)>>;

    fn accept(&self, stream: I, service: S) -> Self::Future {
        let acceptor = self.inner.clone();
        Box::pin(async move {
            let (stream, service) = acceptor.accept(stream, service).await?;
            let ders = stream
                .get_ref()
                .1
                .peer_certificates()
                .map(|chain| chain.iter().map(|cert| cert.as_ref().to_vec()).collect())
                .unwrap_or_default();
            let service = Extension(crate::mtls::TlsClientCerts { ders }).layer(service);
            Ok((stream, service))
        })
    }
}

#[cfg(feature = "platform")]
pub fn into_rustls_server(
    listener: std::net::TcpListener,
    tls: axum_server::tls_rustls::RustlsConfig,
) -> io::Result<axum_server::Server<std::net::SocketAddr, ClientCertAcceptor>> {
    Ok(axum_server::from_tcp_rustls(listener, tls)?.map(ClientCertAcceptor::new))
}

/// Essentials uses axum-server's ordinary TLS acceptor; no certificate capture
/// or client-certificate verifier is linked into this server assembly.
#[cfg(not(feature = "platform"))]
pub fn into_rustls_server(
    listener: std::net::TcpListener,
    tls: axum_server::tls_rustls::RustlsConfig,
) -> io::Result<axum_server::Server<std::net::SocketAddr, axum_server::tls_rustls::RustlsAcceptor>>
{
    axum_server::from_tcp_rustls(listener, tls)
}

async fn shutdown() {
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler");
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = term.recv() => {} }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
