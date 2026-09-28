"use strict";
// A device code opens a review. Only the two decision buttons send a decision POST.
(() => {
  const $ = (id) => document.getElementById(id);
  const { base } = RiAuth;
  const screens = ["enter", "loading", "unavailable", "signin", "review", "finished"];
  const scopeHelp = {
    openid: "Confirm your riAuth identity",
    profile: "See your name and username",
    email: "See your email address",
    groups: "See your group memberships",
    offline_access: "Stay connected while you're away"
  };
  let code = null, review = null, screen = "loading", reauthenticate = false;
  let decisionKeys = {};
  let sequence = 0, authBusy = false, decisionBusy = false;

  function unavailable(message) {
    sequence += 1;
    code = null;
    review = null;
    $("device-unavailable-text").textContent = message;
    show("unavailable", "Device approval unavailable");
  }
  async function refreshCapabilities() {
    let state;
    try { state = await RiAuthCapabilities.refresh(); }
    catch { unavailable("Could not check device approval right now. Check your connection and try again."); return; }
    RiAuthCapabilities.apply();
    if (!RiAuthCapabilities.usable("oidc.device")) {
      unavailable("This server doesn't offer device approval. Contact your administrator if you need to connect a device.");
      return;
    }
    if (screen === "loading" || screen === "unavailable") {
      const url = new URL(location.href);
      const linked = url.searchParams.get("user_code") || url.searchParams.get("code");
      if (linked) lookup(linked);
      else enter();
    } else if (screen === "signin") {
      const passkeyAvailable = RiAuthCapabilities.usable("identity.passkeys") && RiAuth.passkeysAvailable();
      $("device-passkey").hidden = $("device-signin-divider").hidden = !passkeyAvailable;
    }
  }

  function announce(message) { $("device-announcement").textContent = message; }
  function show(name, title) {
    for (const item of screens) $(`device-${item}`).hidden = item !== name;
    document.title = `${title} · riAuth`;
    if (screen !== name) {
      screen = name;
      const heading = $(`device-${name}`).querySelector("h1");
      heading.focus();
      announce(heading.textContent);
    }
    RiAuth.arm();
  }
  function error(id, message) {
    const target = $(id);
    target.textContent = message;
    target.hidden = false;
    target.focus();
    announce(message);
  }
  function clearError(id) {
    $(id).textContent = "";
    $(id).hidden = true;
  }
  function describe(problem, fallback) {
    if (problem?.name === "NotAllowedError" || problem?.name === "AbortError") return "Passkey sign-in was cancelled or timed out. Try again.";
    if (problem?.status === 429) return "Too many attempts from your network. Try again in a minute.";
    if (problem?.status === 503) return "riAuth is busy. Try again in a moment.";
    if (problem?.status === 0) return "Couldn't reach riAuth. Check your connection and try again.";
    return problem?.description || fallback;
  }
  function normalize(input) {
    const compact = String(input || "").replace(/[ -]/g, "").toUpperCase();
    return /^[A-Z0-9]{10}$/.test(compact) ? `${compact.slice(0, 5)}-${compact.slice(5)}` : null;
  }
  function rememberCode(value) {
    const url = new URL(`${base}device`, location.origin);
    if (value) url.searchParams.set("user_code", value);
    history.replaceState(null, "", `${url.pathname}${url.search}`);
  }
  function enter(message = "") {
    sequence += 1;
    code = null;
    review = null;
    reauthenticate = false;
    rememberCode(null);
    clearError("device-enter-error");
    show("enter", "Approve a device");
    if (message) error("device-enter-error", message);
    else $("device-code").focus();
  }
  function finished(title, message) {
    sequence += 1;
    code = null;
    review = null;
    rememberCode(null);
    $("device-finished-title").textContent = title;
    $("device-finished-text").textContent = message;
    show("finished", title);
  }
  function list(id, values, label) {
    const items = Array.isArray(values) ? values.filter((value) => typeof value === "string") : [];
    $(id).replaceChildren(...items.map((value) => {
      const item = document.createElement("li");
      item.textContent = label(value);
      return item;
    }));
    return items.length;
  }
  function render(data) {
    const application = data?.application;
    const account = data?.account;
    if (typeof data?.user_code !== "string" || normalize(data.user_code) !== code
      || typeof application?.client_id !== "string" || !application.client_id
      || typeof application?.name !== "string" || !application.name
      || typeof account?.username !== "string" || typeof account?.display_name !== "string"
      || typeof data?.session_ref !== "string" || !data.session_ref
      || !Array.isArray(data?.scopes)) {
      enter("riAuth couldn't verify the application and account for this request. Try the code again.");
      return;
    }
    review = data;
    decisionKeys = {};
    code = normalize(data.user_code);
    $("device-code").value = code;
    $("device-review-code").textContent = code;
    $("device-account").textContent = `${account.display_name} (@${account.username})`;
    $("device-application").textContent = application.name;
    $("device-client-id").textContent = `Client ID: ${application.client_id}`;
    list("device-scopes", data.scopes, (scope) => scopeHelp[scope] ? `${scope} — ${scopeHelp[scope]}` : scope);
    $("device-claims-section").hidden = list("device-claims", data.claims, (claim) => claim) === 0;
    $("device-resource").textContent = data.resource ? `Resource: ${data.resource}` : "";
    $("device-resource").hidden = !data.resource;
    const expires = Number(data.expires_at);
    $("device-expiry").textContent = Number.isFinite(expires) && expires > 0
      ? `This request expires at ${new Date(expires * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}.`
      : "";
    $("device-expiry").hidden = !$("device-expiry").textContent;

    const needsMfa = data.require_mfa === true && account.mfa !== true;
    const needsSignIn = data.reauthentication_required === true || data.approval_allowed !== true;
    const explanation = needsMfa
      ? "This application requires extra verification. Sign in again with a passkey or an authenticator code before approving."
      : data.reauthentication_required === true
        ? "Your sign-in is too old for this approval. Sign in again to continue."
        : data.approval_allowed !== true
          ? "This account cannot approve yet. Sign in again or use another account."
          : "";
    $("device-requirement-text").textContent = explanation;
    $("device-requirement").hidden = !explanation;
    $("device-reauth").hidden = !needsSignIn;
    $("device-approve").disabled = data.approval_allowed !== true;
    $("device-deny").disabled = false;
    clearError("device-review-error");
    show("review", "Review device access");
  }
  async function lookup(nextCode) {
    if (!RiAuthCapabilities.usable("oidc.device")) return;
    const requested = normalize(nextCode);
    if (!requested) {
      enter("Enter the ten-character code shown on your device.");
      $("device-code").value = String(nextCode || "");
      return;
    }
    const at = ++sequence;
    code = requested;
    review = null;
    $("device-code").value = requested;
    rememberCode(requested);
    show("loading", "Checking your code");
    try {
      const data = await RiAuth.get(`api/device/browser/${encodeURIComponent(requested)}`);
      if (at !== sequence) return;
      render(data);
    } catch (problem) {
      if (at !== sequence) return;
      if (problem.status === 401) {
        signin(false);
      } else if (problem.status === 409) {
        finished("Request already decided", "This device request has already been approved or rejected. Start a new request on your device if needed.");
      } else if ([400, 404, 410].includes(problem.status)) {
        enter("This device code is invalid or has expired. Check the code on your device and try again.");
      } else {
        enter(describe(problem, "Couldn't check the device code. Try again."));
      }
    }
  }
  function signin(again) {
    if (!code) { enter(); return; }
    reauthenticate = again;
    $("device-signin-code").textContent = code;
    $("device-signin-title").textContent = again ? "Confirm it's you" : "Sign in to review this request";
    const passkeyAvailable = RiAuthCapabilities.usable("identity.passkeys") && RiAuth.passkeysAvailable();
    $("device-signin-reason").textContent = again
      ? `Sign in again with the account shown on the review. Use ${passkeyAvailable ? "a passkey or " : ""}your password and authenticator code.`
      : "Your code is ready. Sign in to see which application is asking for access.";
    $("device-username").value = again && review ? review.account.username : "";
    $("device-username").readOnly = again && !!review;
    $("device-password").value = "";
    $("device-otp").value = "";
    $("device-passkey").hidden = $("device-signin-divider").hidden = !passkeyAvailable;
    $("device-signin-back").textContent = again && review ? "Back to review" : "Use another device code";
    clearError("device-signin-error");
    show("signin", again ? "Confirm it's you" : "Sign in to review");
  }
  async function signedIn() {
    $("device-password").value = "";
    $("device-otp").value = "";
    await lookup(code);
  }
  const passkeyFlows = [false, true].map((again) => RiAuth.passkeyFlow(
    () => RiAuth.post("api/portal/login/passkey/start", { reauthenticate: again }, { retry: true }),
    (credential, started) => RiAuth.post("api/portal/login/passkey/finish", { ceremony: started.ceremony, credential })
  ));

  $("device-code-form").addEventListener("submit", (event) => {
    event.preventDefault();
    clearError("device-enter-error");
    lookup($("device-code").value);
  });
  $("device-signin-form").addEventListener("submit", (event) => {
    event.preventDefault();
    if (authBusy) return;
    RiAuth.inFlight($("device-signin-submit"), async () => {
      authBusy = true;
      clearError("device-signin-error");
      const username = $("device-username").value.trim();
      const password = $("device-password").value;
      const otp = $("device-otp").value.trim() || null;
      $("device-password").value = "";
      if (!username || !password) {
        error("device-signin-error", "Enter your username and password.");
        authBusy = false;
        return;
      }
      try {
        await RiAuth.post("api/portal/login/password", { username, password, otp, reauthenticate });
        await signedIn();
      } catch (problem) {
        $("device-otp").value = "";
        error("device-signin-error", describe(problem, "Couldn't sign in. Try again."));
      } finally { authBusy = false; }
    });
  });
  $("device-passkey").addEventListener("click", () => {
    if (authBusy || !RiAuthCapabilities.usable("identity.passkeys") || !RiAuth.passkeysAvailable()) return;
    RiAuth.inFlight($("device-passkey"), async () => {
      authBusy = true;
      clearError("device-signin-error");
      try {
        await passkeyFlows[reauthenticate ? 1 : 0]();
        await signedIn();
      } catch (problem) {
        error("device-signin-error", describe(problem, "Couldn't sign in with your passkey. Try again or use your password."));
      } finally { authBusy = false; }
    });
  });
  $("device-signin-back").addEventListener("click", () => {
    if (authBusy) return;
    if (reauthenticate && review) render(review);
    else enter();
  });
  $("device-switch-account").addEventListener("click", () => signin(false));
  $("device-reauth").addEventListener("click", () => signin(true));
  $("device-change-code").addEventListener("click", () => enter());
  $("device-start-again").addEventListener("click", () => enter());
  $("device-retry").addEventListener("click", () => { show("loading", "Checking device approval"); void refreshCapabilities(); });

  async function decide(approve) {
    if (!RiAuthCapabilities.usable("oidc.device") || !review || !code || decisionBusy || approve && review.approval_allowed !== true) return;
    const button = $(approve ? "device-approve" : "device-deny");
    RiAuth.inFlight(button, async () => {
      decisionBusy = true;
      $("device-approve").disabled = $("device-deny").disabled = true;
      clearError("device-review-error");
      const requested = code;
      const sessionRef = review.session_ref;
      const choice = approve ? "approve" : "deny";
      decisionKeys[choice] ||= crypto.randomUUID();
      try {
        await RiAuth.post("api/device/browser/decision", { user_code: requested, approve, session_ref: sessionRef },
          { key: decisionKeys[choice], retry: true });
        finished(approve ? "Device approved" : "Request rejected", approve
          ? "You can return to your device. It will finish signing in shortly."
          : "The device was not allowed to sign in. You can close this page.");
      } catch (problem) {
        if (problem.status === 401) {
          signin(false);
        } else if ([400, 404, 410].includes(problem.status)) {
          finished("Request expired", "This device request has expired. Start a new request on your device.");
        } else if (problem.status === 409 && problem.code !== "account_changed") {
          finished("Request already decided", "This device request has already been approved or rejected. Start a new request on your device if needed.");
        } else if (problem.code === "account_changed") {
          await lookup(requested);
          if (screen === "review") error("device-review-error", "Your signed-in account changed. Review the application and access again before deciding.");
        } else {
          error("device-review-error", describe(problem, "Couldn't record your decision. Review the request and try again."));
        }
      } finally {
        decisionBusy = false;
        if (screen === "review" && review) {
          $("device-approve").disabled = review.approval_allowed !== true;
          $("device-deny").disabled = false;
        }
      }
    });
  }
  RiAuth.guard($("device-approve"), () => decide(true));
  RiAuth.guard($("device-deny"), () => decide(false));

  window.addEventListener("pageshow", (event) => { if (event.persisted) void refreshCapabilities(); });
  void refreshCapabilities();
})();
