//! Private fixture helper for the disposable native PostgreSQL recovery drill.
//! Never print tokens, passwords, proofs, or stored key material to stdout.
use anyhow::{Context, Result, ensure};
use riauth::{
    config::{Config, write_private},
    core::Core,
    crypto::{self, digest},
    lifecycle::{Invitation, MailConfig, MailSecurity, Purpose},
    model::{NewClient, NewUser, User, UserPatch},
    oidc::{Authorization, TokenRequest},
    recovery::STRIDE,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, env, fs, io, path::Path};

const CLIENT: &str = "native-drill-client";
const REDIRECT: &str = "http://127.0.0.1:17861/callback";

fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn save(path: &Path, value: &Value) -> Result<()> {
    write_private(path, &serde_json::to_vec_pretty(value)?, false)?;
    Ok(())
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .with_context(|| format!("Missing fixture field {key}"))
}

fn user(core: &Core, name: &str) -> Result<User> {
    let id: String = core
        .store
        .get("usernames", name)?
        .with_context(|| format!("Missing fixture user {name}"))?;
    core.store
        .get("users", &id)?
        .with_context(|| format!("Missing fixture record {name}"))
}

fn active_kid(core: &Core) -> Result<String> {
    Ok(core.jwks()?["keys"][0]["kid"]
        .as_str()
        .context("Missing active signing kid")?
        .to_owned())
}

fn authorization(core: &Core, session: &str) -> Result<(String, String)> {
    let verifier = crypto::random_token("");
    let redirect = core.authorize(
        session,
        Authorization {
            response_type: "code".into(),
            client_id: CLIENT.into(),
            redirect_uri: REDIRECT.into(),
            scope: "openid offline_access".into(),
            state: Some("native-recovery-drill".into()),
            nonce: Some("native-recovery-nonce".into()),
            code_challenge: digest(&verifier),
            code_challenge_method: "S256".into(),
            decision: Some("approve".into()),
            ..Default::default()
        },
    )?;
    let code = url::Url::parse(&redirect)?
        .query_pairs()
        .find(|(key, _)| key == "code")
        .context("Authorization did not return a code")?
        .1
        .into_owned();
    Ok((code, verifier))
}

fn issue(config: &Path, input: &Path, output: &Path) -> Result<()> {
    let input = read(input)?;
    let mut core = Core::open(Config::load(config)?)?;
    let admin = core.login(
        "admin".into(),
        field(&input, "admin_password")?.into(),
        None,
    )?;
    let admin_session = field(&admin, "session_token")?;
    core.create_client(
        admin_session,
        NewClient {
            client_id: CLIENT.into(),
            name: "Native recovery drill".into(),
            confidential: false,
            redirect_uris: vec![REDIRECT.into()],
            scopes: ["openid", "offline_access"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            allowed_groups: BTreeSet::new(),
            require_mfa: false,
            service: false,
            settings: Default::default(),
        },
    )?;
    core.create_user(
        admin_session,
        NewUser {
            username: "drill-user".into(),
            password: field(&input, "user_password")?.into(),
            email: Some("drill-user@example.test".into()),
            display_name: "Drill User".into(),
            admin: false,
        },
    )?;
    let session = core.login(
        "drill-user".into(),
        field(&input, "user_password")?.into(),
        None,
    )?;
    let session = field(&session, "session_token")?;
    let (redeemed_code, redeemed_verifier) = authorization(&core, session)?;
    let tokens = core.token(TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some(CLIENT.into()),
        code: Some(redeemed_code),
        redirect_uri: Some(REDIRECT.into()),
        code_verifier: Some(redeemed_verifier),
        ..Default::default()
    })?;
    let (pending_code, pending_verifier) = authorization(&core, session)?;

    // Keep delivery local and queued; the server's own config has no SMTP worker.
    core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Drill <drill@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    core.account_invite(
        admin_session,
        Invitation {
            username: "proof-user".into(),
            email: "proof-user@example.test".into(),
            display_name: "Proof User".into(),
            groups: BTreeSet::new(),
        },
    )?;
    let proof = core
        .store
        .list::<Value>("mail_deliveries")?
        .into_iter()
        .filter_map(|(_, delivery)| {
            let body = delivery["body"].as_str()?;
            body.lines()
                .find(|line| line.starts_with("ri_mail_"))
                .map(str::to_owned)
        })
        .next()
        .context("Invitation proof was not queued")?;
    let account = user(&core, "drill-user")?;
    ensure!(
        core.me(session).is_ok(),
        "Fixture session did not authenticate"
    );
    ensure!(
        core.userinfo(field(&tokens, "access_token")?).is_ok(),
        "Fixture access token did not authenticate"
    );
    ensure!(
        core.store
            .get::<Value>("refresh", &digest(field(&tokens, "refresh_token")?))?
            .is_some(),
        "Fixture refresh token is missing"
    );
    ensure!(
        core.store
            .get::<Value>("codes", &digest(&pending_code))?
            .is_some(),
        "Fixture pending code is missing"
    );
    ensure!(
        core.store
            .get::<Value>("account_proofs", &digest(&proof))?
            .is_some(),
        "Fixture one-use proof is missing"
    );
    save(
        output,
        &json!({
            "session": session,
            "access": field(&tokens, "access_token")?,
            "refresh": field(&tokens, "refresh_token")?,
            "code": pending_code,
            "verifier": pending_verifier,
            "proof": proof,
            "user_id": account.id,
            "subjects": account.subjects,
            "pairwise_seed": account.pairwise_seed,
            "epoch": account.epoch,
            "password_hash_digest": digest(&account.password_hash),
            "active_kid": active_kid(&core)?,
        }),
    )
}

