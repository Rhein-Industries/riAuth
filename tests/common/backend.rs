//! Factories for the same contracts over real storage backends.
//! PostgreSQL is opt-in and each fixture owns a new database in a verified
//! disposable cluster; no schema in an existing application database is reused.
use super::{Fixture, PASSWORD, text};
use postgres::{Client, NoTls, config::Host};
use riauth::{config::Config, core::Core, crypto, model::NewUser, postgres_store::PostgresConfig};
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    Redb,
    EncryptedRedb,
    Postgres,
    EncryptedPostgres,
}

pub struct BackendFixture {
    // Fields drop in declaration order: close Core/pool before dropping its DB.
    fixture: Fixture,
    _database: Option<DisposableDatabase>,
}

impl Deref for BackendFixture {
    type Target = Fixture;
    fn deref(&self) -> &Fixture {
        &self.fixture
    }
}

impl DerefMut for BackendFixture {
    fn deref_mut(&mut self) -> &mut Fixture {
        &mut self.fixture
    }
}

impl BackendFixture {
    pub fn reopen_with(self, check: impl FnOnce(&Config)) -> Self {
        Self {
            fixture: self.fixture.reopen_with(check),
            _database: self._database,
        }
    }

    /// Drop the open store and open the same files again after `edit`.
    /// The disposable database, when there is one, stays mounted.
    pub fn reopen_edited(self, edit: impl FnOnce(&mut Config)) -> Self {
        let Self {
            fixture: Fixture { _dir, core, admin },
            _database,
        } = self;
        let mut config = core.config.clone();
        drop(core);
        edit(&mut config);
        Self {
            fixture: Fixture {
                _dir,
                core: Core::open(config).unwrap(),
                admin,
            },
            _database,
        }
    }

    /// A database-native restore of this instance's current state: PostgreSQL
    /// clones the database (a new database OID; `TEMPLATE` keeps relation OIDs);
    /// redb copies the closed file. The copy is opened; the original is discarded.
    pub fn restored_copy(self) -> Self {
        let Self {
            fixture: Fixture { _dir, core, admin },
            _database,
        } = self;
        let mut config = core.config.clone();
        drop(core);
        let database = match (&mut config.postgres, _database) {
            (Some(postgres), Some(mut original)) => {
                let copy = original.clone_database();
                let file = _dir.path().join("restored-connection");
                riauth::config::write_private(&file, copy.connection.as_bytes(), false).unwrap();
                postgres.connection_file = file;
                drop(original);
                Some(copy)
            }
            (None, None) => {
                let data = _dir.path().join("restored-data");
                riauth::config::private_dir(&data).unwrap();
                std::fs::copy(
                    config.data_dir.join("riauth.redb"),
                    data.join("riauth.redb"),
                )
                .unwrap();
                config.data_dir = data;
                None
            }
            _ => unreachable!("fixture backend and database disagree"),
        };
        Self {
            fixture: Fixture {
                _dir,
                core: Core::open(config).unwrap(),
                admin,
            },
            _database: database,
        }
    }
}

/// Configuration for a store that was never initialized: no redb file, or an
/// empty disposable PostgreSQL database.
pub struct EmptyStore {
    _dir: tempfile::TempDir,
    pub config: Config,
    database: Option<DisposableDatabase>,
}

impl EmptyStore {
    /// Nothing exists yet at the configured location: no data directory, or no
    /// riAuth schema in the database.
    pub fn untouched(&self) -> bool {
        match &self.database {
            None => !self.config.data_dir.exists(),
            Some(database) => database
                .connection
                .parse::<postgres::Config>()
                .unwrap()
                .connect(NoTls)
                .unwrap()
                .query_one(
                    "SELECT NOT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'riauth_store')",
                    &[],
                )
                .unwrap()
                .get(0),
        }
    }
}

impl Backend {
    pub fn uninitialized(self) -> EmptyStore {
        let (_dir, config, database) = self.empty();
        EmptyStore {
            _dir,
            config,
            database,
        }
    }

