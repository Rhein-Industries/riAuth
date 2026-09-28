#[path = "common/mod.rs"]
mod common;

#[cfg(feature = "platform")]
#[test]
fn configured_workflow_roundtrips_through_manifest_with_scoped_authority() {
    use common::Fixture;
    use riauth::{
        agent::{NewAgent, Permission},
        state::{ApplyRequest, Manifest},
        workflow::Definition,
    };
    use serde_json::json;

    let fixture = Fixture::new();
    let definition: Definition = serde_json::from_value(json!({
        "format":"riauth.workflow/v1", "id":"platform-password", "revision":1,
        "category":"authentication", "origin":"configured", "entry":"password",
        "limits":{"max_duration_seconds":600,"max_executions":3},
        "steps":[{"id":"password","action":{"type":"verify_password"},
            "max_attempts":3,"timeout_seconds":300,"cancellable":true,
            "transitions":[{"on":"verified","to":"success"},{"on":"failed","to":"denied"}]}],
        "terminals":[{"id":"success","outcome":"authenticated","requires":[]},
            {"id":"denied","outcome":"denied","requires":[]}]
    }))
    .unwrap();
    let manifest = Manifest {
        api_version: "riauth/v1".into(),
        workflows: vec![definition.clone()],
        ..Default::default()
    };
    let reader = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "workflow-reader".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "workflow.read".into(),
                    resource: "workflow/platform-password".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        fixture
            .core
            .plan_state(&reader, manifest.clone())
            .err()
            .unwrap()
            .code,
        "access_denied"
    );

    let plan = fixture.core.plan_state(&fixture.admin, manifest).unwrap();
    assert_eq!(plan.changes.len(), 1);
    fixture
        .core
        .apply_state(
            &fixture.admin,
            ApplyRequest {
                plan,
                secrets: Default::default(),
                run_id: None,
            },
        )
        .unwrap();
    let exported: Manifest = serde_json::from_value(
        fixture.core.export_state(&fixture.admin).unwrap()["manifest"].clone(),
    )
    .unwrap();
    assert_eq!(exported.workflows, vec![definition.clone()]);
    let listed: Vec<Definition> =
        serde_json::from_value(fixture.core.list_workflow_definitions(&reader).unwrap()).unwrap();
    assert_eq!(listed, vec![definition.clone()]);

    let mut changed = definition.clone();
    changed.revision = 2;
    changed.steps[0].timeout_seconds = 240;
    let writer = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "workflow-writer".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "workflow.write".into(),
                    resource: "workflow/platform-password".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        fixture.core.list_workflow_definitions(&writer).unwrap(),
        json!([])
    );
    let plan = fixture
        .core
        .plan_state(
            &writer,
            Manifest {
                api_version: "riauth/v1".into(),
                workflows: vec![changed.clone()],
                ..Default::default()
            },
        )
        .unwrap();
    fixture
        .core
        .apply_state(
            &writer,
            ApplyRequest {
                plan,
                secrets: Default::default(),
                run_id: None,
            },
        )
        .unwrap();
    assert_eq!(
        fixture.core.list_workflow_definitions(&reader).unwrap(),
        json!([changed])
    );
}