fn post(config: &Path, input: &Path, output: &Path) -> Result<()> {
    let input = read(input)?;
    let core = Core::open(Config::load(config)?)?;
    let admin = core.login(
        "admin".into(),
        field(&input, "admin_password")?.into(),
        None,
    )?;
    let admin_session = field(&admin, "session_token")?;
    core.update_user(
        admin_session,
        "drill-user",
        UserPatch {
            password: Some(field(&input, "new_user_password")?.into()),
            ..Default::default()
        },
    )?;
    core.rotate_key(admin_session)?;
    core.create_user(
        admin_session,
        NewUser {
            username: "late-user".into(),
            password: field(&input, "late_user_password")?.into(),
            email: None,
            display_name: "Late User".into(),
            admin: false,
        },
    )?;
    let account = user(&core, "drill-user")?;
    ensure!(
        core.store
            .get::<String>("usernames", "late-user")?
            .is_some(),
        "The post-snapshot user is missing"
    );
    save(
        output,
        &json!({
            "user_id": account.id,
            "password_hash_digest": digest(&account.password_hash),
            "active_kid": active_kid(&core)?,
            "late_user_present": true,
        }),
    )
}

fn rejected<T>(result: riauth::error::Result<T>) -> Result<String> {
    Ok(result
        .err()
        .context("Restored artifact unexpectedly authenticated")?
        .code
        .to_owned())
}

fn verify(config: &Path, artifacts: &Path, post: &Path, output: &Path) -> Result<()> {
    let artifacts = read(artifacts)?;
    let post = read(post)?;
    let core = Core::open(Config::load(config)?)?;
    let account = user(&core, "drill-user")?;
    let recovered_kid = active_kid(&core)?;
    let identity_preserved = account.id == field(&artifacts, "user_id")?
        && serde_json::to_value(&account.subjects)? == artifacts["subjects"]
        && account.pairwise_seed == field(&artifacts, "pairwise_seed")?;
    let epoch_advanced = account.epoch
        == artifacts["epoch"]
            .as_u64()
            .context("Missing original epoch")?
            + STRIDE;
    let stale_password = digest(&account.password_hash)
        == field(&artifacts, "password_hash_digest")?
        && digest(&account.password_hash) != field(&post, "password_hash_digest")?;
    let stale_signing_key = recovered_kid == field(&artifacts, "active_kid")?
        && recovered_kid != field(&post, "active_kid")?;
    let late_user_absent = core
        .store
        .get::<String>("usernames", "late-user")?
        .is_none();
    ensure!(
        identity_preserved
            && epoch_advanced
            && stale_password
            && stale_signing_key
            && late_user_absent,
        "Restored identity or stale-credential comparison failed"
    );
    let session_code = rejected(core.me(field(&artifacts, "session")?))?;
    let access_code = rejected(core.userinfo(field(&artifacts, "access")?))?;
    let refresh_code = rejected(core.token(TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some(CLIENT.into()),
        refresh_token: Some(field(&artifacts, "refresh")?.into()),
        ..Default::default()
    }))?;
    let authorization_code = rejected(core.token(TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some(CLIENT.into()),
        code: Some(field(&artifacts, "code")?.into()),
        redirect_uri: Some(REDIRECT.into()),
        code_verifier: Some(field(&artifacts, "verifier")?.into()),
        ..Default::default()
    }))?;
    let proof_code = rejected(core.account_complete(
        field(&artifacts, "proof")?.into(),
        Purpose::Invite,
        Some("new-fixture-password-123".into()),
    ))?;
    ensure!(
        core.store
            .get::<Value>("account_proofs", &digest(field(&artifacts, "proof")?))?
            .is_none(),
        "Restored one-use proof survived invalidation"
    );
    save(
        output,
        &json!({
            "identity_preserved": identity_preserved,
            "epoch_advanced": epoch_advanced,
            "late_user_absent": late_user_absent,
            "stale_password": stale_password,
            "stale_signing_key": stale_signing_key,
            "session_refusal": session_code,
            "bearer_refusal": access_code,
            "refresh_refusal": refresh_code,
            "code_refusal": authorization_code,
            "one_use_proof_refusal": proof_code,
            "one_use_proof_removed": true,
            "reconciliation_complete": false,
        }),
    )
}

fn hold(config: &Path, ready: &Path) -> Result<()> {
    let core = Core::open(Config::load(config)?)?;
    ensure!(
        core.store.get::<u32>("meta", "schema")?.is_some(),
        "Held store has no schema"
    );
    write_private(ready, b"ready", false)?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    drop(core);
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, config, input, output] if mode == "issue" => {
            issue(Path::new(config), Path::new(input), Path::new(output))
        }
        [mode, config, input, output] if mode == "post" => {
            post(Path::new(config), Path::new(input), Path::new(output))
        }
        [mode, config, artifacts, post, output] if mode == "verify" => verify(
            Path::new(config),
            Path::new(artifacts),
            Path::new(post),
            Path::new(output),
        ),
        [mode, config, ready] if mode == "hold" => hold(Path::new(config), Path::new(ready)),
        _ => anyhow::bail!("usage: recovery_native_artifacts issue|post|verify|hold ..."),
    }
}
