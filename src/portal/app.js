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
    for (const id of ["catalogue", "auth", "error", "loading"]) $(id).hidden = id !== name;
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
    $("nav-all").disabled = true; $("nav-favorites").disabled = true;
    $("all-count").textContent = "—"; $("favorite-count").textContent = "—";
    $("search").value = ""; $("category").replaceChildren(new Option("All categories", ""));
    $("nav-all").classList.add("active"); $("nav-all").setAttribute("aria-current", "page");
    $("nav-favorites").classList.remove("active"); $("nav-favorites").removeAttribute("aria-current");
    $("mfa-notice").hidden = true; $("verification-notice").hidden = true; $("passkey-list").replaceChildren();
    if ($("security-dialog").open) $("security-dialog").close();
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
      const data = await api("");
      if (generation !== state.generation) return;
      const changedUser = state.data?.user.id !== data.user.id;
      if (changedUser && $("security-dialog").open) $("security-dialog").close();
      state.data = data;
      if (changedUser) {
        loadPreferences(); state.section = "all"; $("search").value = ""; resetFlows();
        state.verificationMessage = null; state.verificationRequested = false; state.verificationNeedsRelogin = false;
      }
      const accessible = new Set(data.apps.map((app) => app.id));
      state.favorites = new Set([...state.favorites].filter((id) => accessible.has(id)));
      savePreferences(); stopRequest();
      $("account").hidden = false; $("signed-out-label").hidden = true;
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
      $("mfa-notice-text").textContent = data.mfa_available ? "Some applications need your passkey or authenticator code." : "Some applications need extra verification. Add a passkey under Sign-in and security.";
      $("mfa-action").textContent = data.mfa_available ? "Sign in with your passkey" : "Sign-in and security";
      clearAuthError();
      const category = changedUser ? "" : $("category").value;
      $("category").replaceChildren(new Option("All categories", ""));
      [...new Set(data.apps.map((app) => app.category))].sort((a, b) => a.localeCompare(b)).forEach((name) => $("category").add(new Option(name, name)));
      $("category").value = [...$("category").options].some((option) => option.value === category) ? category : "";
      screen("catalogue"); connection("Connected", true); render();
      clearTimeout(expiryTimer);
      expiryTimer = setTimeout(refresh, Math.max(1000, Math.min(30000, data.expires_at * 1000 - Date.now())));
    } catch (error) {
      if (generation !== state.generation) return;
      const hadSession = !!state.data; clearIdentity();
      if (error.status === 401) {
        screen("auth"); connection("Not signed in");
        $("auth-description").textContent = hadSession ? "Your session has ended. Sign in again to return to your applications." : "Sign in to see the applications available to you. Your workspace is ready when you are.";
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
    if (!state.data || event.isComposing || $("security-dialog").open) return;
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
  }
  async function signedIn() { $("login-password").value = ""; $("login-otp").value = ""; stopRequest(); await refresh(); }
  $("password-form").addEventListener("submit", (event) => {
    event.preventDefault();
    RiAuth.inFlight($("password-login"), async () => {
      const username = $("login-username").value.trim(), password = $("login-password").value;
      $("login-password").value = "";
      clearAuthError();
      if (!username || !password) { showError("auth-error", "Enter your username and password."); return; }
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
  for (const id of ["login-username", "login-password", "login-otp"]) $(id).addEventListener("input", () => $(id).removeAttribute("aria-invalid"));
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
    $("passkey-form").hidden = !data || pending || !RiAuth.passkeysAvailable() || data.passkeys.length >= data.limit;
    for (const button of $("passkey-list").querySelectorAll("button")) button.disabled = security.busy || button.dataset.unavailable === "true";
    for (const id of ["add-passkey", "passkey-action-confirm", "reauth-passkey", "reauth-confirm", "password-change-start", "password-change-submit", "password-change-cancel", "password-current", "password-new", "password-confirm"]) $(id).disabled = security.busy;
    $("passkey-name").disabled = $("passkey-rename").disabled = security.busy;
    // A password verification waits in the shared panel; Cancel change belongs to passkeys.
    $("passkey-cancel").hidden = !pending && !security.busy && !security.flows.add && (!security.retry || security.retry === openPassword);
    renderPassword();
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
    securityControls(); await loadPasskeys();
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
    const add = security.flows.add, reauth = security.flows.reauth;
    security.flows.add = null; security.flows.name = null; security.retry = null; security.action = null;
    security.busy = false; security.run = null;
    $("passkey-action").hidden = true; $("passkey-rename").value = ""; hideReauth(); closePassword();
    securityControls();
    await Promise.all([add?.cancel(), reauth?.cancel()]);
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
    await loadPasskeys();
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
    if (await cancelChange()) { await loadPasskeys(); securityStatus("Change cancelled."); }
  });
  $("account-security").addEventListener("click", () => openSecurity());
  async function closeSecurity() { if (await cancelChange()) $("security-dialog").close(); }
  $("security-close").addEventListener("click", closeSecurity);
  $("security-dialog").addEventListener("cancel", (event) => { event.preventDefault(); closeSecurity(); });
  $("security-dialog").addEventListener("close", () => {
    void cancelChange();
    if (!$("account").hidden) $("account-security").focus();
  });
  $("mfa-action").addEventListener("click", () => {
    if (!state.data) return;
    if (!state.data.mfa_available || !RiAuth.passkeysAvailable()) { openSecurity(state.data.mfa_available ? "mfa" : null); return; }
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
