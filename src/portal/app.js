"use strict";
(() => {
  const $ = (id) => document.getElementById(id);
  const { base } = RiAuth;
  const state = { data: null, favorites: new Set(), view: "grid", section: "all", loading: false, generation: 0, request: null,
    verificationMessage: null, verificationRequested: false, verificationNeedsRelogin: false };
  // Passkey flows keep cancelled options for WebKit's gesture rule; `retry` runs after re-authentication.
  const security = { flows: {}, retry: null, data: null, action: null, generation: 0, busy: false, passwordOpen: false };
  // Names what a verification unlocks: the password too while it is local and blocked.
  function blocked() {
    const data = security.data;
    return data?.password === "local" && data.can_change_password === false ? "your password or passkeys" : "your passkeys";
  }
  function hint(kind, what = blocked()) {
    if (kind === "terminal") return `This browser uses your terminal's sign-in. Sign in here to change ${what}.`;
    if (kind === "fresh") return `Confirm it's you to change ${what}.`;
    return `Sign in with your passkey or authenticator code to change ${what}.`;
  }
  // Authenticator app state. A setup key or recovery code lives only in the DOM of its step.
  const factor = { data: null, enrollment: null, action: null, codes: null };
  let pollTimer, expiryTimer, toastTimer;
  const icons = new Set(["app", "code", "chart", "files", "messages", "book", "cloud", "terminal", "shield", "globe"]);
  const accents = new Set(["violet", "blue", "teal", "amber", "rose", "slate"]);

  function icon(name) {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("class", "icon"); svg.setAttribute("aria-hidden", "true");
    const use = document.createElementNS(svg.namespaceURI, "use");
    use.setAttribute("href", `#i-${name}`); svg.append(use); return svg;
  }
  function element(tag, className, text) {
    const node = document.createElement(tag); if (className) node.className = className;
    if (text !== undefined) node.textContent = text; return node;
  }
  function toast(message) {
    clearTimeout(toastTimer); $("toast").textContent = message; $("toast").hidden = false;
    toastTimer = setTimeout(() => { $("toast").hidden = true; }, 5000);
  }
  function connection(label, live = false) {
    $("connection-label").textContent = label; $("connection").classList.toggle("live", live);
  }
  function screen(name) {
    if (name === "catalogue" && $("catalogue").hidden) RiAuth.arm();
    const active = document.activeElement;
    const enteringAuth = name === "auth" && $("auth").hidden;
    const workspace = name === "catalogue";
    document.body.classList.toggle("portal-entry", !workspace);
    $("workspace-navigation").hidden = $("workspace-header").hidden = !workspace;
    document.title = workspace ? "Your applications · riAuth" : name === "error" ? "Applications unavailable · riAuth" : "Sign in · riAuth";
    for (const id of ["catalogue", "auth", "error", "loading"]) $(id).hidden = id !== name;
    if (enteringAuth || active?.closest("[hidden]")) $(name === "auth" ? "auth-title" : "main").focus({ preventScroll: true });
  }
  function api(path, method = "GET") {
    return method === "POST" ? RiAuth.post(`api/portal${path}`) : RiAuth.get(`api/portal${path}`);
  }
  function preferenceKey() { return `riauth:portal:${base}:${state.data.user.id}`; }
  function loadPreferences() {
    state.favorites = new Set(); state.view = "grid";
    try {
      const value = JSON.parse(localStorage.getItem(preferenceKey()) || "{}");
      if (Array.isArray(value.favorites)) state.favorites = new Set(value.favorites.filter((id) => typeof id === "string").slice(0, 1000));
      if (value.view === "list") state.view = "list";
    } catch { /* Storage is optional, including in private browsing. */ }
  }
  function savePreferences() {
    try { localStorage.setItem(preferenceKey(), JSON.stringify({ favorites: [...state.favorites], view: state.view })); }
    catch { /* Keep this tab usable when storage is unavailable. */ }
  }
  function clearIdentity() {
    // Signed-out refreshes (for example on focus after a cancelled passkey prompt) keep the flows.
    if (state.data) resetFlows();
    state.data = null; state.favorites.clear(); state.section = "all";
    state.verificationMessage = null; state.verificationRequested = false; state.verificationNeedsRelogin = false;
    $("apps").replaceChildren(); $("account-name").textContent = ""; $("account-username").textContent = "";
    $("avatar").textContent = ""; $("avatar").removeAttribute("title"); $("account").removeAttribute("aria-label");
    $("welcome").textContent = "Everything you need, one sign-in away."; $("access-count").textContent = "";
    $("account").hidden = true; $("signed-out-label").hidden = false;
    $("admin-link").hidden = true;
    $("access-review-link").hidden = true;
    $("nav-all").disabled = true; $("nav-favorites").disabled = true;
    $("all-count").textContent = "—"; $("favorite-count").textContent = "—";
    $("search").value = ""; $("category").replaceChildren(new Option("All categories", ""));
    $("nav-all").classList.add("active"); $("nav-all").setAttribute("aria-current", "page");
    $("nav-favorites").classList.remove("active"); $("nav-favorites").removeAttribute("aria-current");
    $("mfa-notice").hidden = true; $("verification-notice").hidden = true; $("passkey-list").replaceChildren();
    if ($("security-dialog").open) $("security-dialog").close();
    if ($("settings-dialog").open) $("settings-dialog").close();
    $("settings-account").textContent = "";
    clearTimeout(expiryTimer);
  }
  function resetFlows() {
    for (const flow of Object.values(security.flows)) if (typeof flow === "function") void flow.cancel();
    const finish = (credential, started) => RiAuth.post("api/portal/login/passkey/finish", { ceremony: started.ceremony, credential });
    const start = (reauthenticate) => () => RiAuth.post("api/portal/login/passkey/start", { reauthenticate }, { retry: true });
    const cancel = (started) => RiAuth.post("api/portal/login/passkey/cancel", { ceremony: started.ceremony });
    security.flows = { signIn: RiAuth.passkeyFlow(start(false), finish, false, cancel), reauth: RiAuth.passkeyFlow(start(true), finish, false, cancel), add: null, name: null };
    security.retry = null; security.data = null; security.action = null; security.generation += 1;
    security.busy = false; security.committing = false; security.run = null;
    $("passkey-action").hidden = true; $("passkey-rename").value = "";
    $("passkey-login-cancel").hidden = true;
    closePassword();
    factor.data = null; factor.enrollment = null; factor.action = null; clearEnrollment(); renderFactor();
  }
  function stopRequest() {
    state.request = null; clearTimeout(pollTimer);
    $("login-request").hidden = true; $("start-login").hidden = false; $("start-login").disabled = false;
    $("login-command").textContent = ""; $("user-code").textContent = "";
  }

  function renderVerificationNotice() {
    const user = state.data?.user;
    $("verification-notice").hidden = !user || user.email_verified !== false;
    if (!user || user.email_verified !== false) return;
    $("verification-notice-text").textContent = state.verificationMessage || (user.has_email
      ? "Your email address is not verified. Send a one-use link to the address on your account."
      : "Your account has no email address. Ask your administrator to add one before verifying it.");
    $("verification-action").hidden = !user.has_email || state.verificationNeedsRelogin;
    $("verification-action").textContent = state.verificationRequested ? "Resend verification email" : "Send verification email";
    const showRelogin = user.has_email && state.verificationNeedsRelogin;
    if (showRelogin && $("verification-relogin").hidden) RiAuth.arm();
    $("verification-relogin").hidden = !showRelogin;
  }

  async function refresh() {
    if (state.loading) { state.refreshAgain = true; return; }
    state.loading = true; const generation = state.generation;
    $("refresh").disabled = true;
    try {
      await RiAuthCapabilities.refresh();
      const passkeyLogin = RiAuthCapabilities.usable("identity.passkeys") && RiAuth.passkeysAvailable();
      $("passkey-login").hidden = $("auth-divider").hidden = !passkeyLogin;
      const data = await api("");
      if (generation !== state.generation) return;
      const changedUser = state.data?.user.id !== data.user.id;
      if (changedUser && $("security-dialog").open) $("security-dialog").close();
      if (changedUser && $("settings-dialog").open) $("settings-dialog").close();
      state.data = data;
      if (changedUser) {
        loadPreferences(); state.section = "all"; $("search").value = ""; resetFlows();
        state.verificationMessage = null; state.verificationRequested = false; state.verificationNeedsRelogin = false;
      }
      const accessible = new Set(data.apps.map((app) => app.id));
      state.favorites = new Set([...state.favorites].filter((id) => accessible.has(id)));
      savePreferences(); stopRequest();
      $("account").hidden = false; $("signed-out-label").hidden = true;
      $("admin-link").hidden = data.user.admin !== true;
      $("access-review-link").hidden = data.access_review_available !== true || !RiAuthCapabilities.compiled("access.temporary_entitlements");
      $("nav-all").disabled = false; $("nav-favorites").disabled = false;
      $("account-name").textContent = data.user.display_name;
      $("account-username").textContent = `@${data.user.username}`;
      $("avatar").title = `${data.user.display_name} (@${data.user.username})`;
      $("account").setAttribute("aria-label", `Signed in as ${data.user.display_name} (@${data.user.username})`);
      $("avatar").textContent = (data.user.display_name.trim().split(/\s+/).map((part) => Array.from(part)[0]).filter(Boolean).slice(0, 2).join("") || "U").toLocaleUpperCase();
      $("welcome").textContent = `Welcome back, ${data.user.display_name}. Find your next starting point.`;
      $("access-count").textContent = `${data.apps.length} ${data.apps.length === 1 ? "app" : "apps"} available`;
      renderVerificationNotice();
      // A password-only session cannot see applications that require MFA.
      $("mfa-notice").hidden = data.mfa !== false;
      $("mfa-notice-text").textContent = data.mfa_available ? "Some applications need your passkey or authenticator code." : "Some applications need extra verification. Add a passkey or an authenticator app under Sign-in and security.";
      $("mfa-action").textContent = data.mfa_available && passkeyLogin ? "Sign in with your passkey" : "Sign-in and security";
      clearAuthError();
      const category = changedUser ? "" : $("category").value;
      $("category").replaceChildren(new Option("All categories", ""));
      [...new Set(data.apps.map((app) => app.category))].sort((a, b) => a.localeCompare(b)).forEach((name) => $("category").add(new Option(name, name)));
      $("category").value = [...$("category").options].some((option) => option.value === category) ? category : "";
      screen("catalogue"); connection("Connected", true); render(); RiAuthCapabilities.apply();
      clearTimeout(expiryTimer);
      expiryTimer = setTimeout(refresh, Math.max(1000, Math.min(30000, data.expires_at * 1000 - Date.now())));
    } catch (error) {
      if (generation !== state.generation) return;
      const hadSession = !!state.data; clearIdentity();
      if (error.status === 401) {
        screen("auth"); connection("Not signed in");
        $("auth-description").textContent = hadSession ? "Your session has ended. Sign in again to return to your applications." : "Sign in to open your applications.";
      } else {
        screen("error"); connection("Connection interrupted");
        $("announcement").textContent = "We couldn’t check your access. Retry to load your applications.";
      }
    } finally {
      state.loading = false; $("refresh").disabled = false;
      if (state.refreshAgain) { state.refreshAgain = false; setTimeout(refresh, 0); }
    }
  }

  function render() {
    if (!state.data) return;
    const query = $("search").value.trim().toLocaleLowerCase(), category = $("category").value;
    const all = state.data.apps;
    const visible = all.filter((app) => (state.section !== "favorites" || state.favorites.has(app.id)) && (!category || app.category === category) && (!query || `${app.name} ${app.description} ${app.category} ${app.host || ""}`.toLocaleLowerCase().includes(query)));
    const focusedFavorite = document.activeElement?.dataset.favorite;
    const focusedLaunch = document.activeElement?.dataset.launch;
    $("all-count").textContent = String(all.length); $("favorite-count").textContent = String(state.favorites.size);
    $("clear-search").hidden = !$("search").value; $("search-shortcut").hidden = !!$("search").value;
    for (const section of ["all", "favorites"]) {
      const selected = state.section === section, button = $(`nav-${section}`);
      button.classList.toggle("active", selected); if (selected) button.setAttribute("aria-current", "page"); else button.removeAttribute("aria-current");
    }
    const favorites = state.section === "favorites";
    $("page-title").replaceChildren(document.createTextNode(favorites ? "Your favorites" : "Your applications"), element("span", "heading-dot", "."));
    $("collection-title").replaceChildren(document.createTextNode(favorites ? "Favorite applications " : "All applications "), element("span", "", `(${visible.length})`));
    $("breadcrumb-current").textContent = favorites ? "Favorites" : "Applications";
    document.title = `${favorites ? "Your favorites" : "Your applications"} · riAuth`;
    $("apps").classList.toggle("list", state.view === "list");
    for (const view of ["grid", "list"]) { $(`view-${view}`).classList.toggle("selected", state.view === view); $(`view-${view}`).setAttribute("aria-pressed", String(state.view === view)); }
    const fragment = document.createDocumentFragment();
    for (const app of visible) {
      const card = element("article", "app-card");
      const top = element("div", "app-card-top");
      const mark = element("span", `app-icon accent-${accents.has(app.accent) ? app.accent : "violet"}`);
      mark.append(icon(icons.has(app.icon) ? app.icon : "app"));
      const favorite = element("button", "icon-button favorite-button");
      const selected = state.favorites.has(app.id);
      favorite.dataset.favorite = app.id; favorite.setAttribute("aria-label", `${selected ? "Remove" : "Add"} ${app.name} ${selected ? "from" : "to"} favorites`);
      favorite.title = selected ? "Remove from favorites" : "Add to favorites";
      favorite.setAttribute("aria-pressed", String(selected)); favorite.append(icon("star"));
      favorite.addEventListener("click", () => {
        if (state.favorites.has(app.id)) state.favorites.delete(app.id); else state.favorites.add(app.id);
        savePreferences(); render();
        if (favorites && selected) $("nav-favorites").focus();
        $("announcement").textContent = `${app.name} ${selected ? "removed from" : "added to"} favorites.`;
      });
      top.append(mark, favorite);
      const content = element("div", "app-card-content");
      const heading = element("h3");
      // Only local, policy-checked launch routes are actionable. Metadata never becomes HTML.
      const launchPath = `${base}apps/launch?client_id=${encodeURIComponent(app.id)}`;
      const launchable = typeof app.launch_path === "string" && app.launch_path === launchPath;
      if (launchable) {
        const link = element("a", "app-link", app.name); link.href = launchPath;
        link.dataset.launch = app.id;
        link.target = "_blank"; link.rel = "noopener noreferrer";
        link.setAttribute("aria-label", `Open ${app.name} (opens in a new tab)`); heading.append(link);
      } else heading.textContent = app.name;
      content.append(heading, element("p", "app-description", app.description || app.host || "Your connected application."));
      const bottom = element("div", "app-card-bottom");
      bottom.append(element("span", "category-tag", app.category));
      if (launchable) {
        const action = element("span", "open-indicator"); action.setAttribute("aria-hidden", "true");
        action.append(element("span", "", "Open app"), icon("arrow")); bottom.append(action);
      } else {
        const unavailable = element("span", "unavailable", "Setup pending");
        unavailable.title = "Your administrator needs to configure this application’s launch URL."; bottom.append(unavailable);
      }
      card.append(top, content, bottom); fragment.append(card);
    }
    $("apps").replaceChildren(fragment);
    if (focusedFavorite) [...$("apps").querySelectorAll("[data-favorite]")].find((button) => button.dataset.favorite === focusedFavorite)?.focus({ preventScroll: true });
    if (focusedLaunch) [...$("apps").querySelectorAll("[data-launch]")].find((link) => link.dataset.launch === focusedLaunch)?.focus({ preventScroll: true });
    $("empty").hidden = visible.length !== 0;
    const filtered = !!query || !!category;
    $("reset-filters").hidden = !filtered;
    $("empty-title").textContent = filtered ? "No matching applications" : favorites ? "Keep your go-to apps close" : "Your workspace is ready to grow";
    $("empty-description").textContent = filtered ? "Try another name or category, or clear your filters to see more applications." : favorites ? "Select the star on an application to find it here next time." : "No applications are available for this account yet. Contact your administrator if you’re expecting access.";
    $("empty-svg").replaceChildren(icon(favorites && !filtered ? "star" : filtered ? "search" : "app").firstChild);
    $("announcement").textContent = `${visible.length} ${visible.length === 1 ? "application" : "applications"} shown.`;
  }

  function setSection(section) { if (!state.data) return; state.section = section; $("search").value = ""; $("category").value = ""; render(); }
  $("nav-all").addEventListener("click", () => setSection("all"));
  $("nav-favorites").addEventListener("click", () => setSection("favorites"));
  $("search").addEventListener("input", render); $("category").addEventListener("change", render);
  $("clear-search").addEventListener("click", () => { $("search").value = ""; render(); $("search").focus(); });
  $("reset-filters").addEventListener("click", () => { $("search").value = ""; $("category").value = ""; render(); $("search").focus(); });
  for (const view of ["grid", "list"]) $(`view-${view}`).addEventListener("click", () => { state.view = view; savePreferences(); render(); });
  document.addEventListener("keydown", (event) => {
    if (!state.data || event.isComposing || $("security-dialog").open || $("settings-dialog").open) return;
    const typing = /INPUT|TEXTAREA|SELECT/.test(document.activeElement?.tagName) || document.activeElement?.isContentEditable;
    if ((!typing && event.key === "/") || ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k")) { event.preventDefault(); $("search").focus(); }
    if (event.key === "Escape" && document.activeElement === $("search")) { $("search").value = ""; render(); }
  });
  $("refresh").addEventListener("click", refresh); $("retry").addEventListener("click", refresh);
  window.addEventListener("focus", refresh);
  document.addEventListener("visibilitychange", () => { if (!document.hidden) refresh(); });
  window.addEventListener("pageshow", (event) => { if (event.persisted) { clearIdentity(); screen("loading"); refresh(); } });

  $("start-login").addEventListener("click", async () => {
    if (!(await security.flows.signIn.cancel())) return;
    $("passkey-login-cancel").hidden = true;
    $("start-login").disabled = true;
    try {
      const request = await api("/sign-in", "POST");
      state.request = request;
      $("login-request").hidden = false; $("start-login").hidden = true;
      $("user-code").textContent = request.code;
      $("login-command").textContent = `riauth --server ${RiAuth.shellQuote(request.issuer)} portal approve ${request.code}`;
      $("copy-command").focus();
      poll();
    } catch (error) { toast(error.status === 429 ? "Too many sign-in attempts. Try again in a minute." : "Couldn’t start sign-in. Please try again."); }
    finally { $("start-login").disabled = false; }
  });
  async function poll() {
    const request = state.request; if (!request) return;
    const seconds = Math.ceil((request.expires_at * 1000 - Date.now()) / 1000);
    if (seconds <= 0) { stopRequest(); toast("This sign-in code has expired. Start again for a new code."); return; }
    $("code-time").textContent = `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")} remaining`;
    try {
      const result = await api(`/sign-in/${encodeURIComponent(request.id)}`, "POST");
      if (state.request !== request) return;
      if (result.status === "approved") { stopRequest(); await refresh(); return; }
      if (result.status === "denied") { stopRequest(); toast("Sign-in was declined in your terminal."); return; }
    } catch (error) {
      if (state.request !== request) return;
      if (error.status === 401 || error.status === 404) { stopRequest(); await refresh(); toast("The sign-in request has ended."); return; }
      // Preserve a valid request across short network interruptions, within its deadline.
      connection("Reconnecting");
    }
    if (state.request === request) pollTimer = setTimeout(poll, 2000);
  }
  $("cancel-login").addEventListener("click", async () => {
    const request = state.request; stopRequest();
    if (request) { try { await api(`/sign-in/${encodeURIComponent(request.id)}/cancel`, "POST"); } catch { /* Expired or already consumed. Refresh resolves the actual session state. */ } }
    await refresh();
  });
  $("copy-command").addEventListener("click", async () => {
    try { await navigator.clipboard.writeText($("login-command").textContent); toast("Sign-in command copied."); }
    catch { const selection = window.getSelection(), range = document.createRange(); range.selectNodeContents($("login-command")); selection.removeAllRanges(); selection.addRange(range); toast("Select and copy the command with your keyboard."); }
  });
  RiAuth.guard($("verification-action"), () => {
    const user = state.data?.user;
    if (!user || user.email_verified || !user.has_email) return;
    RiAuth.inFlight($("verification-action"), async () => {
      try {
        const result = await RiAuth.post("api/portal/account/verify-request", {});
        if (state.data?.user.id !== user.id) return;
        state.verificationNeedsRelogin = false;
        if (result.status === "already_verified") {
          await refresh();
          return;
        }
        state.verificationRequested = true;
        state.verificationMessage = result.status === "queued"
          ? "Verification email queued. Check your inbox for a one-use link."
          : "A verification email was requested recently. Check your inbox; try again later.";
      } catch (error) {
        if (state.data?.user.id !== user.id) return;
        if (error.status === 401) { await refresh(); return; }
        if (error.code === "reauthentication_required" || error.status === 403) {
          state.verificationNeedsRelogin = true;
          state.verificationMessage = "Sign in again to request a verification link. This confirms it is still you.";
        } else if (error.code === "delivery_unavailable") {
          state.verificationMessage = "Email delivery is unavailable. Contact your administrator and try again later.";
        } else if (error.status === 429) {
          state.verificationMessage = "Too many requests from your network. Try again in a minute.";
        } else {
          state.verificationMessage = "Could not request a verification email. Check your connection and try again.";
        }
      }
      renderVerificationNotice();
    });
  });
  RiAuth.guard($("verification-relogin"), () => RiAuth.inFlight($("verification-relogin"), async () => {
    const username = state.data?.user.username;
    if (!username) return;
    try {
      await RiAuth.post("api/portal/sign-out", { scope: "browser" });
      state.generation += 1; clearIdentity(); stopRequest(); screen("auth"); connection("Not signed in");
      $("auth-description").textContent = "Sign in again, then request a new verification link.";
      $("login-username").value = username;
      ($("passkey-login").hidden ? $("login-password") : $("passkey-login")).focus();
    } catch {
      state.verificationMessage = "Could not start a fresh sign-in. Try again or use Sign out, then sign in.";
      renderVerificationNotice();
    }
  }));
  RiAuth.guard($("sign-out"), async () => {
    $("sign-out").disabled = true; state.generation += 1;
    try {
      const result = await api("/sign-out", "POST"); clearIdentity(); stopRequest(); screen("auth"); connection("Not signed in");
      toast("You’re signed out.");
      if (typeof result.saml_logout_url === "string") {
        const target = new URL(result.saml_logout_url, location.origin);
        if (target.origin === location.origin && target.pathname.startsWith(`${base}saml/logout/`)) location.assign(target.href);
      }
    } catch { toast("Couldn’t sign out. Check your connection and try again."); }
    finally { $("sign-out").disabled = false; }
  });

  // Browser sign-in. Credential POSTs are never retried: a replay would spend a code.
  function describe(error, fallback) {
    if (error?.name === "NotAllowedError" || error?.name === "AbortError") return "Passkey sign-in was cancelled or timed out. Select the button to try again.";
    if (error?.status === 429) return "Too many attempts from your network. Try again in a minute.";
    if (error?.status === 503) return "riAuth is busy. Try again in a moment.";
    if (error?.status === 0) return "Couldn't reach riAuth. Check your connection and try again.";
    if (["invalid_credentials", "unknown_passkey"].includes(error?.code) && error.description) return error.description;
    return fallback;
  }
  function code(value) {
    const trimmed = value.trim();
    return (trimmed.startsWith("ri_recovery_") ? trimmed : value.replace(/\s+/g, "")) || null;
  }
  function showError(id, message) { $(id).textContent = message; $(id).hidden = false; $(id).focus(); }
  function clearAuthError() {
    $("auth-error").hidden = true; $("auth-error").textContent = "";
    for (const id of ["login-username", "login-password", "login-otp"]) $(id).removeAttribute("aria-invalid");
    for (const id of ["login-username", "login-password"]) $(id).removeAttribute("aria-describedby");
  }
  async function signedIn() { $("login-password").value = ""; $("login-otp").value = ""; stopRequest(); await refresh(); }
  $("password-form").addEventListener("submit", (event) => {
    event.preventDefault();
    RiAuth.inFlight($("password-login"), async () => {
      const username = $("login-username").value.trim(), password = $("login-password").value;
      $("login-password").value = "";
      clearAuthError();
      if (!username || !password) {
        if (!username) { $("login-username").setAttribute("aria-invalid", "true"); $("login-username").setAttribute("aria-describedby", "auth-error"); }
        if (!password) { $("login-password").setAttribute("aria-invalid", "true"); $("login-password").setAttribute("aria-describedby", "auth-error"); }
        showError("auth-error", "Enter your username and password.");
        return;
      }
      try {
        if (!(await security.flows.signIn.cancel())) { showError("auth-error", "Finishing passkey sign-in. Please wait."); return; }
        $("passkey-login-cancel").hidden = true;
        await RiAuth.post("api/portal/login/password", { username, password, otp: code($("login-otp").value), reauthenticate: false });
        await signedIn();
      } catch (error) {
        $("login-password").value = "";
        if (error.code === "invalid_credentials") {
          $("login-otp").value = "";
          for (const id of ["login-username", "login-password", "login-otp"]) $(id).setAttribute("aria-invalid", "true");
        }
        showError("auth-error", describe(error, error.description || "Couldn't sign in. Try again."));
      }
    });
  });
  for (const id of ["login-username", "login-password", "login-otp"]) $(id).addEventListener("input", () => {
    $(id).removeAttribute("aria-invalid");
    if (id !== "login-otp") $(id).removeAttribute("aria-describedby");
  });
  $("passkey-login").addEventListener("click", () => RiAuth.inFlight($("passkey-login"), async () => {
    const generation = state.generation;
    clearAuthError();
    $("passkey-login-cancel").hidden = false;
    try { await security.flows.signIn(); if (generation === state.generation) await signedIn(); else await refresh(); }
    catch (error) { if (generation === state.generation) showError("auth-error", describe(error, "Couldn't sign in with your passkey. Try again or use your password.")); }
  }));
  $("passkey-login-cancel").addEventListener("click", async () => {
    if (security.flows.signIn.finishing) { showError("auth-error", "Finishing sign-in. Please wait."); return; }
    state.generation += 1;
    await security.flows.signIn.cancel();
    $("passkey-login-cancel").hidden = true;
    clearAuthError(); $("passkey-login").focus();
  });

  // Passkey management stays bound to the account and dialog that opened the action.
  function securityStatus(message) { $("security-status").textContent = message; }
  function showReauth(hint) {
    $("reauth-hint").textContent = hint; $("reauth-panel").hidden = false; $("reauth-error").hidden = true;
    $("reauth-passkey").hidden = !RiAuth.passkeysAvailable() || security.data?.passkeys.length === 0;
    // Directory accounts confirm with their directory password; only passwordless ones cannot.
    $("reauth-form").hidden = security.data?.password === "none";
  }
  function hideReauth() {
    $("reauth-panel").hidden = true; $("reauth-password").value = ""; $("reauth-otp").value = "";
  }
  function securityControls() {
    const data = security.data, pending = !!security.action;
    $("passkey-form").hidden = !RiAuthCapabilities.usable("identity.passkeys") || !data || pending || !RiAuth.passkeysAvailable() || data.passkeys.length >= data.limit;
    for (const button of $("passkey-list").querySelectorAll("button")) button.disabled = security.busy || button.dataset.unavailable === "true";
    for (const id of ["add-passkey", "passkey-action-confirm", "reauth-passkey", "reauth-confirm", "password-change-start", "password-change-submit", "password-change-cancel", "password-current", "password-new", "password-confirm"]) $(id).disabled = security.busy;
    $("passkey-name").disabled = $("passkey-rename").disabled = security.busy;
    // Password and authenticator verifications wait in the shared panel; Cancel change belongs to passkeys.
    $("passkey-cancel").hidden = !pending && !security.busy && !security.flows.add && (!security.retry || security.retry === openPassword || security.retry.factor === true);
    renderPassword(); renderFactor();
  }
  function verificationHint(what) {
    return hint(security.data?.terminal ? "terminal" : security.data?.fresh === false ? "fresh" : "mfa", what);
  }
  function canChange(kind) {
    const data = security.data;
    return data && data.fresh && !data.terminal && data[kind === "add" ? "can_register" : kind === "rename" ? "can_rename" : "can_remove"];
  }
  async function loadPasskeys() {
    const generation = security.generation, user = state.data?.user.id;
    securityStatus("Loading your passkeys…");
    try {
      const data = await RiAuth.get("api/portal/passkeys");
      if (generation !== security.generation || state.data?.user.id !== user || !$("security-dialog").open) return;
      if (data.user_id !== user) { $("security-dialog").close(); await refresh(); return; }
      security.data = data;
      const rows = data.passkeys.map((passkey) => {
        const row = element("li", "passkey-row"), text = element("div", "passkey-text"), actions = element("div", "passkey-actions");
        text.append(element("strong", "", passkey.name), element("span", "", `Added ${new Date(passkey.created_at * 1000).toLocaleDateString()}`));
        const rename = element("button", "button secondary", "Rename");
        rename.type = "button"; rename.setAttribute("aria-label", `Rename ${passkey.name}`);
        rename.addEventListener("click", () => selectPasskey("rename", passkey));
        const remove = element("button", "button secondary", "Remove");
        remove.type = "button"; remove.dataset.unavailable = String(passkey.removable === false);
        remove.setAttribute("aria-label", `Remove ${passkey.name}`);
        remove.addEventListener("click", () => selectPasskey("remove", passkey));
        if (passkey.removable === false) text.append(element("span", "", "Add another passkey before removing this one."));
        actions.append(rename, remove); row.append(text, actions); return row;
      });
      $("passkey-list").replaceChildren(...rows);
      $("passkey-only-hint").hidden = !data.passkey_only;
      $("passkey-unavailable").hidden = RiAuth.passkeysAvailable();
      $("add-passkey").textContent = data.passkeys.length ? "Add another passkey" : "Add a passkey";
      securityStatus(data.passkeys.length ? `${data.passkeys.length} of ${data.limit} passkeys.${data.passkeys.length >= data.limit ? " Remove a passkey before adding another." : ""}` : "You have no passkeys yet.");
      if (data.terminal) showReauth(hint("terminal"));
      else if (!data.fresh) showReauth(hint("fresh"));
      else if (data.passkeys.length ? !data.can_rename : !data.can_register) showReauth(hint("mfa"));
      else hideReauth();
      securityControls();
    } catch (error) {
      if (generation !== security.generation) return;
      if (error.status === 401) { refresh(); return; }
      securityStatus(describe(error, "Couldn't load your passkeys. Close this dialog and try again."));
    }
  }
  async function openSecurity(kind) {
    if (!state.data) return;
    if (!$("security-dialog").open) $("security-dialog").showModal();
    $("security-account").textContent = `${state.data.user.display_name} (@${state.data.user.username})`;
    if (!$("passkey-name").value) $("passkey-name").value = navigator.userAgentData?.platform || "This device";
    securityControls(); await Promise.all([loadPasskeys(), loadFactor()]);
    if (kind && $("security-dialog").open) showReauth(hint(kind));
  }
  function factorChanged(message) {
    state.generation += 1; $("security-dialog").close();
    clearIdentity(); stopRequest(); screen("auth"); connection("Not signed in");
    $("auth-description").textContent = "Your sessions have ended. Sign in again to return to your applications.";
    toast(message);
    ($("passkey-login").hidden ? $("login-username") : $("passkey-login")).focus();
  }
  function securityError(error, retry) {
    if (error.code === "reauthentication_required" || error.code === "mfa_required") {
      security.retry = retry; showReauth(hint(error.code === "mfa_required" ? "mfa" : security.data?.terminal ? "terminal" : "fresh", "your passkeys")); return;
    }
    if (error.status === 401 || ["account_changed", "account_mismatch"].includes(error.code)) { refresh(); return; }
    if (error.name === "InvalidStateError") { securityStatus("This device already has a passkey for your account. Choose another device or security key."); return; }
    if (error.name === "NotAllowedError" || error.name === "AbortError") { securityStatus("Adding the passkey was cancelled or timed out. Select the add button to try again, or Cancel change."); return; }
    securityStatus(describe(error, error.description || "Couldn't change your passkeys. Try again."));
  }
  async function cancelChange() {
    if (security.committing || security.flows.add?.finishing || security.flows.reauth?.finishing) {
      securityStatus("Finishing your change. Please wait."); return false;
    }
    security.generation += 1;
    const add = security.flows.add, reauth = security.flows.reauth, enrolling = !!factor.enrollment;
    security.flows.add = null; security.flows.name = null; security.retry = null; security.action = null;
    security.busy = false; security.run = null;
    factor.enrollment = null; factor.action = null; clearEnrollment();
    $("passkey-action").hidden = true; $("passkey-rename").value = ""; hideReauth(); closePassword();
    securityControls();
    // A setup key shown in this dialog is gone once it closes, so its enrollment goes too.
    await Promise.all([add?.cancel(), reauth?.cancel(), enrolling && RiAuth.post("api/portal/mfa/totp/cancel", {}).catch(() => {})]);
    return true;
  }
  function selectPasskey(kind, passkey) {
    if (security.busy) return;
    void security.flows.add?.cancel(); security.flows.add = null; security.flows.name = null;
    security.retry = null; security.action = { kind, passkey };
    $("passkey-action").hidden = false;
    $("passkey-action-title").textContent = kind === "rename" ? `Rename ${passkey.name}` : `Remove ${passkey.name}?`;
    $("passkey-action-description").textContent = kind === "rename" ? "Choose a name that helps you recognize this device or security key." : "This passkey will stop working for this account and all your sessions will end. Make sure you have another way to sign in.";
    $("passkey-rename-field").hidden = kind !== "rename";
    $("passkey-rename").value = kind === "rename" ? passkey.name : "";
    $("passkey-action-confirm").textContent = kind === "rename" ? "Save name" : "Remove passkey";
    securityControls();
    $(kind === "rename" ? "passkey-rename" : "passkey-action-title").focus();
  }
  async function changePasskey(kind) {
    if (security.busy || !state.data || !security.data) return;
    const action = security.action;
    const name = $(kind === "rename" ? "passkey-rename" : "passkey-name").value.trim();
    if (kind !== "remove" && !name) {
      securityStatus("Enter a name for this passkey."); $(kind === "rename" ? "passkey-rename" : "passkey-name").focus(); return;
    }
    if (kind !== "add" && (!action || action.kind !== kind)) return;
    if (!canChange(kind)) {
      security.retry = () => changePasskey(kind); showReauth(verificationHint("your passkeys")); securityControls(); return;
    }
    const generation = security.generation, user = state.data.user.id, run = {};
    const current = () => generation === security.generation && user === state.data?.user.id;
    security.run = run; security.busy = true; securityControls();
    try {
      if (kind === "add") {
        if (security.flows.name !== name) {
          await security.flows.add?.cancel();
          if (!current()) return;
          security.flows.name = name;
          security.flows.add = RiAuth.passkeyFlow(
            () => RiAuth.post("api/portal/passkeys/registration/start", { name, expected_user_id: user }, { retry: true }),
            (credential, started) => RiAuth.post("api/portal/passkeys/registration/finish", { ceremony: started.ceremony, credential }), true,
            (started) => RiAuth.post("api/portal/passkeys/registration/cancel", { ceremony: started.ceremony }));
        }
        await security.flows.add();
        if (current()) factorChanged("Passkey added. Sign in with it to continue."); else await refresh();
      } else {
        security.committing = true;
        await RiAuth.post(`api/portal/passkeys/${encodeURIComponent(action.passkey.id)}/${kind}`, kind === "rename" ? { name } : {});
        if (!current()) { await refresh(); return; }
        if (kind === "remove") factorChanged("Passkey removed. Sign in again.");
        else {
          security.action = null; $("passkey-action").hidden = true; $("passkey-rename").value = "";
          await loadPasskeys(); securityStatus("Passkey renamed.");
        }
      }
    } catch (error) { if (current()) securityError(error, () => changePasskey(kind)); }
    finally {
      if (security.run === run) { security.busy = false; security.committing = false; security.run = null; securityControls(); }
    }
  }
  // Password change. The form proves the current password; the server also requires this
  // browser's own session and, once TOTP or a passkey is enrolled, MFA from the last five
  // minutes. Password fields are cleared on every submit, close and page hide.
  function clearPassword() {
    for (const id of ["password-current", "password-new", "password-confirm"]) { $(id).value = ""; $(id).removeAttribute("aria-invalid"); }
  }
  function renderPassword() {
    const data = security.data, local = data?.password === "local";
    $("password-status").textContent = !data ? "" : local
      ? "Changing your password signs you out everywhere. Your passkeys and authenticator app stay as they are."
      : data.password === "directory"
        ? "Your organization's directory manages your password. Change it there."
        : "This account has no password here. It signs in with a passkey or another account.";
    $("password-change-start").hidden = !local || security.passwordOpen;
    $("password-change").hidden = !local || !security.passwordOpen;
  }
  function closePassword() {
    security.passwordOpen = false; clearPassword(); $("password-error").hidden = true;
    renderPassword();
  }
  function passwordProblem(id, message) {
    if (id) $(id).setAttribute("aria-invalid", "true");
    showError("password-error", message);
  }
  function openPassword() {
    const data = security.data;
    if (!data || data.password !== "local" || security.busy) return;
    if (!data.can_change_password) {
      security.retry = openPassword; showReauth(verificationHint("your password")); securityControls(); return;
    }
    if (security.retry === openPassword) { security.retry = null; hideReauth(); }
    security.passwordOpen = true; $("password-error").hidden = true;
    securityControls();
    $("password-current").focus();
  }
  async function changePassword() {
    if (security.busy || !state.data || !security.data || !security.passwordOpen) return;
    const current = $("password-current").value, next = $("password-new").value, again = $("password-confirm").value;
    $("password-error").hidden = true;
    for (const id of ["password-current", "password-new", "password-confirm"]) $(id).removeAttribute("aria-invalid");
    if (!current) { passwordProblem("password-current", "Enter your current password."); return; }
    if (next.length < 12) { passwordProblem("password-new", "Use at least 12 characters for your new password."); return; }
    if (next !== again) { passwordProblem("password-confirm", "Your new passwords do not match."); return; }
    if (next === current) { passwordProblem("password-new", "Choose a new password that differs from your current one."); return; }
    if (!security.data.can_change_password) { openPassword(); return; }
    const generation = security.generation, user = state.data.user.id, run = {};
    const live = () => generation === security.generation && user === state.data?.user.id;
    clearPassword();
    security.run = run; security.busy = true; security.committing = true; securityControls();
    try {
      // Not retried: a replay after a lost response would fail against the new password.
      await RiAuth.post("api/portal/password", { current_password: current, password: next });
      if (live()) factorChanged("Password changed. Sign in with your new password."); else await refresh();
    } catch (error) {
      if (!live()) return;
      if (error.code === "reauthentication_required" || error.code === "mfa_required") {
        security.retry = openPassword; security.passwordOpen = false;
        showReauth(hint(error.code === "mfa_required" ? "mfa" : security.data?.terminal ? "terminal" : "fresh", "your password"));
      } else if (error.code === "invalid_current_password") {
        passwordProblem("password-current", "Your current password is incorrect. After several failed attempts, password sign-in pauses for 15 minutes.");
      } else if (error.code === "password_unavailable") {
        closePassword(); await loadPasskeys();
      } else if (error.status === 401 || ["account_changed", "account_mismatch"].includes(error.code)) {
        refresh();
      } else if (error.status === 429) {
        passwordProblem(null, "Too many attempts. Wait a few minutes before trying again.");
      } else {
        passwordProblem(null, describe(error, error.description || "Couldn't change your password. Try again."));
      }
    } finally {
      if (security.run === run) { security.busy = false; security.committing = false; security.run = null; securityControls(); }
    }
  }
  async function reauthenticated() {
    hideReauth();
    await refresh();
    if (!state.data || !$("security-dialog").open) return;
    await Promise.all([loadPasskeys(), loadFactor()]);
    const retry = security.retry; security.retry = null;
    if (retry) await retry();
  }
  function reauthError(error) {
    if (error.code === "no_passkey") { showError("reauth-error", "Your account has no passkey yet. Confirm with your password and authenticator code."); return; }
    if (["account_changed", "account_mismatch"].includes(error.code) || error.status === 401) { refresh(); return; }
    showError("reauth-error", describe(error, error.description || "Couldn't confirm it's you. Try again."));
  }
  async function reauthenticate(password = null) {
    if (security.busy || !state.data) return;
    const generation = security.generation, user = state.data.user.id, username = state.data.user.username, run = {};
    security.run = run; security.busy = true; securityControls(); $("reauth-error").hidden = true;
    try {
      if (password !== null) {
        security.committing = true;
        await security.flows.reauth.cancel();
        await RiAuth.post("api/portal/login/password", { username, password, otp: code($("reauth-otp").value), reauthenticate: true });
      } else await security.flows.reauth();
      if (generation !== security.generation || user !== state.data?.user.id) { await refresh(); return; }
      security.busy = false; security.committing = false; security.run = null;
      await reauthenticated();
    } catch (error) { if (generation === security.generation) reauthError(error); }
    finally {
      $("reauth-password").value = ""; $("reauth-otp").value = "";
      if (security.run === run) { security.busy = false; security.committing = false; security.run = null; securityControls(); }
    }
  }
  $("reauth-passkey").addEventListener("click", () => reauthenticate());
  $("reauth-form").addEventListener("submit", (event) => {
    event.preventDefault();
    const password = $("reauth-password").value;
    if (!password) { showError("reauth-error", "Enter your password."); return; }
    reauthenticate(password);
  });
  $("passkey-form").addEventListener("submit", (event) => { event.preventDefault(); changePasskey("add"); });
  $("password-change-start").addEventListener("click", openPassword);
  $("password-change").addEventListener("submit", (event) => { event.preventDefault(); changePassword(); });
  $("password-change-cancel").addEventListener("click", () => { closePassword(); securityControls(); $("password-change-start").focus(); });
  for (const id of ["password-current", "password-new", "password-confirm"]) $(id).addEventListener("input", () => $(id).removeAttribute("aria-invalid"));
  window.addEventListener("pagehide", clearPassword);
  $("passkey-action").addEventListener("submit", (event) => { event.preventDefault(); if (security.action) changePasskey(security.action.kind); });
  $("passkey-cancel").addEventListener("click", async () => {
    if (await cancelChange()) { await Promise.all([loadPasskeys(), loadFactor()]); securityStatus("Change cancelled."); }
  });

  // Authenticator app (TOTP) and recovery codes. Every change runs for the account and dialog
  // that started it; enabling, replacing and removing end every session, this one included.
  function factorStatus(message) { $("factor-status").textContent = message; }
  function factorReady() {
    const data = factor.data;
    return !!data && data.fresh && !data.terminal && (!data.factor || data.mfa);
  }
  function factorHint(what) {
    return hint(factor.data?.terminal ? "terminal" : factor.data?.fresh === false ? "fresh" : "mfa", what);
  }
  function renderFactor() {
    const data = factor.data, enabled = data?.totp_enabled === true, idle = !factor.enrollment && !factor.action;
    $("totp-section").hidden = !data; $("recovery-section").hidden = !enabled;
    $("totp-enroll").hidden = !factor.enrollment; $("factor-confirm").hidden = !factor.action;
    if (!data) return;
    $("totp-summary").textContent = enabled
      ? "On. After your password, enter the 6-digit code your authenticator app shows."
      : "Off. Use an authenticator app on your phone or computer for 6-digit sign-in codes after your password.";
    $("totp-setup").hidden = enabled || !idle;
    $("totp-replace").hidden = $("totp-remove").hidden = !enabled || !idle;
    $("recovery-rotate").hidden = !idle;
    const left = data.recovery_codes_remaining;
    $("recovery-summary").textContent = left > 0
      ? `${left} of ${data.recovery_codes_total} recovery codes left. Each one signs you in once instead of an app code.${left <= 3 ? " Create new codes soon." : ""}`
      : "You have no recovery codes. Create a set so you can still sign in if you lose your authenticator app.";
    for (const id of ["totp-setup", "totp-replace", "totp-remove", "recovery-rotate", "totp-code", "totp-confirm", "totp-enroll-cancel", "totp-copy-key", "factor-confirm-submit", "factor-confirm-cancel"]) $(id).disabled = security.busy;
  }
  async function loadFactor() {
    const generation = security.generation, user = state.data?.user.id;
    try {
      const data = await RiAuth.get("api/portal/mfa");
      if (generation !== security.generation || state.data?.user.id !== user || !$("security-dialog").open) return;
      if (data.user_id !== user) { $("security-dialog").close(); await refresh(); return; }
      factor.data = data; renderFactor();
    } catch (error) {
      if (generation !== security.generation) return;
      if (error.status === 401) { refresh(); return; }
      factorStatus(describe(error, "Couldn't load your authenticator app settings. Close this dialog and try again."));
    }
  }
  // The retry waits in the shared panel; the factor's own cancel buttons drop it.
  function needProof(retry, text) {
    retry.factor = true; security.retry = retry; showReauth(text); securityControls();
    ($("reauth-passkey").hidden ? $("reauth-password") : $("reauth-passkey")).focus();
  }
  function dropProof() {
    if (security.retry?.factor) { security.retry = null; hideReauth(); securityControls(); }
  }
  function factorError(error, retry, what) {
    if (error.code === "reauthentication_required" || error.code === "mfa_required") {
      needProof(retry, hint(error.code === "mfa_required" ? "mfa" : factor.data?.terminal ? "terminal" : "fresh", what)); return;
    }
    if (error.status === 401 || error.code === "account_mismatch") { refresh(); return; }
    if (error.status === 409) void loadFactor();
    factorStatus(describe(error, error.description || "Couldn't change your authenticator app. Try again."));
  }
  // `commit` marks a request that changes credentials: the dialog stays until it settles.
  async function factorRun(commit, retry, work, what = "your authenticator app") {
    if (security.busy || !state.data || !factor.data) return;
    if (!factorReady()) { needProof(retry, factorHint(what)); return; }
    const generation = security.generation, user = state.data.user.id, run = {};
    const current = () => generation === security.generation && user === state.data?.user.id;
    security.run = run; security.busy = true; security.committing = commit; securityControls(); factorStatus("");
    try { await work(user, current); }
    catch (error) { if (current()) factorError(error, retry, what); }
    finally {
      if (security.run === run) { security.busy = false; security.committing = false; security.run = null; securityControls(); }
    }
  }
  function clearEnrollment() {
    $("totp-qr").replaceChildren(); $("totp-key").textContent = ""; $("totp-key-details").textContent = "";
    $("totp-code").value = ""; $("totp-open").removeAttribute("href"); $("totp-error").hidden = true;
  }
  function endEnrollment() { factor.enrollment = null; clearEnrollment(); renderFactor(); }
  function showEnrollment(started, replace) {
    factor.enrollment = { replace, digits: Number(started.digits) || 6, secret: String(started.secret || "") };
    const size = Number(started.qr?.size);
    if (Number.isInteger(size) && size > 0 && typeof started.qr.path === "string") {
      const ns = "http://www.w3.org/2000/svg", svg = document.createElementNS(ns, "svg"), path = document.createElementNS(ns, "path");
      svg.setAttribute("viewBox", `-4 -4 ${size + 8} ${size + 8}`); svg.setAttribute("role", "img");
      svg.setAttribute("aria-label", "QR code of your setup key. The same key is shown as text below.");
      path.setAttribute("d", started.qr.path); svg.append(path); $("totp-qr").replaceChildren(svg);
    } else $("totp-qr").replaceChildren();
    $("totp-key").textContent = factor.enrollment.secret.replace(/(.{4})(?=.)/g, "$1 ");
    $("totp-key-details").textContent = `Time-based, ${factor.enrollment.digits} digits, new code every ${Number(started.period) || 30} seconds (${started.algorithm || "SHA1"}). Spaces in the key are optional.`;
    const uri = typeof started.otpauth_uri === "string" && started.otpauth_uri.startsWith("otpauth://totp/") ? started.otpauth_uri : null;
    $("totp-open").parentElement.hidden = !uri;
    if (uri) $("totp-open").href = uri;
    $("totp-enroll-title").textContent = replace ? "Replace your authenticator app" : "Set up your authenticator app";
    $("totp-confirm").textContent = replace ? "Verify and replace" : "Verify and turn on";
    $("totp-code").removeAttribute("aria-invalid"); $("totp-error").hidden = true;
    factorStatus(replace ? "Your current app keeps working until the new one is verified." : "");
    renderFactor(); $("totp-enroll-title").focus();
  }
  function startTotp(replace) {
    return factorRun(false, () => startTotp(replace), async (user, current) => {
      const started = await RiAuth.post("api/portal/mfa/totp/start", { expected_user_id: user, replace });
      if (current()) showEnrollment(started, replace);
      else await RiAuth.post("api/portal/mfa/totp/cancel", {}).catch(() => {});
    });
  }
  function confirmTotp() {
    const enrollment = factor.enrollment;
    if (!enrollment || security.busy) return;
    const code = $("totp-code").value.replace(/\s+/g, "");
    if (!new RegExp(`^\\d{${enrollment.digits}}$`).test(code)) {
      $("totp-code").setAttribute("aria-invalid", "true");
      showError("totp-error", `Enter the ${enrollment.digits}-digit code your authenticator app shows.`); return;
    }
    return factorRun(true, () => confirmTotp(), async (user) => {
      const username = state.data.user.username;
      let result;
      try { result = await RiAuth.post("api/portal/mfa/totp/confirm", { expected_user_id: user, code }); }
      catch (error) {
        if (error.code === "invalid_code") {
          $("totp-code").value = ""; $("totp-code").setAttribute("aria-invalid", "true");
          showError("totp-error", "That code didn't match. Enter the newest code from your app."); return;
        }
        if (["enrollment_expired", "enrollment_not_found"].includes(error.code)) {
          endEnrollment(); factorStatus("This setup ended before it was verified. Start again for a new setup key."); void loadFactor(); return;
        }
        throw error;
      }
      const replaced = result.status === "replaced";
      endEnrollment();
      factorChanged(replaced ? "Authenticator app replaced. Sign in with a code from your new app." : "Authenticator app turned on. Sign in with your password and a code from it.");
      showCodes(result.recovery_codes, replaced ? "replaced" : "enabled", username);
    });
  }
  function askFactor(kind) {
    if (security.busy || !factor.data) return;
    const remove = kind === "remove";
    factor.action = kind; RiAuth.arm();
    $("factor-confirm-title").textContent = remove ? "Remove your authenticator app?" : "Create new recovery codes?";
    $("factor-confirm-description").textContent = remove
      ? "Codes from the app and your recovery codes will stop working, and you'll be signed out everywhere. Applications that need extra verification will then need a passkey."
      : "Your current recovery codes stop working as soon as the new ones are created.";
    $("factor-confirm-submit").textContent = remove ? "Remove authenticator app" : "Create new codes";
    factorStatus(""); renderFactor(); $("factor-confirm-title").focus();
  }
  function runFactorAction() {
    const kind = factor.action;
    if (!kind) return;
    return factorRun(true, () => runFactorAction(), async (user) => {
      const username = state.data.user.username;
      if (kind === "remove") {
        await RiAuth.post("api/portal/mfa/totp/remove", { expected_user_id: user });
        factor.action = null; factorChanged("Authenticator app removed. Sign in again.");
        return;
      }
      const result = await RiAuth.post("api/portal/mfa/recovery-codes", { expected_user_id: user });
      factor.action = null; renderFactor();
      // The previous codes are already void; this response is the only copy of the new ones.
      showCodes(result.recovery_codes, "rotated", username);
      void loadFactor();
    }, kind === "remove" ? "your authenticator app" : "your recovery codes");
  }
  function showCodes(codes, kind, username) {
    const list = Array.isArray(codes) ? codes.filter((code) => typeof code === "string") : [];
    factor.codes = { list, username };
    $("codes-title").textContent = kind === "rotated" ? "Save your new recovery codes" : kind === "replaced" ? "Authenticator app replaced" : "Authenticator app turned on";
    $("codes-description").textContent = `${kind === "rotated" ? "Your previous recovery codes no longer work." : kind === "replaced" ? "Your old app and recovery codes no longer work, and you're signed out everywhere." : "You're signed out everywhere. Sign in again with your password and a code from your app."} Save these recovery codes somewhere safe, like a password manager. Each one signs you in once if you can't use your authenticator app. They won't be shown again.`;
    $("codes-list").replaceChildren(...list.map((code) => element("li", "", code)));
    $("codes-saved").checked = false; $("codes-status").textContent = "";
    if (!$("codes-dialog").open) $("codes-dialog").showModal();
    RiAuth.arm(); $("codes-title").focus();
  }
  function closeCodes() {
    factor.codes = null; $("codes-list").replaceChildren(); $("codes-saved").checked = false; $("codes-status").textContent = "";
    if ($("codes-dialog").open) $("codes-dialog").close();
    if ($("security-dialog").open) $("recovery-rotate").focus();
  }
  async function copyText(text, target, status, done, fallback) {
    try { await navigator.clipboard.writeText(text); status(done); }
    catch {
      const selection = window.getSelection(), range = document.createRange();
      range.selectNodeContents(target); selection.removeAllRanges(); selection.addRange(range); status(fallback);
    }
  }
  $("totp-setup").addEventListener("click", () => startTotp(false));
  $("totp-replace").addEventListener("click", () => startTotp(true));
  $("totp-remove").addEventListener("click", () => askFactor("remove"));
  $("recovery-rotate").addEventListener("click", () => askFactor("rotate"));
  $("totp-enroll").addEventListener("submit", (event) => { event.preventDefault(); confirmTotp(); });
  $("totp-code").addEventListener("input", () => $("totp-code").removeAttribute("aria-invalid"));
  $("totp-copy-key").addEventListener("click", () => {
    if (factor.enrollment) void copyText(factor.enrollment.secret, $("totp-key"), factorStatus, "Setup key copied.", "Select the key and copy it with your keyboard.");
  });
  $("totp-enroll-cancel").addEventListener("click", async () => {
    if (security.busy || !factor.enrollment) return;
    endEnrollment(); dropProof(); factorStatus("Setup cancelled.");
    ($("totp-setup").hidden ? $("totp-replace") : $("totp-setup")).focus();
    try { await RiAuth.post("api/portal/mfa/totp/cancel", {}); } catch { /* It expires on its own. */ }
  });
  $("factor-confirm").addEventListener("submit", (event) => event.preventDefault());
  RiAuth.guard($("factor-confirm-submit"), (event) => { event.preventDefault(); runFactorAction(); });
  $("factor-confirm-cancel").addEventListener("click", () => {
    const kind = factor.action; factor.action = null; dropProof(); renderFactor();
    (kind === "remove" ? $("totp-remove") : $("recovery-rotate")).focus();
  });
  $("codes-copy").addEventListener("click", () => {
    if (factor.codes) void copyText(factor.codes.list.join("\n"), $("codes-list"), (message) => { $("codes-status").textContent = message; }, "Recovery codes copied.", "Select the codes and copy them with your keyboard.");
  });
  $("codes-download").addEventListener("click", () => {
    if (!factor.codes) return;
    const heading = `riAuth recovery codes${factor.codes.username ? ` for ${factor.codes.username}` : ""} at ${location.host}`;
    const url = URL.createObjectURL(new Blob([`${heading}\nEach code signs you in once.\n\n${factor.codes.list.join("\n")}\n`], { type: "text/plain;charset=utf-8" }));
    const link = element("a"); link.href = url; link.download = "riauth-recovery-codes.txt";
    document.body.append(link); link.click(); link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  });
  RiAuth.guard($("codes-done"), () => {
    if ($("codes-saved").checked) { closeCodes(); return; }
    $("codes-status").textContent = "Save the codes, then confirm that you saved them."; $("codes-saved").focus();
  });
  $("codes-dialog").addEventListener("cancel", (event) => {
    if (!factor.codes || $("codes-saved").checked) return;
    event.preventDefault();
    $("codes-status").textContent = "Save your recovery codes before closing. They won't be shown again."; $("codes-saved").focus();
  });
  $("codes-dialog").addEventListener("close", () => { if (factor.codes) closeCodes(); });
  $("account-security").addEventListener("click", () => openSecurity());
  $("workspace-settings").addEventListener("click", () => {
    if (!state.data) return;
    $("settings-account").textContent = `${state.data.user.display_name} (@${state.data.user.username})`;
    $("appearance").value = RiAuthAppearance.get();
    $("settings-dialog").showModal();
  });
  $("settings-close").addEventListener("click", () => $("settings-dialog").close());
  $("settings-dialog").addEventListener("close", () => { if (state.data) $("workspace-settings").focus(); });
  $("settings-security").addEventListener("click", () => { $("settings-dialog").close(); openSecurity(); });
  async function closeSecurity() { if (await cancelChange()) $("security-dialog").close(); }
  $("security-close").addEventListener("click", closeSecurity);
  $("security-dialog").addEventListener("cancel", (event) => { event.preventDefault(); closeSecurity(); });
  $("security-dialog").addEventListener("close", () => {
    void cancelChange();
    if (!$("account").hidden) $("account-security").focus();
  });
  $("mfa-action").addEventListener("click", () => {
    if (!state.data) return;
    if (!state.data.mfa_available || !RiAuthCapabilities.usable("identity.passkeys") || !RiAuth.passkeysAvailable()) { openSecurity(state.data.mfa_available ? "mfa" : null); return; }
    RiAuth.inFlight($("mfa-action"), async () => {
      try { await security.flows.reauth(); await refresh(); }
      catch (error) {
        if (error.code === "no_passkey") openSecurity("mfa");
        else toast(describe(error, "Couldn't sign in with your passkey. Try again."));
      }
    });
  });

  $("passkey-login").hidden = $("auth-divider").hidden = !RiAuth.passkeysAvailable();
  resetFlows();
  refresh();
})();
