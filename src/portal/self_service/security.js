"use strict";
(() => {
  const $ = (id) => document.getElementById(id);
  const { base } = RiAuth;
  let snapshot = null, generation = 0, providers = null;
  const passkey = RiAuth.passkeyFlow(
    () => RiAuth.post("api/portal/login/passkey/start", { reauthenticate: true }, { retry: true }),
    (credential, started) => RiAuth.post("api/portal/login/passkey/finish", { ceremony: started.ceremony, credential }),
    false,
    (started) => RiAuth.post("api/portal/login/passkey/cancel", { ceremony: started.ceremony })
  );

  function node(tag, className, text) {
    const item = document.createElement(tag);
    if (className) item.className = className;
    if (text !== undefined) item.textContent = text;
    return item;
  }
  function showStatus(message, url) {
    $("status").replaceChildren(document.createTextNode(message));
    if (url) {
      const target = new URL(url, location.origin);
      if (target.origin === location.origin && target.pathname.startsWith(`${base}saml/logout/`)) {
        const link = node("a", "", " Finish connected-app sign-out");
        link.href = target.href; $("status").append(link);
      }
    }
    $("status").hidden = false;
  }
  function signedOut(message) {
    snapshot = null;
    $("security-admin-link").hidden = $("security-events-link").hidden = true;
    $("loading").hidden = $("security-content").hidden = true;
    $("signed-out").hidden = false;
    if (message) showStatus(message); else $("status").hidden = true;
  }
  function date(seconds) { return new Date(seconds * 1000).toLocaleString(); }
  function binding() {
    return { expected_user_id: snapshot.user.id, expected_session_id: snapshot.current_session_id };
  }
  function showVerify() {
    $("verify-panel").hidden = false;
    $("verify-panel").scrollIntoView({ block: "nearest", behavior: "smooth" });
    ($("verify-passkey").hidden ? $("verify-password") : $("verify-passkey")).focus();
  }
  function render(data) {
    snapshot = data;
    $("security-admin-link").hidden = data.user.admin !== true;
    $("security-events-link").hidden = data.user.admin !== true || !RiAuthCapabilities.usable("audit.self_hosted_event_map");
    $("loading").hidden = $("signed-out").hidden = true;
    $("security-content").hidden = false;
    $("account-line").textContent = `Signed in as ${data.user.display_name} (@${data.user.username})`;
    $("verify-panel").hidden = data.can_manage;
    $("verify-hint").textContent = data.browser_owned
      ? "Sign in again in this browser before changing sessions or consent. Then choose your action again."
      : "This browser shares a terminal sign-in. Confirm with a passkey or password here first; your terminal sign-in remains available until you revoke it.";
    const passkeyAvailable = RiAuthCapabilities.usable("identity.passkeys") && RiAuth.passkeysAvailable();
    $("verify-passkey").hidden = !passkeyAvailable;
    $("verify-form").hidden = !data.password_available;
    if (!data.password_available && !passkeyAvailable) {
      $("verify-hint").textContent = "This account has no browser sign-in method available here. Use an enrolled passkey in a supported browser, or contact your administrator.";
    }
    $("verify-error").hidden = true;
    const sessions = data.sessions.map((session) => {
      const row = node("li", "security-row"), details = node("div");
      const title = node("strong", "", session.kind === "terminal" ? "Terminal sign-in" : "Browser sign-in");
      if (session.current) title.append(node("span", "security-badge", "This browser"));
      details.append(title, node("p", "", `Signed in ${date(session.auth_time)} · Expires ${date(session.expires_at)}${session.mfa ? " · Extra verification" : ""}`));
      const button = node("button", "button secondary", "Revoke session");
      button.type = "button";
      button.setAttribute("aria-label", `Revoke ${session.current ? "this browser's" : session.kind} session signed in ${date(session.auth_time)}`);
      RiAuth.guard(button, () => act(button, `sessions/${encodeURIComponent(session.id)}/revoke`,
        session.current ? "Sign out this browser session?" : "Revoke this session?",
        "Session revoked. Your other riAuth sessions remain available."));
      row.append(details, button); return row;
    });
    $("session-list").replaceChildren(...sessions);
    $("sessions-empty").hidden = sessions.length > 0;
    const consents = data.consents.map((consent) => {
      const row = node("li", "security-row"), details = node("div");
      const protocol = consent.protocol === "saml" ? "SAML" : "OAuth";
      details.append(node("strong", "", consent.name), node("p", "", `${protocol} approval · Expires ${date(consent.expires_at)}`));
      if (Array.isArray(consent.scopes) && consent.scopes.length) details.append(node("p", "", `Permissions: ${consent.scopes.join(", ")}`));
      if (consent.resource) details.append(node("p", "", `Resource: ${consent.resource}`));
      const button = node("button", "button secondary", "Withdraw");
      button.type = "button"; button.setAttribute("aria-label", `Withdraw remembered consent for ${consent.name}`);
      RiAuth.guard(button, () => act(button, `consents/${encodeURIComponent(consent.client_id)}/withdraw`,
        `Withdraw remembered consent for ${consent.name}?`,
        "Remembered consent withdrawn. riAuth grants for this application were revoked."));
      row.append(details, button); return row;
    });
    $("consent-list").replaceChildren(...consents);
    $("consents-empty").hidden = consents.length > 0;
    RiAuthCapabilities.apply();
  }
  async function load() {
    const mine = ++generation;
    $("status").hidden = true;
    $("loading").hidden = false;
    try {
      await RiAuthCapabilities.refresh().catch(() => {});
      const data = await RiAuth.get("api/portal/security");
      if (mine === generation) { render(data); await loadProviders(); }
    } catch (error) {
      if (mine !== generation) return;
      if (error.status === 401) signedOut();
      else {
        snapshot = null; $("loading").hidden = true; $("security-content").hidden = true;
        showStatus("Could not load your sessions and consent. Reload this page to try again.");
      }
    }
  }
  // Linked sign-in providers. Linking and unlinking need this browser's own recent local
  // sign-in, bound to the account shown; starting a link only follows the provider's URL.
  function host(issuer) { try { return new URL(issuer).host; } catch { return issuer; } }
  function providerBinding() {
    return { expected_user_id: providers.user.id, expected_session_id: providers.current_session_id };
  }
  function renderProviders(data) {
    providers = data;
    const rows = data.links.map((link) => {
      const row = node("li", "security-row"), details = node("div");
      details.append(node("strong", "", link.name), node("p", "", `Provider account ${link.subject} at ${host(link.issuer)}`));
      const button = node("button", "button secondary", "Unlink");
      button.type = "button"; button.setAttribute("aria-label", `Unlink ${link.name}`);
      RiAuth.guard(button, () => changeProvider(button, async () => {
        if (!window.confirm(`Unlink ${link.name}? You won't be able to sign in with it any more, and the sessions it started end now.`)) return;
        await RiAuth.post(`api/portal/sources/links/${encodeURIComponent(link.id)}/unlink`, providerBinding());
        await loadProviders(); showStatus(`${link.name} unlinked. Sessions it started have ended.`);
      }));
      row.append(details, button); return row;
    });
    $("provider-list").replaceChildren(...rows);
    $("providers-empty").hidden = rows.length > 0;
    const buttons = data.linkable.map((source) => {
      const button = node("button", "button secondary", `Link ${source.name}`);
      button.type = "button";
      RiAuth.guard(button, () => changeProvider(button, async () => {
        const started = await RiAuth.post(`api/portal/sources/${encodeURIComponent(source.id)}/start`, { link: providerBinding() });
        const target = new URL(started.authorization_url);
        if (target.protocol !== "https:" && target.protocol !== "http:") throw new Error("Unexpected provider address");
        location.assign(target.href);
      }));
      return button;
    });
    $("provider-buttons").replaceChildren(...buttons);
    $("provider-link").hidden = buttons.length === 0;
    $("providers").hidden = rows.length === 0 && buttons.length === 0;
  }
  async function loadProviders() {
    try { renderProviders(await RiAuth.get("api/portal/sources/links")); }
    catch { providers = null; $("providers").hidden = true; }
  }
  async function changeProvider(button, run) {
    if (!providers) return;
    if (!providers.local_session) { showStatus("Sign in with your riAuth password or passkey in this browser to change linked providers."); return; }
    if (!providers.can_change) { showVerify(); return; }
    await RiAuth.inFlight(button, async () => {
      try { await run(); } catch (error) {
        if (error.status === 401) { signedOut(); return; }
        if (["account_changed", "session_changed"].includes(error.code)) { await load(); showStatus("Your sign-in changed. Review the current account and choose your action again."); return; }
        if (error.code === "reauthentication_required") { providers.can_change = false; showVerify(); return; }
        showStatus(error.description || "The change could not be completed. Try again.");
      }
    });
  }
  async function act(button, path, question, success) {
    if (!snapshot) return;
    if (!snapshot.can_manage) { showVerify(); return; }
    if (!window.confirm(question)) return;
    await RiAuth.inFlight(button, async () => {
      try {
        const result = await RiAuth.post(`api/portal/security/${path}`, binding());
        if (result.signed_out) {
          signedOut(path === "sessions/revoke-all"
            ? "Your riAuth sessions have ended. Connected applications may need to finish sign-out."
            : "This browser is signed out. Your other riAuth sessions remain available.");
        } else {
          await load(); showStatus(success, result.logout_url);
        }
        if (result.signed_out && typeof result.logout_url === "string") {
          const target = new URL(result.logout_url, location.origin);
          if (target.origin === location.origin && target.pathname.startsWith(`${base}saml/logout/`)) location.assign(target.href);
        }
      } catch (error) {
        if (error.status === 401) { signedOut(); return; }
        if (["account_changed", "session_changed"].includes(error.code)) { await load(); showStatus("Your sign-in changed. Review the current account and choose your action again."); return; }
        if (["reauthentication_required", "mfa_required"].includes(error.code)) { snapshot.can_manage = false; showVerify(); return; }
        showStatus(error.description || "The change could not be completed. Try again.");
      }
    });
  }
  RiAuth.guard($("revoke-all"), () => act($("revoke-all"), "sessions/revoke-all",
    "Sign out of every riAuth browser and terminal session? This also signs out this browser.",
    "Signed out everywhere."));
  $("verify-form").addEventListener("submit", (event) => {
    event.preventDefault();
    if (!snapshot) return;
    const password = $("verify-password").value;
    if (!password) { $("verify-error").textContent = "Enter your password."; $("verify-error").hidden = false; return; }
    RiAuth.inFlight($("verify-password-button"), async () => {
      try {
        await RiAuth.post("api/portal/login/password", {
          username: snapshot.user.username, password, otp: $("verify-otp").value.trim() || null, reauthenticate: true
        });
        $("verify-password").value = $("verify-otp").value = "";
        await load(); showStatus("Identity confirmed. Choose your action again.");
      } catch (error) {
        $("verify-error").textContent = error.description || "Could not confirm your identity. Try again.";
        $("verify-error").hidden = false;
        $("verify-error").focus();
      }
    });
  });
  $("verify-passkey").addEventListener("click", () => RiAuth.inFlight($("verify-passkey"), async () => {
    if (!snapshot) return;
    try {
      await passkey(); await load(); showStatus("Identity confirmed. Choose your action again.");
    } catch (error) {
      $("verify-error").textContent = error.name === "NotAllowedError" || error.name === "AbortError"
        ? "Passkey confirmation was cancelled or timed out. Try again."
        : error.description || "Could not confirm your identity with a passkey.";
      $("verify-error").hidden = false; $("verify-error").focus();
    }
  }));
  window.addEventListener("pageshow", (event) => { if (event.persisted) load(); });
  // A finished link returns here with ?linked; the address keeps no trace of it.
  const linked = new URLSearchParams(location.search).has("linked");
  if (linked) history.replaceState(history.state, "", location.pathname);
  load().then(() => { if (linked && providers) showStatus("Provider linked. You can now sign in with it."); });
})();
