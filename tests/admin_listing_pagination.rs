mod common;

use common::{Fixture, text};
use riauth::{
    agent::{NewAgent, Permission},
    model::User,
    telemetry::ReadContext,
};

fn reader(fixture: &Fixture, id: &str, resources: &[&str]) -> String {
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: id.into(),
                ttl: 600,
                parent: None,
                permissions: resources
                    .iter()
                    .map(|resource| Permission {
                        action: "user.read".into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

#[test]
fn user_listing_pages_bucket_and_keeps_order_and_per_row_permissions() {
    let fixture = Fixture::new();
    let template = fixture
        .core
        .store
        .read(|tx| Ok(tx.list::<User>("users")?.pop().unwrap().1))
        .unwrap();
    let mut expected = vec![(template.id.clone(), template.username.clone())];
    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..130 {
                let mut user = template.clone();
                user.id = format!("u{index:03}");
                user.username = format!("reader-{index:03}");
                user.display_name = user.username.clone();
                user.admin = false;
                tx.put("users", &user.id, &user)?;
                tx.put("usernames", &user.username, &user.id)?;
                expected.push((user.id, user.username));
            }
            Ok(())
        })
        .unwrap();
    expected.sort_by(|left, right| left.0.cmp(&right.0));
    let all_reader = reader(&fixture, "all-users-reader", &["*"]);
    let scoped_reader = reader(
        &fixture,
        "two-users-reader",
        &["user/reader-000", "user/reader-129"],
    );
    let scans = &fixture.core.store.telemetry().reads;
    let counts = || {
        (
            scans.scans(ReadContext::Read, false).count(),
            scans.scans(ReadContext::Read, true).count(),
            scans.scans(ReadContext::Read, true).sum(),
        )
    };

    let before = counts();
    let all = fixture.core.list_users(&all_reader).unwrap();
    let after = counts();
    assert_eq!(after.0, before.0, "listing must not load the full bucket");
    assert_eq!(after.1 - before.1, 2);
    assert_eq!(after.2 - before.2, 131);
    let usernames: Vec<_> = all
        .as_array()
        .unwrap()
        .iter()
        .map(|user| text(user, "username"))
        .collect();
    assert_eq!(
        usernames,
        expected
            .into_iter()
            .map(|(_, username)| username)
            .collect::<Vec<_>>()
    );
    assert!(
        all.as_array()
            .unwrap()
            .iter()
            .all(|user| user.get("password_hash").is_none())
    );

    let before = counts();
    let scoped = fixture.core.list_users(&scoped_reader).unwrap();
    let after = counts();
    assert_eq!(after.0, before.0);
    assert_eq!(after.1 - before.1, 2);
    assert_eq!(after.2 - before.2, 131);
    assert_eq!(
        scoped
            .as_array()
            .unwrap()
            .iter()
            .map(|user| text(user, "username"))
            .collect::<Vec<_>>(),
        ["reader-000", "reader-129"]
    );
    assert_eq!(fixture.core.list_users(&fixture.admin).unwrap(), all);
    fixture
        .core
        .revoke_agent(&fixture.admin, "two-users-reader")
        .unwrap();
    assert!(fixture.core.list_users(&scoped_reader).is_err());
}
