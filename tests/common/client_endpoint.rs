//! Endpoint setup for tests of other features still uses the production review
//! boundary. The client-endpoint regression exercises refusals and exact bindings.
use riauth::{
    core::Core,
    model::{Client, ClientEndpointBinding, ClientEndpointInput, NewUser},
};

pub fn set(
    core: &Core,
    author: &str,
    client_id: &str,
    redirects: Option<Vec<String>>,
    origins: Option<std::collections::BTreeSet<String>>,
) {
    let client: Client = core.store.get("clients", client_id).unwrap().unwrap();
    let before = ClientEndpointInput {
        redirect_uris: client.redirect_uris.clone(),
        origins: client.settings.origins.clone(),
        post_logout_redirect_uris: client.settings.post_logout_redirect_uris.clone(),
        frontchannel_logout_uri: client.settings.frontchannel_logout_uri.clone(),
        backchannel_logout_uri: client.settings.backchannel_logout_uri.clone(),
    };
    let after = ClientEndpointInput {
        redirect_uris: redirects.unwrap_or(client.redirect_uris),
        origins: origins.unwrap_or(client.settings.origins),
        post_logout_redirect_uris: client.settings.post_logout_redirect_uris,
        frontchannel_logout_uri: client.settings.frontchannel_logout_uri,
        backchannel_logout_uri: client.settings.backchannel_logout_uri,
    };
    if before == after {
        return;
    }
    let administrator = |name: &str| {
        const PASSWORD: &str = "fixture-client-endpoint-review-only";
        if core
            .store
            .get::<String>("usernames", name)
            .unwrap()
            .is_none()
        {
            core.create_user(
                author,
                NewUser {
                    username: name.into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: name.into(),
                    admin: true,
                },
            )
            .unwrap();
        }
        core.login(name.into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let reviewer = administrator("client-endpoint-reviewer");
    let executor = administrator("client-endpoint-executor");
    let staged = core
        .stage_client_endpoint(author, client_id, after)
        .unwrap();
    let id = staged["proposal"]["id"].as_str().unwrap();
    let binding = ClientEndpointBinding {
        digest: staged["digest"].as_str().unwrap().into(),
    };
    core.approve_client_endpoint_change(&reviewer, id, binding.clone())
        .unwrap();
    core.execute_client_endpoint_change(&executor, id, binding)
        .unwrap();
}