    fn empty(self) -> (tempfile::TempDir, Config, Option<DisposableDatabase>) {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config {
            data_dir: dir.path().join("data"),
            ..Default::default()
        };
        if matches!(self, Self::EncryptedRedb | Self::EncryptedPostgres) {
            let key = dir.path().join("storage.key");
            riauth::config::write_private(&key, crypto::random_token("").as_bytes(), false)
                .unwrap();
            config.database_key_file = Some(key);
        }
        let database = if matches!(self, Self::Postgres | Self::EncryptedPostgres) {
            let database = DisposableDatabase::new();
            let connection_file = dir.path().join("connection");
            riauth::config::write_private(&connection_file, database.connection.as_bytes(), false)
                .unwrap();
            config.postgres = Some(PostgresConfig {
                connection_file,
                ca_file: None,
                local_unencrypted: true,
                pool_size: 2,
            });
            Some(database)
        } else {
            None
        };
        (dir, config, database)
    }

    pub fn fixture(self) -> BackendFixture {
        if matches!(self, Self::Redb) {
            return BackendFixture {
                fixture: Fixture::new(),
                _database: None,
            };
        }
        let (dir, config, database) = self.empty();
        let core = Core::initialize(
            config,
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        assert_eq!(
            core.store.backend(),
            if database.is_some() {
                "postgresql"
            } else {
                "redb"
            }
        );
        let admin = text(
            &core.login("admin".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        BackendFixture {
            fixture: Fixture {
                _dir: dir,
                core,
                admin,
            },
            _database: database,
        }
    }
}

struct DisposableDatabase {
    control: Client,
    name: String,
    connection: String,
}

impl DisposableDatabase {
    fn new() -> Self {
        let root = PathBuf::from(
            std::env::var_os("RIAUTH_TEST_CONTRACT_PG_ROOT")
                .expect("PostgreSQL contracts require scripts/test-contracts-postgres.sh"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("marker")).unwrap(),
            "riauth disposable contract cluster\n"
        );
        let connection =
            riauth::config::read_private_secret(&root.join("connection"), 16384).unwrap();
        let config: postgres::Config = connection.trim().parse().unwrap();
        assert_eq!(config.get_dbname(), Some("postgres"));
        assert_eq!(config.get_user(), Some("riauth_test"));
        assert!(matches!(config.get_hosts(), [Host::Tcp(host)] if host == "127.0.0.1"));
        assert!(config.get_hostaddrs().is_empty());
        assert!(matches!(config.get_ports(), [port] if *port != 0));
        let mut control = config.connect(NoTls).unwrap();
        // A marker beside a connection file alone is insufficient: verify the
        // actual server data directory before creating or dropping any database.
        let actual: String = control
            .query_one("SHOW data_directory", &[])
            .unwrap()
            .get(0);
        assert_eq!(
            PathBuf::from(actual).canonicalize().unwrap(),
            root.join("primary").canonicalize().unwrap(),
            "Refusing a PostgreSQL server outside the disposable cluster"
        );
        let name = format!("riauth_contract_{}", uuid::Uuid::new_v4().simple());
        control
            .batch_execute(&format!("CREATE DATABASE {name} TEMPLATE template0"))
            .unwrap();
        Self {
            connection: format!(
                "host=127.0.0.1 port={} dbname={name} user=riauth_test sslmode=disable",
                config.get_ports()[0]
            ),
            control,
            name,
        }
    }
}

impl DisposableDatabase {
    /// Requires that no connection to this database remains open.
    fn clone_database(&mut self) -> Self {
        let name = format!("riauth_contract_{}", uuid::Uuid::new_v4().simple());
        self.control
            .batch_execute(&format!("CREATE DATABASE {name} TEMPLATE {}", self.name))
            .unwrap();
        let config: postgres::Config = self.connection.parse().unwrap();
        let connection = format!(
            "host=127.0.0.1 port={} dbname={name} user=riauth_test sslmode=disable",
            config.get_ports()[0]
        );
        let root = PathBuf::from(std::env::var_os("RIAUTH_TEST_CONTRACT_PG_ROOT").unwrap());
        let control: postgres::Config = riauth::config::read_private_secret(
            &root.canonicalize().unwrap().join("connection"),
            16384,
        )
        .unwrap()
        .trim()
        .parse()
        .unwrap();
        Self {
            control: control.connect(NoTls).unwrap(),
            name,
            connection,
        }
    }
}

impl Drop for DisposableDatabase {
    fn drop(&mut self) {
        if let Err(error) = self
            .control
            .batch_execute(&format!("DROP DATABASE {}", self.name))
        {
            if std::thread::panicking() {
                eprintln!("Disposable contract database cleanup failed: {error}");
            } else {
                panic!("Disposable contract database cleanup failed: {error}");
            }
        }
    }
}
