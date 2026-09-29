//! Shared contract bodies over real redb and PostgreSQL backend factories.
#[path = "contracts/backend_parity.rs"]
mod backend_parity;
mod common;
#[path = "contracts/shared.rs"]
mod shared;

macro_rules! backend_contract {
    ($name:ident) => {
        mod $name {
            use super::common::backend::Backend;
            #[test]
            fn redb() {
                super::shared::$name(Backend::Redb);
            }
            #[test]
            fn redb_encrypted() {
                super::shared::$name(Backend::EncryptedRedb);
            }
            #[test]
            #[ignore = "requires an isolated cluster; use scripts/test-contracts-postgres.sh"]
            fn postgres() {
                super::shared::$name(Backend::Postgres);
            }
            #[test]
            #[ignore = "requires an isolated cluster; use scripts/test-contracts-postgres.sh"]
            fn postgres_encrypted() {
                super::shared::$name(Backend::EncryptedPostgres);
            }
        }
    };
}

backend_contract!(identity_and_issuer_continuity);
backend_contract!(disable_reenable_revokes_dependents);
backend_contract!(proof_account_session_request_binding);
backend_contract!(code_binding_and_verified_replay);
backend_contract!(refresh_rotation_and_verified_replay);
backend_contract!(live_group_policy_revalidation);
backend_contract!(indexed_user_group_membership);
backend_contract!(group_binding_metadata);
backend_contract!(prepared_authority_revalidation);
#[cfg(feature = "test-support")]
backend_contract!(prepared_deadline_revalidation);
backend_contract!(last_admin_failure_is_atomic);
backend_contract!(http_mutation_receipts_and_audit);
backend_contract!(plan_binding_atomicity_and_retry);
backend_contract!(application_writers_share_management_seam);
backend_contract!(password_attempts_and_change);
backend_contract!(totp_and_recovery_code_binding);
backend_contract!(passkey_ceremony_binding_and_replay);
backend_contract!(account_reset_binding_and_atomicity);
#[cfg(feature = "test-support")]
backend_contract!(account_proof_supersession_and_expiry);
#[cfg(feature = "test-support")]
backend_contract!(verification_proof_binding_and_replay);
backend_contract!(invitation_acceptance_revalidates_creator);
#[cfg(feature = "platform")]
backend_contract!(invitation_passkey_bound_competing_completion);
backend_contract!(user_writers_share_management_seam);
backend_contract!(desired_state_user_writes_share_management_seam);
backend_contract!(passkey_registration_requires_user_verification);
#[cfg(feature = "test-support")]
backend_contract!(passkey_management_requires_fresh_mfa);
backend_contract!(offboard_intent_durable_cancel);
#[cfg(feature = "test-support")]
backend_contract!(offboard_retry_rechecks_authority);
backend_contract!(cloud_snapshot_apply_atomic_retry);
backend_contract!(database_native_restore_policy);
backend_contract!(direct_restore_selected_backend);
backend_contract!(direct_restore_failure_preserves_original);
#[test]
#[ignore = "requires an isolated cluster; use scripts/test-contracts-postgres.sh"]
fn direct_restore_postgres_archive_into_redb() {
    shared::direct_restore_postgres_archive_into_redb();
}
backend_contract!(recovery_status_never_creates_or_writes_a_store);
