//! Duties of the one process that is starting.
//!
//! The omitted role is `integrated`: authentication routes, configured protocol
//! listeners, and background loops. `gateway` and `worker` run a fixed subset
//! and require `accept_partial_duties`, because this process does not look for
//! a peer that covers the rest. The embedded store still has one owner.

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessRole {
    #[default]
    Integrated,
    Gateway,
    Worker,
}

impl ProcessRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Integrated => "integrated",
            Self::Gateway => "gateway",
            Self::Worker => "worker",
        }
    }

    pub fn duties(self) -> Duties {
        let authentication = matches!(self, Self::Integrated | Self::Gateway);
        Duties {
            authentication,
            // Essentials has no LDAP, RADIUS, or proxy listener implementation.
            protocol_listeners: authentication && cfg!(feature = "platform"),
            background_jobs: matches!(self, Self::Integrated | Self::Worker),
        }
    }
}

/// What this process starts. Flags describe startup ownership, not peer health.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Duties {
    pub authentication: bool,
    pub protocol_listeners: bool,
    pub background_jobs: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessSelection {
    #[serde(default)]
    pub role: ProcessRole,
    /// Required for `gateway` and `worker`. Confirms the omitted duties are
    /// intentional and that no peer was verified.
    #[serde(default, skip_serializing_if = "is_false")]
    pub accept_partial_duties: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl ProcessSelection {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Copy)]
pub struct ListenerNames<'a> {
    pub ldap: &'a [&'a str],
    pub radius: &'a [&'a str],
    pub proxy: &'a [&'a str],
}

pub fn validate(
    selection: &ProcessSelection,
    browser_ui: bool,
    listeners: ListenerNames<'_>,
) -> Result<()> {
    match selection.role {
        ProcessRole::Integrated => {
            if selection.accept_partial_duties {
                bail!(
                    "process.accept_partial_duties applies to the gateway and worker roles. The integrated role runs authentication, configured protocol listeners, and background jobs in this process"
                );
            }
        }
        ProcessRole::Gateway => {
            if !selection.accept_partial_duties {
                bail!(
                    "Process role gateway does not run background jobs. Set process.accept_partial_duties = true to confirm that omission. riAuth does not check that any other process runs them"
                );
            }
        }
        ProcessRole::Worker => {
            if !selection.accept_partial_duties {
                bail!(
                    "Process role worker does not serve authentication routes or bind protocol listeners. Set process.accept_partial_duties = true to confirm that omission. riAuth does not check that any other process serves them"
                );
            }
            if browser_ui {
                bail!(
                    "Process role worker does not serve embedded browser pages. Set browser_ui = false on this process, or select the integrated or gateway role"
                );
            }
            refuse_listeners("LDAP", listeners.ldap)?;
            refuse_listeners("RADIUS", listeners.radius)?;
            refuse_listeners("proxy", listeners.proxy)?;
        }
    }
    Ok(())
}

fn refuse_listeners(kind: &str, ids: &[&str]) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let shown = ids.iter().take(8).copied().collect::<Vec<_>>().join(", ");
    bail!(
        "Process role worker does not bind {kind} listeners ({shown}). Remove them from this process, or select the integrated or gateway role"
    );
}

/// A worker has no setup or product routes, so it cannot create the store.
pub fn reject_uninitialized_worker(role: ProcessRole) -> Result<()> {
    if role == ProcessRole::Worker {
        bail!(
            "Process role worker cannot initialize a store or serve setup. Use the integrated or gateway role until the store is initialized"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omitted_role_is_integrated_and_owns_this_builds_duties() {
        let selection = ProcessSelection::default();
        assert_eq!(selection.role, ProcessRole::Integrated);
        assert!(!selection.accept_partial_duties);
        validate(
            &selection,
            true,
            ListenerNames {
                ldap: &[],
                radius: &[],
                proxy: &[],
            },
        )
        .unwrap();
        let duties = selection.role.duties();
        assert!(duties.authentication);
        assert!(duties.background_jobs);
        assert_eq!(duties.protocol_listeners, cfg!(feature = "platform"));
        assert_eq!(ProcessRole::Gateway.duties().background_jobs, false);
        assert_eq!(ProcessRole::Worker.duties().authentication, false);
        assert_eq!(ProcessRole::Worker.duties().protocol_listeners, false);
        assert!(ProcessRole::Worker.duties().background_jobs);
    }

    #[test]
    fn partial_roles_fail_closed_without_acknowledgement_or_with_worker_listeners() {
        let listeners = ListenerNames {
            ldap: &[],
            radius: &[],
            proxy: &[],
        };
        let gateway = ProcessSelection {
            role: ProcessRole::Gateway,
            accept_partial_duties: false,
        };
        let error = validate(&gateway, true, listeners).unwrap_err().to_string();
        assert!(error.contains("accept_partial_duties"), "{error}");
        let worker = ProcessSelection {
            role: ProcessRole::Worker,
            accept_partial_duties: false,
        };
        let error = validate(&worker, false, listeners).unwrap_err().to_string();
        assert!(error.contains("accept_partial_duties"), "{error}");
        let acknowledged = ProcessSelection {
            role: ProcessRole::Worker,
            accept_partial_duties: true,
        };
        let error = validate(&acknowledged, true, listeners)
            .unwrap_err()
            .to_string();
        assert!(error.contains("browser_ui"), "{error}");
        let error = validate(
            &acknowledged,
            false,
            ListenerNames {
                ldap: &["local"],
                radius: &["wifi"],
                proxy: &[],
            },
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("LDAP"), "{error}");
        assert!(error.contains("local"), "{error}");
        validate(&acknowledged, false, listeners).unwrap();
        let integrated = ProcessSelection {
            role: ProcessRole::Integrated,
            accept_partial_duties: true,
        };
        let error = validate(&integrated, true, listeners)
            .unwrap_err()
            .to_string();
        assert!(error.contains("integrated role"), "{error}");
        reject_uninitialized_worker(ProcessRole::Integrated).unwrap();
        reject_uninitialized_worker(ProcessRole::Gateway).unwrap();
        let error = reject_uninitialized_worker(ProcessRole::Worker)
            .unwrap_err()
            .to_string();
        assert!(error.contains("cannot initialize"), "{error}");
    }
}
