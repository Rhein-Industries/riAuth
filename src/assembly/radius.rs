//! RADIUS client, identity and account reads over concrete storage.

use crate::{
    core::Core,
    crypto::now,
    error::{Error, Result},
    model::{Client, Identity, Session, User},
    radius::Settings,
    store::Tx,
};

impl Core {
    pub(crate) fn radius_client_profile(
        &self,
        tx: &Tx<'_>,
        id: &str,
    ) -> Result<(Client, Settings)> {
        let client = tx
            .get::<Client>("clients", id)?
            .filter(|c| c.enabled)
            .ok_or_else(Error::forbidden)?;
        let settings = client
            .settings
            .radius
            .clone()
            .ok_or_else(Error::forbidden)?;
        settings.validate(&client)?;
        Ok((client, settings))
    }

    pub(crate) fn radius_identity(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        identity: &Identity,
    ) -> Result<User> {
        let user = self.authorize_identity(tx, client, identity)?;
        if tx
            .get::<Session>("sessions", &identity.session_id)?
            .is_none_or(|s| s.expires_at <= now())
            || crate::assurance::needs_step_up(client, &Default::default(), identity)
        {
            return Err(Error::forbidden());
        }
        crate::claims::enforce(tx, client, &user, identity, &["radius".into()].into())?;
        Ok(user)
    }

    pub(crate) fn radius_eap_client(&self, id: &str) -> Result<Option<Client>> {
        self.store.read(|tx| {
            let (client, settings) = self.radius_client_profile(tx, id)?;
            Ok(settings.eap_tls.then_some(client))
        })
    }

    pub(crate) fn radius_user_requires_mfa(&self, username: &str) -> Result<bool> {
        self.store
            .read(|tx| match crate::core::user_by_name(tx, username) {
                Ok(user) => Ok(user.totp_secret.is_some()),
                Err(e) if e.status.as_u16() == 404 => Ok(false),
                Err(e) => Err(e),
            })
    }
}
