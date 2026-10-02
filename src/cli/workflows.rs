//! Remote envelope for exact-content workflow approval. The server checks that
//! author, reviewer, and executor are three distinct administrators, that the
//! stored plan is unchanged. First activation checks `--if-revision`; retries
//! revalidate the current selection without answering from a stored receipt.
use super::Remote;
use anyhow::{Result, bail};
use clap::{Subcommand, ValueEnum};
use reqwest::Method;
use serde_json::{Value, json};

#[derive(Clone, Copy, ValueEnum)]
pub enum Decision {
    Approve,
    Refuse,
}

#[derive(Subcommand)]
pub enum WorkflowCommand {
    /// Record your approval or refusal of another administrator's stored workflow plan
    Review {
        #[arg(allow_hyphen_values = true)]
        plan_id: String,
        #[arg(long, value_enum)]
        decision: Decision,
    },
    /// Commit a reviewed workflow plan as the selected definition; a third administrator runs this
    Activate {
        #[arg(allow_hyphen_values = true)]
        plan_id: String,
    },
    /// Retire exactly the named immutable approval of a workflow
    Revoke {
        #[arg(allow_hyphen_values = true)]
        workflow_id: String,
        #[arg(long, allow_hyphen_values = true)]
        approval_id: String,
    },
}

pub(super) async fn run(remote: &Remote, command: WorkflowCommand) -> Result<Value> {
    if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
        bail!(
            "Workflow approval requires --idempotency-key and --if-revision (from `riauth revision`)"
        );
    }
    let (path, body) = match command {
        WorkflowCommand::Review { plan_id, decision } => (
            "/api/workflow-approvals/review",
            json!({
                "plan_id": plan_id,
                "decision": match decision {
                    Decision::Approve => "approve",
                    Decision::Refuse => "refuse",
                },
            }),
        ),
        WorkflowCommand::Activate { plan_id } => (
            "/api/workflow-approvals/activate",
            json!({"plan_id": plan_id}),
        ),
        WorkflowCommand::Revoke {
            workflow_id,
            approval_id,
        } => (
            "/api/workflow-approvals/revoke",
            json!({"workflow_id": workflow_id, "approval_id": approval_id}),
        ),
    };
    let response = remote
        .call(Method::POST, path, Some(body.clone()), true)
        .await?;
    for field in ["plan_id", "decision", "workflow_id", "approval_id"] {
        if let Some(expected) = body.get(field)
            && response.get(field) != Some(expected)
        {
            bail!("Workflow response does not match the requested {field}");
        }
    }
    Ok(response)
}
