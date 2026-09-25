//! Real Chromium checks for the embedded portal and its terminal sign-in.
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::{SinkExt, StreamExt};
use riauth::{
    config::{Config, write_private},
    core::Core,
    model::*,
    portal::Settings,
};
use serde_json::{Value, json};
use std::{
    future::IntoFuture,
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

struct Browser(Child);
impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
struct Cdp {
    socket: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    sequence: u64,
    errors: Vec<Value>,
}
impl Cdp {
    async fn call(&mut self, method: &str, params: Value) -> Value {
        self.sequence += 1;
        let id = self.sequence;
        self.socket
            .send(Message::Text(
                json!({"id":id,"method":method,"params":params})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let message = self.socket.next().await.unwrap().unwrap();
                if let Message::Text(text) = message {
                    let value: Value = serde_json::from_str(&text).unwrap();
                    if value["method"] == "Runtime.exceptionThrown"
                        || (value["method"] == "Log.entryAdded"
                            && value["params"]["entry"]["level"] == "error")
                    {
                        self.errors.push(value.clone());
                    }
                    if value["id"] == id {
                        assert!(value["error"].is_null(), "CDP failed: {value}");
                        return value["result"].clone();
                    }
                }
            }
        })
        .await
        .expect("CDP timeout")
    }
    async fn eval(&mut self, expression: &str) -> Value {
        let result = self
            .call(
                "Runtime.evaluate",
                json!({"expression":expression,"returnByValue":true,"awaitPromise":true}),
            )
            .await;
        assert!(
            result["exceptionDetails"].is_null(),
            "JavaScript failed: {result}"
        );
        result["result"]["value"].clone()
    }
    async fn wait(&mut self, expression: &str) {
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                if self.eval(expression).await == true {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(75)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("UI condition timed out: {expression}"));
    }
    async fn screenshot(&mut self, name: &str) {
        if let Ok(dir) = std::env::var("RIAUTH_PORTAL_SCREENSHOTS") {
            std::fs::create_dir_all(&dir).unwrap();
            let data = self
                .call(
                    "Page.captureScreenshot",
                    json!({"format":"png","captureBeyondViewport":false}),
                )
                .await;
            std::fs::write(
                std::path::Path::new(&dir).join(format!("{name}.png")),
                STANDARD.decode(data["data"].as_str().unwrap()).unwrap(),
            )
            .unwrap();
        }
    }
    async fn width(&mut self, width: u32, height: u32) {
        self.call(
            "Emulation.setDeviceMetricsOverride",
            json!({"width":width,"height":height,"deviceScaleFactor":1,"mobile":false}),
        )
        .await;
        self.call("Runtime.evaluate",json!({"expression":"new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)))","awaitPromise":true})).await;
        assert_eq!(
            self.eval("document.documentElement.scrollWidth <= window.innerWidth")
                .await,
            true,
            "horizontal overflow at {width}px"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Set RIAUTH_TEST_BROWSER to Chrome/Chromium; optional RIAUTH_PORTAL_SCREENSHOTS directory"]
async fn portal_terminal_sign_in_access_changes_and_responsive_interactions() {
    let browser = std::env::var("RIAUTH_TEST_BROWSER").expect("RIAUTH_TEST_BROWSER required");
    let dir = tempfile::TempDir::new().unwrap();
    let profile = dir.path().join("chrome");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}/identity", listener.local_addr().unwrap());
    let core = Core::initialize(
        Config {
            issuer: issuer.clone(),
            listen: listener.local_addr().unwrap(),
            data_dir: dir.path().join("data"),
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: "browser-portal-fixture-password".into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = core
        .login(
            "admin".into(),
            "browser-portal-fixture-password".into(),
            None,
        )
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    core.create_user(
        &admin,
        NewUser {
            username: "alex.morgan".into(),
            password: "browser-user-fixture-password".into(),
            email: None,
            display_name: "Alex Morgan".into(),
            admin: false,
        },
    )
    .unwrap();
    let login = core
        .login(
            "alex.morgan".into(),
            "browser-user-fixture-password".into(),
            None,
        )
        .unwrap();
    let token = login["session_token"].as_str().unwrap();
    let session_file = dir.path().join("session.json");
    write_private(
        &session_file,
        &serde_json::to_vec(
            &json!({"issuer":issuer,"token":token,"expires_at":login["expires_at"]}),
        )
        .unwrap(),
        false,
    )
    .unwrap();
    core.create_group(&admin, "engineering").unwrap();
    core.group_member(&admin, "engineering", "alex.morgan", true)
        .unwrap();
    for (id, name, description, category, icon, accent) in [
        (
            "code",
            "Code workspace",
            "Build, review, and ship great work together.",
            "Engineering",
            "code",
            "violet",
        ),
        (
            "metrics",
            "Team analytics",
            "A clear view of the numbers that matter.",
            "Insights",
            "chart",
            "amber",
        ),
        (
            "docs",
            "Knowledge base",
            "The answers, guides, and ideas your team shares.",
            "Productivity",
            "book",
            "blue",
        ),
        (
            "chat",
            "Team conversations",
            "Stay close to the people and projects you work with.",
            "Communication",
            "messages",
            "rose",
        ),
        (
            "files",
            "Shared files",
            "Everything you’re working on, in one place.",
            "Productivity",
            "files",
            "amber",
        ),
        (
            "cloud",
            "Cloud console",
            "Your infrastructure, services, and environments.",
            "Engineering",
            "cloud",
            "teal",
        ),
        (
            "terminal",
            "Developer tools",
            "A starting point for your development workflow.",
            "Engineering",
            "terminal",
            "slate",
        ),
        (
            "security",
            "Security center",
            "Keep an eye on your team’s security posture.",
            "Insights",
            "shield",
            "teal",
        ),
        (
            "projects",
            "Project hub",
            "Turn plans into progress with your team.",
            "Productivity",
            "app",
            "blue",
        ),
        (
            "secret",
            "Restricted finance",
            "This must never appear to Alex.",
            "Finance",
            "chart",
            "rose",
        ),
    ] {
        let mut settings = ProviderSettings {
            app: Some(Settings {
                description: description.into(),
                category: category.into(),
                icon: icon.into(),
                accent: accent.into(),
                launch_url: Some(format!("https://{id}.example.test/")),
                ..Default::default()
            }),
            ..Default::default()
        };
        if id == "secret" {
            settings
                .policy
                .access
                .denied_users
                .insert("alex.morgan".into());
        }
        core.create_client(
            &admin,
            NewClient {
                client_id: id.into(),
                name: name.into(),
                confidential: false,
                redirect_uris: vec![format!("https://{id}.example.test/callback")],
                scopes: ["openid".into(), "profile".into()].into(),
                allowed_groups: if id == "cloud" {
                    ["engineering".into()].into()
                } else {
                    Default::default()
                },
                require_mfa: false,
                service: false,
                settings,
            },
        )
        .unwrap();
    }
    let server =
        tokio::spawn(axum::serve(listener, riauth::api::router(core.clone())).into_future());
    let _browser = Browser(
        Command::new(browser)
            .args([
                "--headless=new",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-background-networking",
                "--disable-component-update",
                "--remote-debugging-port=0",
            ])
            .arg(format!("--user-data-dir={}", profile.display()))
            .arg("about:blank")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let port = tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if let Ok(text) = std::fs::read_to_string(profile.join("DevToolsActivePort")) {
                break text.lines().next().unwrap().parse::<u16>().unwrap();
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    let targets: Value = reqwest::get(format!("http://127.0.0.1:{port}/json/list"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let target = targets
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["type"] == "page")
        .unwrap();
    let (socket, _) =
        tokio_tungstenite::connect_async(target["webSocketDebuggerUrl"].as_str().unwrap())
            .await
            .unwrap();
    let mut cdp = Cdp {
        socket,
        sequence: 0,
        errors: vec![],
    };
    cdp.call("Page.enable", json!({})).await;
    cdp.call("Runtime.enable", json!({})).await;
    cdp.width(1440, 1000).await;
    cdp.call("Page.navigate", json!({"url":format!("{issuer}/apps")}))
        .await;
    cdp.wait("document.getElementById('auth')?.hidden === false")
        .await;
    cdp.screenshot("portal-sign-in").await;
    cdp.eval("document.getElementById('start-login').click()")
        .await;
    cdp.wait("document.getElementById('user-code')?.textContent.length > 4")
        .await;
    let code = cdp
        .eval("document.getElementById('user-code').textContent")
        .await;
    let output = Command::new(env!("CARGO_BIN_EXE_riauth"))
        .args([
            "--server",
            &issuer,
            "--session-file",
            session_file.to_str().unwrap(),
            "--non-interactive",
            "--json",
            "portal",
            "approve",
            code.as_str().unwrap(),
            "--yes",
        ])
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    cdp.wait("document.querySelectorAll('.app-card').length === 9")
        .await;
    assert_eq!(
        cdp.eval("document.getElementById('account-name').textContent")
            .await,
        "Alex Morgan"
    );
    assert_eq!(
        cdp.eval("document.body.textContent.includes('Restricted finance')")
            .await,
        false
    );
    assert_eq!(
        cdp.eval("document.cookie.includes('riauth_sso')").await,
        false
    );
    cdp.screenshot("portal-desktop").await;
    cdp.eval("document.querySelector('[data-favorite=code]').click(); document.querySelector('[data-favorite=docs]').click(); document.getElementById('nav-favorites').click()").await;
    assert_eq!(
        cdp.eval("document.querySelectorAll('.app-card').length")
            .await,
        2
    );
    cdp.screenshot("portal-favorites").await;
    cdp.eval("document.getElementById('nav-all').click(); document.getElementById('search').value='analytics'; document.getElementById('search').dispatchEvent(new Event('input'))").await;
    assert_eq!(
        cdp.eval("document.querySelectorAll('.app-card').length")
            .await,
        1
    );
    cdp.eval("document.getElementById('search').value='no application with this name'; document.getElementById('search').dispatchEvent(new Event('input'))").await;
    assert_eq!(
        cdp.eval("document.getElementById('empty').hidden").await,
        false
    );
    cdp.screenshot("portal-empty-search").await;
    cdp.eval("document.getElementById('reset-filters').click(); document.getElementById('category').value='Engineering'; document.getElementById('category').dispatchEvent(new Event('change'))").await;
    assert_eq!(
        cdp.eval("document.querySelectorAll('.app-card').length")
            .await,
        3
    );
    cdp.eval("document.getElementById('category').value=''; document.getElementById('category').dispatchEvent(new Event('change')); document.getElementById('view-list').click()").await;
    cdp.screenshot("portal-list").await;
    cdp.call("Page.reload", json!({})).await;
    cdp.wait("document.querySelectorAll('.app-card').length === 9")
        .await;
    assert_eq!(cdp.eval("document.getElementById('apps').classList.contains('list') && document.getElementById('favorite-count').textContent === '2'").await,true);
    cdp.eval("document.getElementById('view-grid').click()")
        .await;
    cdp.eval("document.getElementById('view-grid').focus()")
        .await;
    cdp.call(
        "Input.dispatchKeyEvent",
        json!({"type":"keyDown","key":"/","code":"Slash"}),
    )
    .await;
    assert_eq!(cdp.eval("document.activeElement.id").await, "search");
    for width in [320, 390, 600, 768, 1024, 1440] {
        cdp.width(width, 900).await;
        assert_eq!(cdp.eval("[...document.querySelectorAll('.app-card')].every(c => {const r=c.getBoundingClientRect(); return r.left>=0 && r.right<=innerWidth;})").await,true);
        if width == 390 {
            cdp.screenshot("portal-mobile").await;
        }
        if width == 768 {
            cdp.screenshot("portal-tablet").await;
        }
    }
    // Losing the connection clears old cards; retry restores the user's preferences.
    cdp.call("Network.enable", json!({})).await;
    cdp.call(
        "Network.emulateNetworkConditions",
        json!({"offline":true,"latency":0,"downloadThroughput":0,"uploadThroughput":0}),
    )
    .await;
    cdp.eval("document.getElementById('refresh').click()").await;
    cdp.wait("document.getElementById('error').hidden === false")
        .await;
    assert_eq!(
        cdp.eval("document.querySelectorAll('.app-card').length")
            .await,
        0
    );
    cdp.screenshot("portal-connection-error").await;
    cdp.call(
        "Network.emulateNetworkConditions",
        json!({"offline":false,"latency":0,"downloadThroughput":-1,"uploadThroughput":-1}),
    )
    .await;
    cdp.eval("document.getElementById('retry').click()").await;
    cdp.wait("document.querySelectorAll('.app-card').length === 9")
        .await;
    // A live group removal changes the catalogue without requiring another sign-in.
    core.group_member(&admin, "engineering", "alex.morgan", false)
        .unwrap();
    cdp.eval("document.getElementById('refresh').click()").await;
    cdp.wait("document.querySelectorAll('.app-card').length === 8")
        .await;
    assert_eq!(
        cdp.eval("document.querySelector('[data-launch=cloud]') === null")
            .await,
        true
    );
    // User-controlled names/descriptions must remain plain text even with HTML payloads.
    let mut settings = core
        .store
        .get::<Client>("clients", "code")
        .unwrap()
        .unwrap()
        .settings;
    settings.app.as_mut().unwrap().description = "<img src=x onerror=alert(1)> & plain text".into();
    core.update_client(
        &admin,
        "code",
        ClientPatch {
            name: Some("<svg onload=alert(1)>".into()),
            settings: Some(settings),
            ..Default::default()
        },
    )
    .unwrap();
    cdp.eval("document.getElementById('refresh').click()").await;
    cdp.wait("document.getElementById('apps').textContent.includes('<svg onload=alert(1)>')")
        .await;
    assert_eq!(
        cdp.eval("document.querySelectorAll('#apps img, #apps [onload], #apps [onerror]').length")
            .await,
        0
    );
    // Sign-out ignores clicks for a moment after the catalogue appears (double-click-jacking).
    cdp.wait("!document.getElementById('sign-out').hasAttribute('aria-disabled')")
        .await;
    cdp.eval("document.getElementById('sign-out').click()")
        .await;
    cdp.wait("document.getElementById('auth').hidden === false")
        .await;
    assert_eq!(
        cdp.eval("document.querySelectorAll('.app-card').length")
            .await,
        0
    );
    assert!(core.me(token).is_err());
    // Signing another account into the same browser cannot inherit Alex's favorites.
    cdp.eval("document.getElementById('start-login').click()")
        .await;
    cdp.wait("document.getElementById('user-code').textContent.length > 4")
        .await;
    let code = cdp
        .eval("document.getElementById('user-code').textContent")
        .await;
    core.portal_decide(&admin, code.as_str().unwrap(), true)
        .unwrap();
    cdp.wait("document.getElementById('account-name').textContent === 'Administrator'")
        .await;
    assert_eq!(
        cdp.eval("document.getElementById('favorite-count').textContent")
            .await,
        "0"
    );
    assert_eq!(
        cdp.eval("document.getElementById('apps').textContent.includes('Restricted finance')")
            .await,
        true
    );
    // Sign-out ignores clicks for a moment after the catalogue appears (double-click-jacking).
    cdp.wait("!document.getElementById('sign-out').hasAttribute('aria-disabled')")
        .await;
    cdp.eval("document.getElementById('sign-out').click()")
        .await;
    cdp.wait("document.getElementById('auth').hidden === false")
        .await;
    assert!(cdp.errors.is_empty(), "Browser errors: {:?}", cdp.errors);
    server.abort();
}
