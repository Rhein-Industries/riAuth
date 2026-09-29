//! Policy setup for tests of other features still uses the production review
//! boundary. The client-policy regression exercises refusals and exact bindings.
use riauth::{
    core::Core,
    model::{Client, ClientPolicyBinding, ClientPolicyInput, NewUser},
};
use std::collections::BTreeSet;

pub fn replace(
    core: &Core,
    author: &str,
    client_id: &str,
    allowed_groups: Option<BTreeSet<String>>,
    require_mfa: Option<bool>,
) {
    let client: Client = core.store.get("clients", client_id).unwrap().unwrap();
    let after = ClientPolicyInput {
        allowed_groups: allowed_groups.unwrap_or_else(|| client.allowed_groups.clone()),
        require_mfa: require_mfa.unwrap_or(client.require_mfa),
    };
    if after.allowed_groups == client.allowed_groups && after.require_mfa == client.require_mfa {
        return;
    }
    let administrator = |name: &str| {
        const PASSWORD: &str = "fixture-client-policy-review-only";
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
    let reviewer = administrator("client-policy-reviewer");
    let executor = administrator("client-policy-executor");
    let staged = core.stage_client_policy(author, client_id, after).unwrap();
    let id = staged["proposal"]["id"].as_str().unwrap();
    let binding = ClientPolicyBinding {
        digest: staged["digest"].as_str().unwrap().into(),
    };
    core.approve_client_policy_change(&reviewer, id, binding.clone())
        .unwrap();
    core.execute_client_policy_change(&executor, id, binding)
        .unwrap();
}
