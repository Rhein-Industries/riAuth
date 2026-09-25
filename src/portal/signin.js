"use strict";
// The interaction page for OIDC and SAML sign-in and for relying-party sign-out. It renders
// the server's state object; the only place it navigates to is `continue`, and only when
// that is this interaction's own resume path. Text is set with textContent only.
(() => {
  const $ = (id) => document.getElementById(id);
  const { base } = RiAuth;
  const SCOPES = { openid: "Confirm your riAuth identity", profile: "See your name and username", email: "See your email address", groups: "See your group memberships", offline_access: "Stay connected while you're away" };
  const HEADINGS = {
    sign_in: ["Sign in to continue to {App}", ""],
    prompt_login: ["Confirm it's you", "{App} asks you to sign in again."],
    force_authn: ["Confirm it's you", "{App} asks you to sign in again."],
    max_age: ["Confirm it's you", "Your sign-in is older than {App} allows."],
    step_up: ["Extra verification needed", "{App} requires a passkey or an authenticator code."],
    select_account: ["Choose an account for {App}", "Sign in with the account you want to use."]
  };
  const ACTIVE = new Set(["authenticate", "consent", "confirm"]);
  const FIELDS = ["signin-username", "signin-password", "signin-otp"];
  const page = (() => {
    for (const [kind, prefix] of [["authorize", "oauth/resume/"], ["saml", "saml/resume/"], ["logout", "oauth/logout/resume/"]]) {
      if (!location.pathname.startsWith(`${base}${prefix}`)) continue;
      const id = location.pathname.slice(`${base}${prefix}`.length);
      if (/^[0-9a-f-]{36}$/.test(id)) return { kind, api: `${prefix}${id}`, path: `${base}${prefix}${id}` };
    }
    return null;
  })();
  const noun = page?.kind === "logout" ? "sign-out request" : "sign-in request";
  // `generation` discards state fetched while a user action was in flight.
  let state = null, current = "loading", generation = 0, acting = 0, loading = false, again = false, leaving = false;
  let pollTimer, messageAction = null, flow = null, flowKey = null;

  const app = () => state?.application?.name || "the application";
  // Polling continues while the request can still change: open screens, and a request only
  // the terminal can approve.
  const waiting = () => !!state && (ACTIVE.has(state.status) || state.error === "step_up_unavailable");
  const fill = (text) => text.replaceAll("{App}", app());
  function announce(text) { $("signin-announcement").textContent = text; }
  function show(name, title) {
    for (const id of ["loading", "authenticate", "consent", "logout", "message"]) $(`signin-${id}`).hidden = id !== name;
    document.title = `${title} · riAuth`;
    if (current === name) return;
    current = name; RiAuth.arm();
    const heading = $(`signin-${name}`).querySelector("h1");
    heading.focus(); announce(heading.textContent);
  }
  // The terminal panel is one element, placed in whichever screen offers it.
  function terminal(section, before) {
    const panel = $("signin-terminal"), info = state?.terminal;
    panel.hidden = !section || !info;
    if (!section || !info) return;
    if (panel.parentElement !== $(section)) $(section).insertBefore(panel, before ? $(before) : null);
    $("signin-code").textContent = info.user_code;
    const command = page.kind === "logout" ? "logout-request approve" : "request approve";
    $("signin-command").textContent = `riauth --server ${RiAuth.shellQuote(info.issuer)} ${command} ${info.user_code}`;
  }
  function clearError(id) {
    $(id).hidden = true; $(id).replaceChildren();
    if (id === "signin-error") for (const field of FIELDS) $(field).removeAttribute("aria-invalid");
  }
  function showError(id, text, portal = false) {
    $(id).replaceChildren(text);
    if (portal) {
      const link = document.createElement("a");
      link.href = `${base}apps`; link.textContent = "Open your applications portal";
      $(id).append(" ", link);
    }
    $(id).hidden = false; $(id).focus();
  }
  function message(title, text, { action = null, link = false } = {}) {
    stop(); terminal(null);
    $("message-title").textContent = title; $("message-text").textContent = text;
    clearError("message-error");
    messageAction = action?.run ?? null;
    $("message-action").textContent = action?.label ?? ""; $("message-action").hidden = !action;
    $("message-link").hidden = !link; $("signin-expiry").hidden = true;
    show("message", title);
  }
  function expired() {
    state = null;
    message(`This ${noun} has expired`, `This ${noun} has expired. Return to the application and try again.`, { link: true });
  }
  function stop() { clearTimeout(pollTimer); pollTimer = undefined; }

  // State: fetched on load, on return to the tab and while polling (F21).
  async function load() {
    if (!page || acting || leaving) return;
    if (loading) { again = true; return; }
    loading = true; stop();
    const at = generation;
    try {
      const next = await RiAuth.get(`${page.api}/state`);
      if (at === generation && !acting) render(next);
    } catch (error) {
      if (at === generation && !acting) failed(error);
    } finally {
      loading = false;
      if (again && !leaving) { again = false; load(); } else schedule();
    }
  }
  function schedule() {
    stop();
    if (!waiting() || document.visibilityState !== "visible") return;
    // The server decides expiry (404 interaction_expired); a skewed client clock only
    // shortens the last wait.
    const left = state.expires_at * 1000 - Date.now();
    const wait = !$("signin-terminal").hidden && $("signin-terminal").open ? 2000 : 15000;
    pollTimer = setTimeout(load, left > 1000 ? Math.min(wait, left) : wait);
  }
  function failed(error) {
    if (error.status === 404) { expired(); return; }
    if (error.status === 401) {
      state = null;
      message(`This ${noun} belongs to another browser`, "Return to the application and start again in this browser.", { link: true });
      return;
    }
    // A failed poll keeps the current screen; the next poll retries.
    if (state) return;
    message(`Couldn't load this ${noun}`, describe(error, "Couldn't reach riAuth. Check your connection and try again."), { action: { label: "Try again", run: load } });
  }
  function render(next) {
    const previous = state;
    state = next;
    if (next.status === "complete" || next.status === "done") { proceed(next); return; }
    if (next.status === "unavailable") unavailable(next);
    else if (next.kind === "logout") logout(next);
    else if (next.status === "consent") consent(next);
    else authenticate(next, previous);
    if (waiting()) expiry();
    schedule();
  }
  function expiry() {
    const at = new Date(state.expires_at * 1000);
    $("signin-expiry").textContent = `This request expires at ${at.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`;
    $("signin-expiry").hidden = false;
  }
  // `continue` is a server-built path; anything but this interaction's resume path is refused.
  function proceed(next) {
    const target = typeof next.continue === "string" ? new URL(next.continue, location.origin) : null;
    if (!target || target.origin !== location.origin || target.pathname !== page.path) {
      message("Something went wrong", "riAuth couldn't continue this request. Return to the application and try again.", { link: true });
      return;
    }
    generation += 1; leaving = true;
    message(page.kind === "logout" ? "Finishing sign-out…" : `Returning to ${app()}…`, "This takes a moment.");
    location.replace(`${target.origin}${target.pathname}`);
  }

  function authenticate(next, previous) {
    const [title, reason] = HEADINGS[next.reason] || HEADINGS.sign_in;
    $("signin-title").textContent = fill(title);
    $("signin-reason").textContent = fill(reason); $("signin-reason").hidden = !reason;
    const host = next.application?.host;
    $("signin-app-host").textContent = host ? `You'll return to ${host}` : ""; $("signin-app-host").hidden = !host;
    const account = next.account, pinned = next.pinned === true && !!account;
    $("signin-account").hidden = !pinned;
    $("signin-account-text").textContent = pinned ? `Signed in as ${account.display_name} (@${account.username})` : "";
    // Keep what the user typed across polls; reset it when the account changes.
    const changed = current !== "authenticate" || previous?.session_ref !== next.session_ref || previous?.pinned !== next.pinned;
    $("signin-username").readOnly = pinned;
    if (pinned) $("signin-username").value = account.username;
    else if (changed) $("signin-username").value = account?.username || "";
    const mfa = next.requirements?.mfa === true;
    $("signin-otp").required = mfa;
    $("signin-requirement").hidden = !mfa;
    $("signin-requirement-text").textContent = `${app()} requires a passkey or an authenticator code. No passkey yet?`;
    $("signin-cancel").textContent = `Cancel and return to ${app()}`;
    const key = `${next.pinned}:${next.session_ref}`;
    if (flowKey !== key) {
      flowKey = key;
      flow = RiAuth.passkeyFlow(
        () => RiAuth.post(`${page.api}/passkey/start`, {}, { retry: true }),
        (credential, started) => RiAuth.post(`${page.api}/passkey/finish`, { ceremony: started.ceremony, credential }));
    }
    if (changed) clearError("signin-error");
    terminal("signin-authenticate", "signin-cancel");
    const moved = current === "authenticate" && changed;
    show("authenticate", `Sign in to ${app()}`);
    // Same screen, another account ("Use another account", or a sign-out elsewhere): announce
    // it, and move focus to the heading only when the focused control went away.
    if (moved) {
      const focused = document.activeElement;
      if (!focused || focused === document.body || focused.closest("[hidden]")) $("signin-title").focus();
      announce($("signin-title").textContent);
    }
  }
  function consent(next) {
    const c = next.consent || {}, required = c.required !== false, account = next.account;
    $("consent-title").textContent = required ? `${app()} wants to use your riAuth account` : `Continue to ${app()}?`;
    $("consent-account").textContent = account ? `Signed in as ${account.display_name} (@${account.username})` : "";
    $("consent-account").hidden = !account;
    const saml = next.kind === "saml";
    const items = saml ? (c.attributes || []).map((a) => a.friendly_name || a.name) : (c.scopes || []).map((s) => SCOPES[s] || `Use ${s}`);
    $("consent-intro").textContent = saml ? "Shares these attributes" : `This allows ${app()} to:`;
    $("consent-scopes").replaceChildren(...items.map((text) => { const li = document.createElement("li"); li.textContent = text; return li; }));
    $("consent-intro").hidden = $("consent-scopes").hidden = !required || !items.length;
    $("consent-resource").textContent = c.resource ? `For ${c.resource}` : ""; $("consent-resource").hidden = !required || !c.resource;
    const host = next.application?.host;
    $("consent-host").textContent = host ? `You'll return to ${host}` : ""; $("consent-host").hidden = !host;
    $("consent-remember-label").hidden = !required;
    if (current !== "consent") { $("consent-remember").checked = c.remember_default !== false; clearError("consent-error"); }
    $("consent-allow").textContent = required ? "Allow" : "Continue";
    $("consent-deny").textContent = required ? "Deny" : "Cancel";
    terminal(null);
    show("consent", `Sign in to ${app()}`);
  }
  function unavailable(next) {
    const [title, text] = {
      access_denied: [`You don't have access to ${app()}`, next.message || "Your account can't use this application."],
      step_up_unavailable: ["Continue in your terminal", `${app()} requires a sign-in method that's only available from your terminal.`],
      source_stage: ["Continue with your organization", "Finish signing in with your organization's provider."]
    }[next.error] || [`This ${noun} can't continue`, "The application's request is no longer valid. Return to the application and try again."];
    // Only a request the server can still answer has a way back; invalid_request has none.
    const back = ["access_denied", "step_up_unavailable"].includes(next.error) ? { label: `Return to ${app()}`, run: () => decide($("message-action"), false, "message-error") } : null;
    message(title, text, { action: back, link: !back });
    if (next.error === "step_up_unavailable") terminal("signin-message", "message-error");
  }
  function logout(next) {
    const l = next.logout || {}, account = next.account;
    let title, text = "", confirm = null, stay = null, other = false;
    if (l.ended) {
      title = "You're already signed out of that session."; confirm = "Continue";
      // Only that session ended: say so when this browser is still signed in to another one.
      if (account) text = `This browser is still signed in as ${account.display_name} (@${account.username}).`;
    }
    else if (l.matches_browser && account) {
      title = "Sign out of riAuth?"; confirm = "Sign out"; stay = "Stay signed in";
      text = `You're signed in as ${account.display_name}. This signs you out of all applications that use riAuth in this browser.`;
    } else if (l.targeted) {
      title = "This sign-out request belongs to another session"; text = "Approve it from that account's terminal.";
      stay = "Return to application"; other = true;
    } else { title = "You're already signed out in this browser."; confirm = "Continue"; }
    $("logout-title").textContent = title;
    $("logout-text").textContent = text; $("logout-text").hidden = !text;
    const mine = !!account && l.matches_browser === true && !l.ended;
    $("logout-account").textContent = mine ? `Signed in as ${account.display_name} (@${account.username})` : "";
    $("logout-account").hidden = !mine;
    $("logout-confirm").textContent = confirm ?? ""; $("logout-confirm").hidden = !confirm;
    $("logout-stay").textContent = stay ?? ""; $("logout-stay").hidden = !stay;
    terminal(other ? "signin-logout" : null);
    show("logout", "Sign out");
  }

  // User actions. Credential POSTs are never retried: a replay would spend a code.
  function describe(error, fallback) {
    if (error?.name === "NotAllowedError" || error?.name === "AbortError") return "Passkey sign-in was cancelled or timed out. Select the button to try again.";
    if (error?.status === 429) return "Too many attempts from your network. Try again in a minute.";
    if (error?.status === 503) return "riAuth is busy. Try again in a moment.";
    if (error?.status === 0) return "Couldn't reach riAuth. Check your connection and try again.";
    return error?.description || fallback;
  }
  function act(button, errorId, request, fallback) {
    return RiAuth.inFlight(button, async () => {
      acting += 1; generation += 1; stop(); clearError(errorId);
      let next = null, failure = null;
      try { next = await request(); } catch (error) { failure = error; }
      acting -= 1; generation += 1;
      if (next) { render(next); return; }
      if (failure.code === "invalid_credentials") {
        $("signin-otp").value = "";
        for (const field of FIELDS) $(field).setAttribute("aria-invalid", "true");
      }
      if (failure.code === "interaction_expired") { expired(); return; }
      if (["request_decided", "account_changed", "login_required"].includes(failure.code)) { load(); return; }
      showError(errorId, describe(failure, fallback), failure.code === "mfa_setup_required");
      // A missing session or binding may have changed what the page should show.
      if (failure.code === "invalid_token") load(); else schedule();
    });
  }
  function decide(button, approve, errorId) {
    const body = page.kind === "logout" ? { approve } : {
      approve, session_ref: state?.session_ref ?? null,
      remember: approve && current === "consent" && !$("consent-remember-label").hidden && $("consent-remember").checked
    };
    return act(button, errorId, () => RiAuth.post(`${page.api}/decision`, body, { retry: true }), "Couldn't send your decision. Try again.");
  }
  function code(value) {
    const trimmed = value.trim();
    return (trimmed.startsWith("ri_recovery_") ? trimmed : value.replace(/\s+/g, "")) || null;
  }
  $("signin-form").addEventListener("submit", (event) => {
    event.preventDefault();
    const username = $("signin-username").value.trim(), password = $("signin-password").value, otp = code($("signin-otp").value);
    if (!username || !password) { clearError("signin-error"); showError("signin-error", "Enter your username and password."); return; }
    if ($("signin-otp").required && !otp) { clearError("signin-error"); showError("signin-error", "Enter your authenticator or recovery code, or sign in with a passkey."); return; }
    $("signin-password").value = "";
    act($("signin-submit"), "signin-error", async () => {
      const next = await RiAuth.post(`${page.api}/password`, { username, password, otp });
      $("signin-otp").value = "";
      return next;
    }, "Couldn't sign in. Try again.");
  });
  for (const field of FIELDS) $(field).addEventListener("input", () => $(field).removeAttribute("aria-invalid"));
  $("signin-passkey").addEventListener("click", () => act($("signin-passkey"), "signin-error", () => flow(), "Couldn't sign in with your passkey. Try again or use your password."));
  // Another account: this browser signs out (a terminal session is only unmapped, F25).
  RiAuth.guard($("signin-switch"), () => act($("signin-switch"), "signin-error", async () => {
    await RiAuth.post("api/portal/sign-out", { scope: "browser" }).catch((error) => { if (error.status !== 401) throw error; });
    return RiAuth.get(`${page.api}/state`);
  }, "Couldn't switch accounts. Try again."));
  RiAuth.guard($("signin-cancel"), () => decide($("signin-cancel"), false, "signin-error"));
  RiAuth.guard($("consent-allow"), () => decide($("consent-allow"), true, "consent-error"));
  RiAuth.guard($("consent-deny"), () => decide($("consent-deny"), false, "consent-error"));
  RiAuth.guard($("logout-confirm"), () => decide($("logout-confirm"), true, "logout-error"));
  RiAuth.guard($("logout-stay"), () => decide($("logout-stay"), false, "logout-error"));
  RiAuth.guard($("message-action"), () => messageAction?.());
  $("signin-copy").addEventListener("click", async () => {
    try { await navigator.clipboard.writeText($("signin-command").textContent); announce("Command copied."); }
    catch {
      const selection = window.getSelection(), range = document.createRange();
      range.selectNodeContents($("signin-command")); selection.removeAllRanges(); selection.addRange(range);
      announce("Select and copy the command with your keyboard.");
    }
  });
  $("signin-terminal").addEventListener("toggle", () => { if ($("signin-terminal").open) load(); else schedule(); });
  document.addEventListener("visibilitychange", () => { if (document.visibilityState === "visible") load(); else stop(); });
  window.addEventListener("focus", () => load());
  window.addEventListener("pageshow", (event) => { if (event.persisted) { leaving = false; load(); } });
  setInterval(() => { if (waiting()) expiry(); }, 30000);

  $("signin-passkey").hidden = $("signin-divider").hidden = !RiAuth.passkeysAvailable();
  if (page) load();
  else message("This link isn't valid", "Return to the application and try again.", { link: true });
})();
