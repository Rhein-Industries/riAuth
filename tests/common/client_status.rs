//! Enabled-state setup for tests of other features still uses the production review
//! boundary. The client-status regression exercises refusals and exact bindings.
use riauth::{
    core::Core,
    model::{Client, ClientStatusBinding, ClientStatusInput, NewUser},
};

pub fn set(core: &Core, author: &str, client_id: &str, enabled: bool) {
    let client: Client = core.store.get("clients", client_id).unwrap().unwrap();
    if client.enabled == enabled {
        return;
    }
    let after = ClientStatusInput { enabled };
    let administrator = |name: &str| {
        const PASSWORD: &str = "fixture-client-status-review-only";
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
    let reviewer = administrator("client-status-reviewer");
    let executor = administrator("client-status-executor");
    let staged = core.stage_client_status(author, client_id, after).unwrap();
    let id = staged["proposal"]["id"].as_str().unwrap();
    let binding = ClientStatusBinding {
        digest: staged["digest"].as_str().unwrap().into(),
    };
    core.approve_client_status_change(&reviewer, id, binding.clone())
        .unwrap();
    core.execute_client_status_change(&executor, id, binding)
        .unwrap();
}
