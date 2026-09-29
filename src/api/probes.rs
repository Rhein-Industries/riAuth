use super::*;

pub(super) async fn live(State(app): State<App>) -> Json<Value> {
    Json(probe_ok(&app, false))
}

fn unavailable(authentication: bool) -> Error {
    Error::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "not_ready",
        if authentication {
            "Service is not ready to accept authentication requests"
        } else {
            "Worker storage is not ready"
        },
    )
}

fn probe_ok(app: &App, include_issuer: bool) -> Value {
    let role = app.core.config.process.role;
    let duties = role.duties();
    let mut body = json!({
        "status": "ok",
        "service": "riAuth",
        "version": env!("CARGO_PKG_VERSION"),
        "role": role.as_str(),
        "duties": {
            "authentication": duties.authentication,
            "protocol_listeners": duties.protocol_listeners,
            "background_jobs": duties.background_jobs,
        },
    });
    if include_issuer {
        body["issuer"] = json!(app.core.config.issuer);
    }
    body
}

pub(super) async fn ready(State(app): State<App>) -> Result<Json<Value>> {
    let authentication = app.core.config.process.role.duties().authentication;
    if authentication && app.workers.available_permits() == 0 {
        return Err(unavailable(true));
    }
    // A timed-out blocking check retains its permit until the database call ends.
    // Repeated probes cannot accumulate unbounded detached storage work.
    let permit = app
        .probes
        .clone()
        .try_acquire_owned()
        .map_err(|_| unavailable(authentication))?;
    let core = app.core.clone();
    let check = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        core.store.ready()
    });
    tokio::time::timeout(Duration::from_secs(2), check)
        .await
        .map_err(|_| unavailable(authentication))?
        .map_err(|_| unavailable(authentication))?
        .map_err(|_| unavailable(authentication))?;
    Ok(Json(probe_ok(&app, authentication)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn saturated_application_or_probe_workers_do_not_disable_liveness() {
        let directory = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            crate::config::Config {
                data_dir: directory.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "probe-test-password-only".into(),
                email: None,
                display_name: "Admin".into(),
                admin: true,
            },
        )
        .unwrap();
        let app = App::new(core);
        let busy = app.workers.clone().acquire_many_owned(8).await.unwrap();
        assert_eq!(
            ready(State(app.clone())).await.unwrap_err().status,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(live(State(app.clone())).await.0["status"], "ok");
        drop(busy);
        let probes = app.probes.clone().acquire_many_owned(2).await.unwrap();
        assert_eq!(
            ready(State(app.clone())).await.unwrap_err().status,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(live(State(app.clone())).await.0["status"], "ok");
        drop(probes);
        let body = ready(State(app)).await.unwrap().0;
        assert_eq!(body["role"], "integrated");
        assert_eq!(body["duties"]["authentication"], true);
        assert_eq!(body["duties"]["background_jobs"], true);
        assert!(body["issuer"].as_str().unwrap().starts_with("http://"));
    }
}
