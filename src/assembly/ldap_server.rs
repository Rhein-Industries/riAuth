//! LDAP bind profile and authorization reads over concrete storage.

use crate::{
    core::Core,
    error::{Error, Result},
    ldap_server::Settings,
    model::{Client, Group, User},
    store::Tx,
};

impl Core {
    pub(crate) fn ldap_profile(&self, tx: &Tx<'_>, id: &str) -> Result<(Client, Settings)> {
        let client = tx
            .get::<Client>("clients", id)?
            .filter(|c| c.enabled)
            .ok_or_else(Error::forbidden)?;
        let settings = client.settings.ldap.clone().ok_or_else(Error::forbidden)?;
        settings.validate(&client)?;
        for group in &settings.search_groups {
            if tx.get::<Group>("groups", group)?.is_none() {
                return Err(Error::bad("LDAP search group does not exist"));
            }
        }
        Ok((client, settings))
    }

    pub(crate) fn ldap_bind_target(
        &self,
        cid: &str,
        dn: &str,
        password: &str,
    ) -> Result<(Settings, Option<(String, bool)>)> {
        self.store.read(|tx| {
            let (_, settings) = self.ldap_profile(tx, cid)?;
            if dn.eq_ignore_ascii_case(&settings.agent_dn()) {
                self.management(tx, password, "ldap.search", &format!("client/{cid}"))?;
                if !password.starts_with("ri_agent_") {
                    return Err(Error::forbidden());
                }
                return Ok((settings, None));
            }
            let matches: Vec<_> = tx
                .list::<User>("users")?
                .into_iter()
                .map(|(_, u)| u)
                .filter(|u| settings.user_dn(&u.username).eq_ignore_ascii_case(dn))
                .collect();
            if matches.len() != 1 {
                return Err(Error::unauthorized());
            }
            Ok((
                settings,
                Some((
                    matches[0].username.clone(),
                    matches[0].totp_secret.is_some(),
                )),
            ))
        })
    }

    pub(crate) fn ldap_bind_authorized(&self, cid: &str, token: &str) -> Result<()> {
        self.store.read(|tx| {
            let (client, _) = self.ldap_profile(tx, cid)?;
            let (_, session) = self.session(tx, token)?;
            self.authorize_identity(tx, &client, &session.identity)?;
            if crate::assurance::needs_step_up(&client, &Default::default(), &session.identity) {
                return Err(Error::forbidden());
            }
            Ok(())
        })
    }
}
