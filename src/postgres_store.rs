//! PostgreSQL connection policy and bounded synchronous worker pool.
use crate::error::{Error, Result};
use postgres::{
    Client,
    config::{Host, SslMode, TargetSessionAttrs},
};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};

pub const WRITE_LOCK: i64 = 0x524941555448;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    /// libpq connection string in a private file; reread when reconnecting.
    pub connection_file: PathBuf,
    pub ca_file: Option<PathBuf>,
    #[serde(default)]
    pub local_unencrypted: bool,
    #[serde(default = "pool_size")]
    pub pool_size: usize,
}
fn pool_size() -> usize {
    8
}
impl PostgresConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let mut config: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        let base = path.parent().unwrap_or(std::path::Path::new("."));
        for file in std::iter::once(&mut config.connection_file).chain(config.ca_file.iter_mut()) {
            if file.is_relative() {
                *file = base.join(&*file);
            }
            *file = file.canonicalize()?;
        }
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            (1..=64).contains(&self.pool_size),
            "PostgreSQL pool_size must be 1..64"
        );
        Ok(())
    }
    fn connect(&self) -> Result<Client> {
        self.validate().map_err(Error::internal)?;
        let secret =
            crate::config::read_private_secret(&self.connection_file, 16384).map_err(|_| {
                Error::bad("PostgreSQL connection must be a private file of at most 16384 bytes")
            })?;
        let mut config: postgres::Config = secret
            .trim()
            .parse()
            .map_err(|_| Error::bad("Invalid PostgreSQL connection file"))?;
        if config.get_hosts().is_empty() || config.get_hosts().len() > 8 {
            return Err(Error::bad(
                "Configure one to eight explicit PostgreSQL hosts",
            ));
        }
        config
            .application_name("riauth")
            .target_session_attrs(TargetSessionAttrs::ReadWrite)
            .connect_timeout(Duration::from_secs(5))
            .keepalives_idle(Duration::from_secs(15))
            .keepalives_interval(Duration::from_secs(5))
            .keepalives_retries(3)
            .tcp_user_timeout(Duration::from_secs(15));
        let mut client = if self.local_unencrypted {
            let local = config.get_hosts().iter().all(|host| match host {
                Host::Tcp(host) => host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback()),
                #[cfg(unix)]
                Host::Unix(_) => true,
            });
            if !local || config.get_hostaddrs().iter().any(|ip| !ip.is_loopback()) {
                return Err(Error::bad(
                    "Unencrypted PostgreSQL requires literal loopback hosts or Unix sockets",
                ));
            }
            config.ssl_mode(SslMode::Disable);
            config.connect(postgres::NoTls).map_err(unavailable)?
        } else {
            if config
                .get_hosts()
                .iter()
                .any(|host| !matches!(host, Host::Tcp(_)))
            {
                return Err(Error::bad("TLS PostgreSQL requires a certificate hostname"));
            }
            let mut roots = rustls::RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            if let Some(path) = &self.ca_file {
                use rustls::pki_types::pem::PemObject;
                for cert in rustls::pki_types::CertificateDer::pem_file_iter(path)
                    .map_err(Error::internal)?
                {
                    roots
                        .add(cert.map_err(Error::internal)?)
                        .map_err(Error::internal)?;
                }
            }
            let mut tls = rustls::ClientConfig::builder_with_provider(Arc::new(
                rustls::crypto::aws_lc_rs::default_provider(),
            ))
            .with_safe_default_protocol_versions()
            .map_err(Error::internal)?
            .with_root_certificates(roots)
            .with_no_client_auth();
            tls.alpn_protocols = vec![b"postgresql".to_vec()];
            config.ssl_mode(SslMode::Require);
            config
                .connect(tokio_postgres_rustls::MakeRustlsConnect::new(tls))
                .map_err(unavailable)?
        };
        client.batch_execute("SET statement_timeout = '15s'; SET lock_timeout = '5s'; SET idle_in_transaction_session_timeout = '30s'; SET search_path = pg_catalog; SET synchronous_commit = on;").map_err(unavailable)?;
        Ok(client)
    }
}
pub fn unavailable(error: postgres::Error) -> Error {
    tracing::warn!(
        code = error.code().map(|c| c.code()),
        "PostgreSQL operation failed"
    );
    Error::new(
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "storage_unavailable",
        "Storage unavailable; inspect state or retry with the same idempotency key",
    )
}
#[derive(Default)]
struct Connections {
    idle: Vec<Client>,
    total: usize,
}
pub struct Pool {
    config: PostgresConfig,
    connections: Mutex<Connections>,
    available: Condvar,
    telemetry: Arc<crate::telemetry::Telemetry>,
}
impl Pool {
    pub fn new(config: PostgresConfig) -> Arc<Self> {
        Self::with_telemetry(config, Arc::default())
    }
    pub(crate) fn with_telemetry(
        config: PostgresConfig,
        telemetry: Arc<crate::telemetry::Telemetry>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            connections: Mutex::default(),
            available: Condvar::new(),
            telemetry,
        })
    }
    pub fn get(self: &Arc<Self>) -> Result<Pooled> {
        let _timer = self.telemetry.pool_wait.timer();
        let started = Instant::now();
        loop {
            let mut connections = self
                .connections
                .lock()
                .map_err(|_| Error::internal("PostgreSQL pool lock poisoned"))?;
            while let Some(client) = connections.idle.pop() {
                if !client.is_closed() {
                    return Ok(Pooled {
                        client: Some(client),
                        pool: self.clone(),
                    });
                }
                connections.total -= 1;
            }
            if connections.total < self.config.pool_size {
                connections.total += 1;
                drop(connections);
                match self.config.connect() {
                    Ok(client) => {
                        return Ok(Pooled {
                            client: Some(client),
                            pool: self.clone(),
                        });
                    }
                    Err(error) => {
                        let mut connections =
                            self.connections.lock().unwrap_or_else(|e| e.into_inner());
                        connections.total -= 1;
                        self.available.notify_one();
                        return Err(error);
                    }
                }
            }
            let remaining = Duration::from_secs(5).saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(Error::new(
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    "storage_busy",
                    "All storage connections are busy; retry shortly",
                ));
            }
            drop(
                self.available
                    .wait_timeout(connections, remaining)
                    .map_err(|_| Error::internal("PostgreSQL pool wait failed"))?,
            );
        }
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        let idle = std::mem::take(
            &mut self
                .connections
                .get_mut()
                .unwrap_or_else(|e| e.into_inner())
                .idle,
        );
        // postgres owns synchronous Tokio runtimes; dispose outside an async request runtime.
        if tokio::runtime::Handle::try_current().is_ok() {
            let _ = std::thread::spawn(move || drop(idle)).join();
        } else {
            drop(idle);
        }
    }
}
pub struct Pooled {
    client: Option<Client>,
    pool: Arc<Pool>,
}
impl Deref for Pooled {
    type Target = Client;
    fn deref(&self) -> &Client {
        self.client.as_ref().unwrap()
    }
}
impl DerefMut for Pooled {
    fn deref_mut(&mut self) -> &mut Client {
        self.client.as_mut().unwrap()
    }
}
impl Drop for Pooled {
    fn drop(&mut self) {
        let mut connections = self
            .pool
            .connections
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let client = self.client.take().unwrap();
        if client.is_closed() {
            connections.total -= 1;
        } else {
            connections.idle.push(client);
        }
        self.pool.available.notify_one();
    }
}
