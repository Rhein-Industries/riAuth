//! TOTP enrollment and recovery code transaction port over concrete storage.

use crate::{
    authenticator::{self, AuthenticatorMaintenance, AuthenticatorTx, Enrollment},
    core::{Core, audit},
    error::Result,
    model::{Session, User},
    store::Tx,
};
use serde_json::Value;

/// Pending-enrollment bindings are keyed by user id.
const ENROLLMENTS: &str = "totp_enrollments";

impl AuthenticatorMaintenance for Tx<'_> {
    fn enrollment_page(&self) -> Result<Vec<(String, Value)>> {
        self.maintenance_page(ENROLLMENTS)
    }

    fn delete_enrollment(&self, user_id: &str) -> Result<()> {
        self.delete(ENROLLMENTS, user_id)
    }

    fn user_record(&self, user_id: &str) -> Result<Option<User>> {
        self.get("users", user_id)
    }

    fn put_user(&self, id: &str, user: &User) -> Result<()> {
        self.put("users", id, user)
    }
}

impl AuthenticatorTx for Tx<'_> {
    fn enrollment_record(&self, user_id: &str) -> Result<Option<Enrollment>> {
        self.get(ENROLLMENTS, user_id)
    }

    fn put_enrollment(&self, user_id: &str, enrollment: &Enrollment) -> Result<()> {
        self.put(ENROLLMENTS, user_id, enrollment)
    }

    fn queue_user_revocation(&self, user_id: &str) -> Result<()> {
        crate::logout::queue_user(self, user_id)
    }

    fn audit_factor(&self, actor: &str, action: &str, target: &str) -> Result<()> {
        audit(self, actor, action, target)
    }
}

impl Core {
    pub(crate) fn totp_start_in(
        &self,
        tx: &Tx<'_>,
        user: User,
        session: &Session,
        replace: bool,
    ) -> Result<Value> {
        authenticator::totp_start_in(tx, user, session, replace)
    }

    pub(crate) fn totp_confirm_in(
        &self,
        tx: &Tx<'_>,
        user: User,
        session: &Session,
        code: &str,
        recovery: bool,
    ) -> Result<Value> {
        authenticator::totp_confirm_in(tx, user, session, code, recovery)
    }

    pub(crate) fn totp_cancel_in(
        &self,
        tx: &Tx<'_>,
        user: User,
        session: &Session,
    ) -> Result<Value> {
        authenticator::totp_cancel_in(tx, user, session)
    }

    pub(crate) fn totp_remove_in(
        &self,
        tx: &Tx<'_>,
        user: User,
        session: &Session,
    ) -> Result<Value> {
        authenticator::totp_remove_in(tx, user, session)
    }

    pub(crate) fn recovery_codes_in(
        &self,
        tx: &Tx<'_>,
        user: User,
        session: &Session,
    ) -> Result<Value> {
        authenticator::recovery_codes_in(tx, user, session)
    }

    pub(crate) fn totp_status_in(
        &self,
        tx: &Tx<'_>,
        user: &User,
        session: &Session,
    ) -> Result<Value> {
        authenticator::totp_status_in(tx, user, session)
    }
}
