//! Workflow approval through the server's one management service.
//!
//! The same routes as `riauth workflows`: `POST /api/workflow-approvals/review`,
//! `/activate` and `/revoke`. The server owns the rule that author, reviewer
//! and executor are three distinct administrators, the exact-content check on
//! the stored plan, the revision precondition, the receipt and the audit
//! record. Every write sends `If-Match` and an `Idempotency-Key`. The ids are
//! sent in the JSON body, never in a path, so only the server's own bounds
//! (1-128 bytes, no control characters) apply locally.

use crate::{
    admin::{MutationOptions, mutate},
    transport::Remote,
};
use anyhow::{Result, bail};
use clap::{Subcommand, ValueEnum};
use reqwest::Method;
use serde_json::{Value, json};

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Decision {
    Approve,
    Refuse,
}

impl Decision {
    fn word(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Refuse => "refuse",
        }
    }
}

#[derive(Subcommand)]
pub(crate) enum WorkflowCommand {
    /// Record your approval or refusal of another administrator's stored workflow plan.
    Review {
        #[arg(allow_hyphen_values = true)]
        plan_id: String,
        #[arg(long, value_enum)]
        decision: Decision,
    },
    /// Commit a reviewed workflow plan as the selected definition; a third administrator runs this.
    Activate {
        #[arg(allow_hyphen_values = true)]
        plan_id: String,
    },
    /// Retire the current approval of a workflow.
    Revoke {
        #[arg(allow_hyphen_values = true)]
        workflow_id: String,
    },
}

/// The server's bounds on an identifier in a workflow approval body.
fn bounded(value: &str, name: &str) -> Result<()> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        bail!("Invalid {name}: 1-128 characters without control characters");
    }
    Ok(())
}

fn named(response: &Value, field: &str, expected: &str, what: &str) -> Result<()> {
    if response.get(field).and_then(Value::as_str) != Some(expected) {
        bail!("{what} response does not match the requested {field}");
    }
    Ok(())
}

pub(crate) async fn run(
    remote: &Remote,
    command: WorkflowCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        WorkflowCommand::Review { plan_id, decision } => {
            bounded(&plan_id, "plan_id")?;
            let body = json!({"plan_id": plan_id, "decision": decision.word()});
            let reviewed = mutate(
                remote,
                Method::POST,
                "/api/workflow-approvals/review",
                Some(&body),
                options,
            )
            .await?;
            named(&reviewed, "plan_id", &plan_id, "Review")?;
            named(&reviewed, "decision", decision.word(), "Review")?;
            Ok(reviewed)
        }
        WorkflowCommand::Activate { plan_id } => {
            bounded(&plan_id, "plan_id")?;
            let body = json!({"plan_id": plan_id});
            let activated = mutate(
                remote,
                Method::POST,
                "/api/workflow-approvals/activate",
                Some(&body),
                options,
            )
            .await?;
            named(&activated, "plan_id", &plan_id, "Activation")?;
            Ok(activated)
        }
        WorkflowCommand::Revoke { workflow_id } => {
            bounded(&workflow_id, "workflow_id")?;
            let body = json!({"workflow_id": workflow_id});
            let revoked = mutate(
                remote,
                Method::POST,
                "/api/workflow-approvals/revoke",
                Some(&body),
                options,
            )
            .await?;
            named(&revoked, "workflow_id", &workflow_id, "Revocation")?;
            Ok(revoked)
        }
    }
}
