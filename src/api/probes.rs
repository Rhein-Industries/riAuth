use super::*;

pub(super) async fn live() -> Json<Value> {
    Json(json!({"status": "ok", "service": "riAuth", "version": env!("CARGO_PKG_VERSION")}))
}

fn unavailable() -> Error {
    Error::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "not_ready",
        "Service is not ready to accept authentication requests",
    )
}

pub(super) async fn ready(State(app): State<App>) -> Result<Json<Value>> {
    if app.workers.available_permits() == 0 {
        return Err(unavailable());
    }
    // A timed-out blocking check retains its permit until the database call ends.
    // Repeated probes cannot accumulate unbounded detached storage work.
    let permit = app
        .probes
        .clone()
        .try_acquire_owned()
        .map_err(|_| unavailable())?;
    let core = app.core.clone();
    let check = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        core.store.ready()
    });
    tokio::time::timeout(Duration::from_secs(2), check)
        .await
        .map_err(|_| unavailable())?
        .map_err(|_| unavailable())?
        .map_err(|_| unavailable())?;
    Ok(Json(
        json!({"status": "ok", "service": "riAuth", "version": env!("CARGO_PKG_VERSION"), "issuer": app.core.config.issuer}),
    ))
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
        assert_eq!(live().await.0["status"], "ok");
        drop(busy);
        let probes = app.probes.clone().acquire_many_owned(2).await.unwrap();
        assert_eq!(
            ready(State(app.clone())).await.unwrap_err().status,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(live().await.0["status"], "ok");
        drop(probes);
        assert!(ready(State(app)).await.is_ok());
    }
}
