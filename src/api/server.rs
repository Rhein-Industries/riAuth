use super::*;
use axum::Extension;
use axum_server::accept::Accept;
use futures_util::future::BoxFuture;
use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::server::TlsStream;
use tower::Layer;

pub async fn serve(core: Core) -> anyhow::Result<()> {
    let tls = tls_configuration(&core.config).await?;
    let listener = tokio::net::TcpListener::bind(core.config.listen).await?;
    let _ldap_servers = crate::ldap_server::start(core.clone()).await?;
    let _radius_servers = crate::radius::start(core.clone()).await?;
    let _proxy_servers = crate::proxy_server::start(core.clone()).await?;
    tracing::info!(listen = %listener.local_addr()?, issuer = %core.config.issuer, "riAuth listening");
    let maintenance_core = core.clone();
    let delivery_core = core.clone();
    let mail_core = core.clone();
    let provisioning_core = core.clone();
    let provisioning_worker = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            if crate::provisioning::deliver(provisioning_core.clone())
                .await
                .is_err()
            {
                tracing::warn!("Provisioning worker unavailable; retrying");
            }
        }
    });
    let mail_worker = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if crate::lifecycle::deliver(mail_core.clone()).await.is_err() {
                tracing::warn!("Account email delivery failed; retrying");
            }
        }
    });
    let delivery_worker = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            if let Err(error) = crate::logout::deliver(delivery_core.clone()).await {
                tracing::warn!(%error, "Logout delivery failed; retrying");
            }
            if let Err(error) = crate::ssf::deliver(delivery_core.clone()).await {
                tracing::warn!(%error, "SSF delivery failed; retrying");
            }
        }
    });
    let maintenance = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let core = maintenance_core.clone();
            match tokio::task::spawn_blocking(move || core.cleanup()).await {
                Ok(Ok(())) => {}
                other => tracing::error!(?other, "database maintenance failed"),
            }
            let core = maintenance_core.clone();
            if let Err(error) = crate::operations::dispatch_alerts(&core).await {
                tracing::warn!(%error, "alert webhook dispatch failed");
            }
        }
    });
    let tls_worker = if let Some(tls) = tls.clone() {
        let config = core.config.clone();
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
    let result = if let Some(tls) = tls {
        let handle = axum_server::Handle::new();
        let signal = handle.clone();
        let shutdown_worker = tokio::spawn(async move {
            shutdown().await;
            signal.graceful_shutdown(Some(Duration::from_secs(30)));
        });
        let result = into_rustls_server(listener.into_std()?, tls)?
            .handle(handle)
            .serve(router(core).into_make_service_with_connect_info::<SocketAddr>())
            .await;
        shutdown_worker.abort();
        result
    } else {
        axum::serve(
            listener,
            router(core).into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown())
        .await
    };
    maintenance.abort();
    delivery_worker.abort();
    mail_worker.abort();
    provisioning_worker.abort();
    if let Some(worker) = tls_worker {
        worker.abort();
    }
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
        Ok(config)
    })
    .await?
}

/// TLS service that records the peer certificate after the handshake.
/// Deployments without `client_certificates` still use `with_no_client_auth`.
#[derive(Clone)]
pub struct ClientCertAcceptor {
    inner: axum_server::tls_rustls::RustlsAcceptor,
}

impl ClientCertAcceptor {
    pub fn new(inner: axum_server::tls_rustls::RustlsAcceptor) -> Self {
        Self { inner }
    }
}

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

pub fn into_rustls_server(
    listener: std::net::TcpListener,
    tls: axum_server::tls_rustls::RustlsConfig,
) -> io::Result<axum_server::Server<std::net::SocketAddr, ClientCertAcceptor>> {
    Ok(axum_server::from_tcp_rustls(listener, tls)?.map(ClientCertAcceptor::new))
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
