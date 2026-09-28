//! Local password policy persistence over the caller's transaction.

use crate::{
    core::audit,
    error::Result,
    model::{Attempts, User},
    password::PasswordTx,
    store::Tx,
};

impl PasswordTx for Tx<'_> {
    fn directory_manages(&self, user_id: &str) -> Result<bool> {
        crate::directory::manages(self, user_id)
    }

    fn attempts(&self, username: &str) -> Result<Option<Attempts>> {
        self.get("attempts", username)
    }

    fn put_attempts(&self, username: &str, attempts: &Attempts) -> Result<()> {
        self.put("attempts", username, attempts)
    }

    fn accept_history(
        &self,
        limit: u32,
        user_id: &str,
        current_hash: &str,
        plaintext: &str,
        new_hash: &str,
    ) -> Result<()> {
        crate::identity::password_history::accept(
            self,
            limit,
            user_id,
            current_hash,
            plaintext,
            new_hash,
        )
    }

    fn put_user(&self, id: &str, user: &User) -> Result<()> {
        self.put("users", id, user)
    }

    fn clear_attempts(&self, username: &str) -> Result<()> {
        self.delete("attempts", username)
    }

    fn queue_user_revocation(&self, user_id: &str) -> Result<()> {
        crate::logout::queue_user(self, user_id)
    }

    fn audit_password(&self, actor: &str, action: &str, target: &str) -> Result<()> {
        audit(self, actor, action, target)
    }
}
