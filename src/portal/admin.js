"use strict";
// Compact administration. Every call goes to /api/admin/*, which runs the same management
// methods, permission checks, validation and audit as the bearer API. Edits send the
// configuration revision they were made against (If-Match) and one Idempotency-Key per
// attempt, so a stale form is rejected and a retried submit is applied at most once.
(() => {
  const $ = (id) => document.getElementById(id);
  const base = document.querySelector("meta[name=riauth-base]").content;
  const SECTIONS = { applications: "Applications", people: "People", groups: "Groups", security: "Security" };
  const ICONS = ["app", "code", "chart", "files", "messages", "book", "cloud", "terminal", "shield", "globe"];
  const ACCENTS = ["violet", "blue", "teal", "amber", "rose", "slate"];
  const CONFLICT = "The configuration changed after this page loaded, so this edit was not saved. Reload to review the latest values, then try again.";
  const data = { me: null, revision: 0, clients: [], users: [], groups: [], requests: [], grants: [], audit: [], invitations: [], mail: false, lifetime: 0 };
  const platform = () => data.me && data.me.edition === "platform";
  let generation = 0, loaded = false, toastTimer, confirmRun = null, confirmOpener = null;
  let draft = null; // the application setup wizard's draft, see newApplication
  let captureWizard = null; // reads the open wizard step's unsaved fields into the draft

  class ApiError extends Error {
    constructor(status, code, message) { super(message); this.status = status; this.code = code; }
  }

  // ---- DOM helpers -------------------------------------------------------------------------
  function h(tag, attrs = {}, ...children) {
    const node = document.createElement(tag);
    for (const [key, value] of Object.entries(attrs)) {
      if (value === undefined || value === null || value === false) continue;
      if (key === "class") node.className = value;
      else if (key === "text") node.textContent = value;
      else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
      else if (["hidden", "disabled", "checked", "required", "readOnly", "value", "selected"].includes(key)) node[key] = value;
      else node.setAttribute(key, value === true ? "" : String(value));
    }
    for (const child of children.flat(Infinity)) {
      if (child === undefined || child === null || child === false) continue;
      node.append(child instanceof Node ? child : document.createTextNode(String(child)));
    }
    return node;
  }
  function icon(name) {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("class", "icon"); svg.setAttribute("aria-hidden", "true");
    const use = document.createElementNS(svg.namespaceURI, "use");
    use.setAttribute("href", `#i-${name}`); svg.append(use); return svg;
  }
  const link = (hash, text, attrs = {}) => h("a", { href: hash, ...attrs }, text);
  const badge = (text, tone = "") => h("span", { class: `badge ${tone}`.trim() }, text);
  const hash = (...parts) => "#/" + parts.map(encodeURIComponent).join("/");
  const byName = (a, b) => a.localeCompare(b, undefined, { sensitivity: "base" });
  function announce(text) { $("announcement").textContent = ""; setTimeout(() => { $("announcement").textContent = text; }, 30); }
  function toast(text) {
    clearTimeout(toastTimer); $("toast").textContent = text; $("toast").hidden = false; announce(text);
    toastTimer = setTimeout(() => { $("toast").hidden = true; }, 5000);
  }
  const rtf = typeof Intl.RelativeTimeFormat === "function" ? new Intl.RelativeTimeFormat(undefined, { numeric: "auto" }) : null;
  function when(seconds) {
    const date = new Date(seconds * 1000);
    const delta = seconds - Date.now() / 1000;
    const abs = Math.abs(delta);
    let text = date.toLocaleString();
    if (rtf && abs < 86400 * 7) {
      const [value, unit] = abs < 60 ? [delta, "second"] : abs < 3600 ? [delta / 60, "minute"] : abs < 86400 ? [delta / 3600, "hour"] : [delta / 86400, "day"];
      text = rtf.format(Math.round(value), unit);
    }
    return h("time", { datetime: date.toISOString(), title: date.toLocaleString() }, text);
  }
  function duration(seconds) {
    if (seconds % 3600 === 0) return `${seconds / 3600} h`;
    if (seconds % 60 === 0) return `${seconds / 60} min`;
    return `${seconds} s`;
  }
  const lines = (text) => text.split("\n").map((line) => line.trim()).filter(Boolean);
  const words = (text) => text.split(/[\s,]+/).map((word) => word.trim()).filter(Boolean);
  const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
  const sorted = (items) => [...items].sort();

  // ---- API ---------------------------------------------------------------------------------
  function requestKey() {
    const bytes = new Uint8Array(16); crypto.getRandomValues(bytes);
    return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  }
  async function api(method, path, body, options = {}) {
    const headers = { "X-Riauth-Portal": "1", "Accept": "application/json" };
    if (body !== undefined) headers["Content-Type"] = "application/json";
    if (options.revision !== undefined) headers["If-Match"] = `"${options.revision}"`;
    if (options.key) headers["Idempotency-Key"] = options.key;
    let response;
    try {
      response = await fetch(`${base}api/${path}`, {
        method, headers, body: body === undefined ? undefined : JSON.stringify(body),
        // same-origin, as in auth.js: under no-referrer a same-origin POST carries
        // "Origin: null" (Fetch spec, followed by Safari) and the write guard refuses it.
        credentials: "same-origin", mode: "same-origin", cache: "no-store", referrerPolicy: "same-origin",
      });
    } catch {
      throw new ApiError(0, "network", "");
    }
    let payload = null;
    try { payload = await response.json(); } catch { payload = null; }
    if (!response.ok) {
      throw new ApiError(response.status, payload && payload.error || "", payload && payload.error_description || "");
    }
    return payload;
  }
  const seg = encodeURIComponent;
  const revisionConflict = (error) => error.status === 409 && /revision/i.test(error.message);
  // Explains a failed call. `overrides` maps a status to context-specific text, or to a
  // function of the error that returns text only for the cause it recognizes.
  function explain(error, overrides = {}) {
    const override = overrides[error.status];
    const text = typeof override === "function" ? override(error) : override;
    if (text) return text;
    if (error.status === 0) return "riAuth didn't answer, so the change may or may not have been saved. Trying again reuses the same request key, so it is applied at most once.";
    if (revisionConflict(error)) return CONFLICT;
    if (error.status === 403) return "Your account isn't allowed to make this change.";
    if (error.status === 404) return "This item no longer exists. Someone may have removed it; reload to see the current state.";
    if (error.status === 429) return "Too many requests from this network. Wait a minute, then try again.";
    if (error.status >= 500) return "riAuth reported a server error. Reload to check whether the change was saved before trying again.";
    return error.message || "riAuth rejected this change.";
  }
  function showError(target, error, overrides) {
    target.replaceChildren(explain(error, overrides));
    if (error.status === 0 || error.status === 404 || error.status >= 500 || revisionConflict(error)) {
      target.append(" ", h("button", { class: "text-button", type: "button", onclick: () => refresh({ focus: true }) }, "Reload latest"));
    }
    target.hidden = false; target.focus();
  }
  // One pending/error/idempotency lifecycle for every form in the views.
  function bindForm(form, action, overrides, busy = "Saving…") {
    let key = null;
    const status = h("p", { class: "form-error", role: "alert", tabindex: "-1", hidden: true });
    form.querySelector(".form-actions").before(status);
    form.addEventListener("input", () => { key = null; });
    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      if (form.getAttribute("aria-busy") === "true") return;
      const button = form.querySelector("[type=submit]");
      const label = button.textContent;
      key = key || requestKey();
      form.setAttribute("aria-busy", "true"); button.disabled = true; button.textContent = busy; status.hidden = true;
      try {
        await action(key);
        key = null;
      } catch (error) {
        if (error.status === 401) { gate("signin"); return; }
        if (error.status === 422) { status.replaceChildren(error.message); status.hidden = false; status.focus(); return; }
        showError(status, error, overrides);
      } finally {
        form.removeAttribute("aria-busy"); button.disabled = false; button.textContent = form.dataset.submitLabel || label;
      }
    });
  }
  // Local validation surfaces through the same alert as server errors.
  const invalid = (message) => new ApiError(422, "invalid", message);

  // ---- Session gate and loading ------------------------------------------------------------
  function screen(name) {
    $("view").hidden = name !== "view"; $("loading").hidden = name !== "loading"; $("gate").hidden = name !== "gate";
  }
  function connection(label, live = false) {
    $("connection-label").textContent = label; $("connection").classList.toggle("live", live);
  }
  // A draft and the one-time client secret belong to the administrator session that made
  // them. Every session or account transition drops them, the views that rendered them and
  // any open secret or confirmation dialog, so a later account in this tab can't see them.
  function forget() {
    draft = null; captureWizard = null;
    $("view").replaceChildren();
    $("secret-value").value = "";
    for (const id of ["secret-dialog", "confirm-dialog"]) if ($(id).open) $(id).close();
  }
  // A one-time client secret lives only while this tab keeps focus. Losing focus, hiding or
  // leaving the page erases it from the draft and from every DOM copy at once, before any
  // other account could come back to this tab. A lost secret is replaced by rotating.
  function eraseSecrets() {
    $("secret-value").value = "";
    if ($("secret-dialog").open) $("secret-dialog").close();
    if (!draft || !draft.created || !draft.created.secret) return;
    draft.created.secret = null; draft.created.discarded = true;
    if (loaded) render(); else $("view").replaceChildren();
  }
  function gate(kind) {
    generation += 1; loaded = false;
    forget();
    const who = data.me && data.me.user ? data.me.user.username : null;
    const content = {
      signin: ["i-lock", "Sign in to administer riAuth", "Your session ended or this browser isn't signed in. Sign in on your applications page with an administrator account, then come back to this page.", "Go to sign-in", false],
      forbidden: ["i-shield", "Administrator access required", `${who ? `You're signed in as ${who}, which isn't an administrator.` : "This account isn't an administrator."} Ask an administrator for access, or sign in with another account.`, "Your applications", false],
      offline: ["i-cloud", "We couldn't reach riAuth", "Nothing was changed. Check the connection, then try again.", "Your applications", true],
    }[kind];
    $("gate-icon").firstElementChild.setAttribute("href", `#${content[0]}`);
    $("gate-title").textContent = content[1]; $("gate-text").textContent = content[2];
    $("gate-link").textContent = content[3]; $("gate-retry").hidden = !content[4];
    $("account").hidden = kind !== "forbidden" || !who;
    for (const id of ["count-applications", "count-people", "count-groups", "count-security"]) $(id).textContent = "—";
    connection(kind === "offline" ? "Offline" : "Not signed in");
    screen("gate"); $("gate-title").focus();
  }
  async function refresh(options = {}) {
    const run = ++generation;
    connection("Refreshing");
    if (!loaded) screen("loading");
    try {
      // The revision is read before the data it guards: a change in between only causes a
      // conflict, never a silent overwrite.
      const me = await api("GET", "admin/session");
      const [clients, users, groups, requests, grants, audit, invites] = await Promise.all([
        api("GET", "admin/clients"), api("GET", "admin/users"), api("GET", "admin/groups"),
        me.edition === "platform" ? api("GET", "admin/access/requests") : Promise.resolve([]),
        me.edition === "platform" ? api("GET", "admin/access/grants") : Promise.resolve([]),
        api("GET", "admin/audit?limit=50"),
        api("GET", "admin/invitations"),
      ]);
      if (run !== generation) return;
      if (draft && draft.owner !== me.user.id) forget();
      Object.assign(data, { me, revision: me.revision, clients, users, groups, requests, grants, audit });
      Object.assign(data, { invitations: invites.invitations, mail: invites.delivery_configured, lifetime: invites.lifetime });
      loaded = true;
      account(); counts(); render(options); connection("Up to date", true);
    } catch (error) {
      if (run !== generation) return;
      if (error.status === 401) gate("signin");
      else if (error.status === 403) {
        data.me = null;
        try { data.me = { user: (await api("GET", "portal")).user }; } catch { /* the gate text copes */ }
        if (data.me && data.me.user) account();
        gate("forbidden");
      } else if (loaded) { connection("Offline"); toast("Couldn't refresh. Showing the last loaded data."); }
      else gate("offline");
    }
  }
  function account() {
    const user = data.me && data.me.user;
    if (!user) return;
    const name = user.display_name || user.username;
    $("avatar").textContent = name.split(/\s+/).map((part) => part[0] || "").join("").slice(0, 2).toUpperCase();
    $("account-name").textContent = name;
    $("account-detail").textContent = user.admin ? `${user.username} · Administrator` : user.username;
    $("account").hidden = false;
  }
  const pending = () => data.requests.filter((request) => request.status === "pending");
  const activeGrants = () => data.grants.filter((grant) => !grant.revoked_at && grant.expires_at > Date.now() / 1000);
  function counts() {
    $("count-applications").textContent = data.clients.length;
    $("count-people").textContent = data.users.length;
    $("count-groups").textContent = data.groups.length;
    const waiting = pending().length;
    const badge = $("count-security");
    badge.hidden = !platform();
    if (platform()) {
      badge.textContent = waiting;
      badge.classList.toggle("attention", waiting > 0);
      badge.setAttribute("aria-label", `${waiting} access ${waiting === 1 ? "request" : "requests"} waiting for review`);
    }
  }

  // ---- Routing -----------------------------------------------------------------------------
  // Only "#/section/id" hashes are routes; in-page anchors keep the current view.
  let lastRoute = { section: "applications", id: null };
  const isRoute = () => !location.hash || location.hash.startsWith("#/");
  function route() {
    if (!isRoute()) return lastRoute;
    const parts = location.hash.replace(/^#\/?/, "").split("/").filter(Boolean).map((part) => {
      try { return decodeURIComponent(part); } catch { return ""; }
    });
    const section = Object.hasOwn(SECTIONS, parts[0]) ? parts[0] : "applications";
    lastRoute = { section, id: parts[1] || null };
    return lastRoute;
  }
  function render(options = {}) {
    if (!loaded) return;
    const { section, id } = route();
    // A refresh re-renders the open step; keep what was typed since the last Continue.
    if (captureWizard) { try { captureWizard(); } catch { /* a partial step is re-read on Continue */ } captureWizard = null; }
    // A created application's one-time secret is dropped once the wizard route is left; an
    // unfinished draft is kept so a detour (for example to create a group) can resume it.
    if (draft && draft.created && !(section === "applications" && id === "new")) draft = null;
    for (const item of document.querySelectorAll("#nav .nav-item")) {
      const active = item.dataset.section === section;
      item.classList.toggle("active", active);
      if (active) item.setAttribute("aria-current", "page"); else item.removeAttribute("aria-current");
    }
    const views = { applications: [applications, application, newApplication], people: [people, person, newPerson], groups: [groups, group, null], security: [security, null, null] }[section];
    const [list, detail, create] = views;
    const content = id === "new" && create ? create() : id && detail ? detail(id) : list();
    const view = $("view");
    view.replaceChildren(content.node);
    const crumbs = [h("a", { href: hash(section) }, SECTIONS[section])];
    if (content.crumb) crumbs.push(icon("chevron"), h("strong", { "aria-current": "page" }, content.crumb));
    else crumbs[0] = h("strong", { "aria-current": "page" }, SECTIONS[section]);
    $("breadcrumb").replaceChildren(h("span", {}, "Administration"), icon("chevron"), ...crumbs);
    document.title = `${content.crumb || SECTIONS[section]} · riAuth administration`;
    screen("view");
    if (options.focus) { const heading = view.querySelector("h1"); if (heading) heading.focus(); announce(document.title); }
  }

  // ---- Shared view pieces ------------------------------------------------------------------
  function heading(eyebrow, title, description, ...actions) {
    return h("section", { class: "page-heading" },
      h("div", {}, h("p", { class: "eyebrow" }, eyebrow), h("h1", { tabindex: "-1" }, title), description ? h("p", { class: "page-description" }, description) : null),
      actions.length ? h("div", { class: "heading-actions" }, actions) : null);
  }
  // A filterable table. Each cell carries its column label for the stacked mobile layout.
  function table(caption, columns, rows, empty, filter) {
    const body = h("tbody");
    const note = h("p", { class: "table-empty", hidden: true }, empty);
    const draw = (query = "") => {
      const shown = rows.filter((row) => !query || filter(row).toLowerCase().includes(query.toLowerCase()));
      body.replaceChildren(...shown.map((row) => h("tr", {}, columns.map((column, index) =>
        h(index === 0 ? "th" : "td", { "data-label": column.label, scope: index === 0 ? "row" : null }, column.cell(row))))));
      note.textContent = rows.length && !shown.length ? "Nothing matches this search." : empty;
      note.hidden = shown.length > 0;
    };
    draw();
    const wrapper = h("div", { class: "table-wrap" },
      h("table", { class: "admin-table" }, h("caption", { class: "sr-only" }, caption),
        h("thead", {}, h("tr", {}, columns.map((column) => h("th", { scope: "col" }, column.label)))), body), note);
    return { node: wrapper, draw };
  }
  function search(label, onInput) {
    const id = "search";
    return h("div", { class: "search-field admin-search" }, icon("search"), h("label", { class: "sr-only", for: id }, label),
      h("input", { id, type: "search", placeholder: `${label}…`, autocomplete: "off", maxlength: "200", oninput: (event) => onInput(event.target.value.trim()) }),
      h("kbd", { "aria-hidden": "true" }, "/"));
  }
  function listView(parts) {
    const { eyebrow, title, description, action, caption, columns, rows, empty, filter, before } = parts;
    const grid = table(caption, columns, rows, empty, filter);
    return h("div", {}, heading(eyebrow, title, description, action), before || null,
      h("div", { class: "toolbar admin-toolbar" }, search(`Search ${title.toLowerCase()}`, grid.draw), h("span", { class: "result-count" }, `${rows.length} total`)),
      grid.node);
  }
  function field(label, input, hint) {
    const hintId = hint ? `${input.id}-hint` : null;
    if (hintId) input.setAttribute("aria-describedby", hintId);
    return h("div", { class: "field" }, h("label", { for: input.id }, label), input, hint ? h("p", { class: "field-hint", id: hintId }, hint) : null);
  }
  function check(id, label, checked, hint) {
    const input = h("input", { id, type: "checkbox", checked, "aria-describedby": hint ? `${id}-hint` : null });
    return h("div", { class: "check-field" }, h("label", { class: "checkbox", for: id }, input, label), hint ? h("p", { class: "field-hint", id: `${id}-hint` }, hint) : null);
  }
  function card(title, ...children) {
    return h("section", { class: "admin-card", "aria-label": title }, h("h2", {}, title), children);
  }
  function choose(id, names, current) {
    const select = h("select", { id }, h("option", { value: "" }, "Default"), names.map((name) => h("option", { value: name }, name)));
    select.value = names.includes(current) ? current : "";
    return select;
  }
  const actions = (...buttons) => h("div", { class: "form-actions" }, buttons);
  function missing(section, noun) {
    return { crumb: "Not found", node: h("div", {}, heading(SECTIONS[section].toUpperCase(), `${noun} not found`, `It doesn't exist or was removed. The list shows what exists now.`, link(hash(section), `Back to ${SECTIONS[section].toLowerCase()}`, { class: "button secondary" }))) };
  }
  const value = (form, id) => form.querySelector(`#${id}`).value.trim();
  const checked = (form, id) => form.querySelector(`#${id}`).checked;
  const userById = (id) => data.users.find((user) => user.id === id);
  const personName = (user) => user ? user.display_name || user.username : "Unknown person";
  const actorName = (id) => {
    if (id.startsWith("agent:")) return `Agent ${id.slice(6)}`;
    const user = userById(id); return user ? user.username : id;
  };

  // Every save reloads the configuration and its revision.
  async function saved(message, target) {
    await refresh({ focus: !target });
    if (target && location.hash !== target) location.hash = target;
    toast(message);
  }

  // ---- Confirmation and secret dialogs -----------------------------------------------------
  // `keyed: false` is for calls the server does not keep idempotency receipts for (access
  // decisions): they get no key, and their overrides explain a lost response instead.
  function confirmAction({ title, text, ok, danger = false, input = null, keyed = true, run, overrides }) {
    const dialog = $("confirm-dialog");
    confirmOpener = document.activeElement;
    $("confirm-title").textContent = title; $("confirm-text").textContent = text;
    $("confirm-ok").textContent = ok; $("confirm-ok").classList.toggle("danger", danger);
    $("confirm-field").hidden = !input; $("confirm-input").value = "";
    if (input) { $("confirm-label").textContent = input.label; $("confirm-input").type = input.type || "text"; }
    $("confirm-error").hidden = true;
    let key = null;
    confirmRun = async () => {
      const entered = $("confirm-input").value;
      if (input && !entered) { $("confirm-error").textContent = `Enter ${input.label.toLowerCase()}.`; $("confirm-error").hidden = false; $("confirm-input").focus(); return; }
      if (keyed) key = key || requestKey();
      $("confirm-ok").disabled = true; $("confirm-cancel").disabled = true; $("confirm-form").setAttribute("aria-busy", "true");
      try {
        await run(entered, key);
        confirmRun = null; dialog.close();
      } catch (error) {
        if (error.status === 401) { confirmRun = null; dialog.close(); gate("signin"); return; }
        showError($("confirm-error"), error, overrides);
      } finally {
        $("confirm-ok").disabled = false; $("confirm-cancel").disabled = false; $("confirm-form").removeAttribute("aria-busy");
      }
    };
    $("confirm-input").oninput = () => { key = null; };
    dialog.showModal();
    (input ? $("confirm-input") : $("confirm-cancel")).focus();
  }
  $("confirm-form").addEventListener("submit", (event) => { event.preventDefault(); if (confirmRun && !$("confirm-ok").disabled) confirmRun(); });
  $("confirm-cancel").addEventListener("click", () => $("confirm-dialog").close());
  $("confirm-dialog").addEventListener("cancel", (event) => { if ($("confirm-ok").disabled) event.preventDefault(); });
  $("confirm-dialog").addEventListener("close", () => {
    $("confirm-input").value = ""; confirmRun = null;
    const target = confirmOpener && confirmOpener.isConnected ? confirmOpener : document.querySelector("#view h1");
    if (target) target.focus();
  });
  function showSecret(clientId, secret) {
    $("secret-client").textContent = clientId; $("secret-value").value = secret;
    $("secret-dialog").showModal(); $("secret-value").focus(); $("secret-value").select();
  }
  $("secret-copy").addEventListener("click", async () => {
    try { await navigator.clipboard.writeText($("secret-value").value); announce("Secret copied"); $("secret-copy").lastChild.textContent = "Copied"; }
    catch { $("secret-value").select(); announce("Select the secret and copy it"); }
  });
  $("secret-close").addEventListener("click", () => $("secret-dialog").close());
  $("secret-dialog").addEventListener("close", () => {
    $("secret-value").value = ""; $("secret-copy").lastChild.textContent = "Copy";
    const heading = document.querySelector("#view h1"); if (heading) heading.focus();
  });

  // ---- Applications ------------------------------------------------------------------------
  // Mirrors validate_client: services use API scopes only, without groups or MFA.
  const IDENTITY_SCOPES = ["openid", "profile", "email", "groups", "offline_access", "bound_key"];
  const SERVICE_HINT = "API scopes this service may request, separated by spaces, for example: api.read api.write. Identity scopes such as openid, profile and email are not allowed.";
  const OIDC_HINT = "Separated by spaces. Must include openid, for example: openid profile email.";
  function checkScopes(scopes, service) {
    if (!scopes.length) throw invalid(service ? "Enter at least one API scope, for example api.read." : "Enter at least one scope, including openid.");
    if (service) {
      const identity = scopes.filter((scope) => IDENTITY_SCOPES.includes(scope));
      if (identity.length) throw invalid(`Services can't use identity scopes: ${identity.join(", ")}. Use API scopes such as api.read.`);
    } else if (!scopes.includes("openid")) throw invalid("Sign-in applications must include the openid scope.");
  }
  function kind(client) {
    if (client.settings.saml) return "SAML";
    if (client.settings.proxy) return "Proxy";
    if (client.service) return "Service";
    return client.confidential ? "Confidential" : "Public";
  }
  function access(client) {
    const groups = sorted(client.allowed_groups);
    return groups.length ? groups.join(", ") : "Everyone";
  }
  function applications() {
    const rows = [...data.clients].sort((a, b) => byName(a.name, b.name));
    return {
      node: listView({
        eyebrow: "APPLICATIONS", title: "Applications",
        description: "Sign-in connections for your apps and services. Groups and MFA set here decide who can sign in.",
        action: link(hash("applications", "new"), "New application", { class: "button primary" }),
        caption: "Applications", rows, filter: (c) => `${c.name} ${c.client_id}`,
        empty: "No applications yet. Create one to connect an app to riAuth.",
        columns: [
          { label: "Application", cell: (c) => h("span", { class: "cell-title" }, link(hash("applications", c.client_id), c.name), h("small", {}, c.client_id)) },
          { label: "Type", cell: kind },
          { label: "Access", cell: access },
          { label: "Sign-in", cell: (c) => c.require_mfa ? badge("MFA required", "info") : "Password or passkey" },
          { label: "Status", cell: (c) => c.enabled ? badge("Enabled", "ok") : badge("Disabled", "muted") },
        ],
      }),
    };
  }
  function groupChecks(prefix, selected) {
    const names = data.groups.map((g) => g.name).sort(byName);
    return h("fieldset", { class: "group-checks" }, h("legend", {}, "Who can sign in"),
      h("p", { class: "field-hint" }, "No group selected means any enabled person can sign in, subject to the application's other policies."),
      names.length ? names.map((name, index) => check(`${prefix}-group-${index}`, name, selected.includes(name))) : h("p", { class: "field-hint" }, "There are no groups yet."));
  }
  const selectedGroups = (form, prefix) => data.groups.map((g) => g.name).sort(byName).filter((_, index) => { const box = form.querySelector(`#${prefix}-group-${index}`); return box && box.checked; });
  function application(id) {
    const client = data.clients.find((c) => c.client_id === id);
    if (!client) return missing("applications", "Application");
    const app = client.settings.app || {};
    const oidcApp = !client.service && !client.settings.saml && !client.settings.proxy;
    const native = () => Boolean(client.settings.native);
    const scopeInput = h("input", { id: "app-scopes", spellcheck: "false", autocomplete: "off", value: sorted(client.scopes).join(" ") });
    const redirects = h("textarea", { id: "app-redirects", rows: "3", spellcheck: "false", value: client.redirect_uris.join("\n") });
    const origins = h("textarea", { id: "app-origins", rows: "2", spellcheck: "false", placeholder: "https://app.example.com", value: sorted(client.settings.origins).join("\n") });
    const logout = h("textarea", { id: "app-logout", rows: "2", spellcheck: "false", value: client.settings.post_logout_redirect_uris.join("\n") });
    const mappingRows = client.settings.claim_mappings.map(fromMapping);
    const editor = oidcApp ? mappingEditor("app", mappingRows, () => sorted(words(scopeInput.value))) : null;
    scopeInput.addEventListener("change", () => { if (editor) editor.draw(); });
    const uriField = (label, input, hint, kind) => h("div", { class: "field" }, h("label", { for: input.id }, label), input, h("p", { class: "field-hint" }, hint), uriFeedback(input, native, kind).node);
    const form = h("form", { class: "admin-form", novalidate: true },
      card("Details",
        field("Name", h("input", { id: "app-name", maxlength: "200", required: true, value: client.name })),
        check("app-enabled", "Enabled", client.enabled, "Disabling signs this application out: its existing grants are revoked and are not restored by enabling it again."),
        client.service ? null : check("app-mfa", "Require a passkey or authenticator code", client.require_mfa)),
      client.service
        ? card("API access", field("API scopes", scopeInput, SERVICE_HINT))
        : card("Access", groupChecks("app", sorted(client.allowed_groups))),
      client.service ? null : card("Sign-in",
        uriField("Redirect URIs", redirects, "One per line. riAuth sends people back only to these exact addresses.", "redirect"),
        field("Scopes", scopeInput, OIDC_HINT),
        oidcApp ? uriField("After sign-out, return to", logout, "Optional. Addresses the app may send people to after signing out.", "logout") : null,
        oidcApp ? uriField("Allowed origins", origins, "Browser origins that may call the token and userinfo endpoints. Single-page apps need their own origin here.", "origin") : null),
      oidcApp ? card("Claims", deliveryChecks("app", client.settings), h("h3", {}, "Custom claims"), editor.node) : null,
      card("Your applications page",
        field("Description", h("input", { id: "app-description", maxlength: "300", value: app.description || "" })),
        field("Category", h("input", { id: "app-category", maxlength: "60", value: app.category || "" }), "Empty shows the app under Workspace."),
        field("Launch URL", h("input", { id: "app-launch", type: "url", spellcheck: "false", value: app.launch_url || "" }), "The app's home or login page, not its callback. Without one, the app shows as Setup pending."),
        h("div", { class: "field-row" },
          field("Icon", choose("app-icon", ICONS, app.icon)),
          field("Accent", choose("app-accent", ACCENTS, app.accent))),
        check("app-hidden", "Hide from Your applications", Boolean(app.hidden), "Access rules still apply to sign-in; this only affects the catalogue.")),
      actions(h("button", { class: "button primary", type: "submit" }, "Save changes"), link(hash("applications"), "Cancel", { class: "button secondary" })));
    bindForm(form, async (key) => {
      const patch = {};
      const name = value(form, "app-name");
      if (!name) throw invalid("Enter a name.");
      if (name !== client.name) patch.name = name;
      if (checked(form, "app-enabled") !== client.enabled) patch.enabled = checked(form, "app-enabled");
      if (!client.service) {
        if (checked(form, "app-mfa") !== client.require_mfa) patch.require_mfa = checked(form, "app-mfa");
        const groups = selectedGroups(form, "app");
        if (!same(groups, sorted(client.allowed_groups).sort(byName))) patch.allowed_groups = groups;
        const uris = lines(redirects.value);
        if (!same(uris, client.redirect_uris)) patch.redirect_uris = uris;
      }
      const scopes = sorted(words(scopeInput.value));
      if (!same(scopes, sorted(client.scopes))) { checkScopes(scopes, client.service); patch.scopes = scopes; }
      // A settings update replaces the whole object, so the current settings travel with it.
      const settings = structuredClone(client.settings);
      if (oidcApp) {
        checkMappings(mappingRows, scopes);
        Object.assign(settings, readDelivery(form, "app"), {
          origins: sorted(lines(origins.value)),
          post_logout_redirect_uris: lines(logout.value),
          claim_mappings: mappingRows.map(toMapping),
        });
      }
      const next = { ...app, description: value(form, "app-description"), category: value(form, "app-category"), launch_url: value(form, "app-launch") || null, icon: value(form, "app-icon"), accent: value(form, "app-accent"), hidden: checked(form, "app-hidden") };
      const before = { description: "", category: "", launch_url: null, icon: "", accent: "", hidden: false, launch_scopes: [], ...app };
      if (!same({ ...before, ...next }, before)) settings.app = { ...before, ...next };
      if (!same(settings, client.settings)) patch.settings = settings;
      if (!Object.keys(patch).length) throw invalid("There are no changes to save.");
      await api("PATCH", `admin/clients/${seg(client.client_id)}`, patch, { revision: data.revision, key });
      await saved(`Saved ${name}.`);
    });
    const secret = client.settings.token_endpoint_auth_method === "private_key_jwt"
      ? h("p", { class: "field-hint" }, "Authenticates with a private key JWT. There is no shared secret to rotate.")
      : client.confidential
        ? h("div", {}, h("p", { class: "field-hint" }, "Rotating issues a new secret, stops the old one immediately and revokes this application's grants."),
          h("button", { class: "button secondary", type: "button", onclick: () => confirmAction({
            title: `Rotate the secret for ${client.name}?`, ok: "Rotate secret", danger: true,
            text: "The current secret stops working now and existing grants are revoked. Update the application with the new secret straight away.",
            run: async (_, key) => {
              const result = await api("POST", `admin/clients/${seg(client.client_id)}/rotate-secret`, undefined, { key });
              showSecret(client.client_id, result.client_secret);
              refresh();
            },
          }) }, "Rotate secret"))
        : h("p", { class: "field-hint" }, "Public client: it signs in with PKCE and has no secret.");
    const { app: _, ...protocol } = client.settings;
    const node = h("div", {},
      heading("APPLICATION", client.name, null),
      h("p", { class: "badges" }, h("code", {}, client.client_id), badge(kind(client)), client.enabled ? badge("Enabled", "ok") : badge("Disabled", "muted"), client.require_mfa ? badge("MFA required", "info") : null,
        !client.service && !app.launch_url && !client.settings.proxy ? badge("Setup pending", "warn") : null),
      h("div", { class: "detail-grid" }, form,
        h("div", { class: "detail-side" },
          diagnosticsCard(client, true),
          signInTest(client),
          card("Credentials", secret),
          card("Protocol settings", h("p", { class: "field-hint" }, "Signing, encryption, token lifetimes and trust are changed with riauth client update or a desired-state manifest; saving this page keeps them."),
            h("details", {}, h("summary", {}, "Show current settings"), h("pre", { class: "settings-json" }, JSON.stringify(protocol, null, 2)))))));
    return { crumb: client.name, node };
  }

  // ---- Application setup: shared pieces ----------------------------------------------------
  const PROTECTED_CLAIMS = ["iss", "sub", "aud", "exp", "iat", "nbf", "jti", "nonce", "auth_time", "amr", "acr", "at_hash", "c_hash", "sid", "client_id", "scope", "cnf", "act", "email", "email_verified"];
  const SOURCES = [["username", "Username"], ["display_name", "Display name"], ["email", "Email address"], ["email_verified", "Email verified"], ["groups", "Group names"], ["attribute", "Person attribute"], ["literal", "Fixed value"]];
  const REASONS = {
    required_groups_missing: "isn't in every group the application's policy requires",
    no_matching_group: "isn't in any group the application's policy accepts",
    denied_group: "is in a group the application's policy denies",
    user_not_allowed: "isn't on the application's list of allowed people",
    user_denied: "is on the application's list of denied people",
    mfa_required: "needs a passkey or authenticator code for this application",
    client_disabled: "can't sign in because the application is disabled",
    user_disabled: "can't sign in because their account is disabled",
    service_client_has_no_user_identity: "can't sign in: services have no people signing in",
    unregistered_scope: "asked for a scope the application doesn't allow",
    no_matching_client_group: "isn't a member of the application's allowed groups",
  };
  // What a claim mapping row shows; `toMapping` turns it into the stored ClaimMapping.
  function fromMapping(m) {
    const type = m.source.type;
    // A text that would read back as JSON (42, true) is shown quoted, so saving keeps its type.
    const literal = (v) => { if (typeof v !== "string") return JSON.stringify(v); try { JSON.parse(v); return JSON.stringify(v); } catch { return v; } };
    const value = type === "attribute" ? m.source.key : type === "literal" ? literal(m.source.value) : "";
    return { claim: m.claim, scope: m.scope, type, value };
  }
  function toMapping(row) {
    const source = { type: row.type };
    if (row.type === "attribute") source.key = row.value.trim();
    if (row.type === "literal") { try { source.value = JSON.parse(row.value); } catch { source.value = row.value; } }
    return { scope: row.scope, claim: row.claim.trim(), source };
  }
  // Names the row at fault before riAuth's own check, which reports only that one is invalid.
  function checkMappings(rows, scopes) {
    const seen = new Set();
    rows.forEach((row, index) => {
      const claim = row.claim.trim(), where = `Custom claim ${index + 1}`;
      if (!claim) throw invalid(`${where}: enter a claim name.`);
      if (PROTECTED_CLAIMS.includes(claim)) throw invalid(`${where}: riAuth sets ${claim} itself, so it can't be mapped.`);
      if (seen.has(claim)) throw invalid(`${where}: ${claim} is mapped twice.`);
      seen.add(claim);
      if (!scopes.includes(row.scope)) throw invalid(`${where}: choose a scope this application allows.`);
      if (row.type === "attribute" && !row.value.trim()) throw invalid(`${where}: enter the attribute name.`);
    });
  }
  // Rows are edited in place; `scopes()` gives the scopes a row may be sent with.
  function mappingEditor(prefix, rows, scopes) {
    const list = h("div", { class: "mapping-list" });
    const draw = () => {
      const allowed = scopes();
      list.replaceChildren(...rows.map((row, index) => {
        const id = `${prefix}-map-${index}`;
        const valueInput = h("input", { id: `${id}-value`, spellcheck: "false", autocomplete: "off", maxlength: "512", value: row.value, placeholder: row.type === "literal" ? "\"staff\" or 42" : "department", oninput: (e) => { row.value = e.target.value; } });
        const valueField = h("div", { class: "field", hidden: !["attribute", "literal"].includes(row.type) }, h("label", { for: valueInput.id }, row.type === "literal" ? "Value" : "Attribute"), valueInput);
        const source = h("select", { id: `${id}-source`, onchange: (e) => { row.type = e.target.value; draw(); document.getElementById(`${id}-source`)?.focus(); } }, SOURCES.map(([v, text]) => h("option", { value: v, selected: v === row.type }, text)));
        const scope = h("select", { id: `${id}-scope`, onchange: (e) => { row.scope = e.target.value; } }, allowed.map((s) => h("option", { value: s, selected: s === row.scope }, s)));
        if (!allowed.includes(row.scope)) { row.scope = allowed.includes("profile") ? "profile" : allowed[0] || ""; scope.value = row.scope; }
        return h("fieldset", { class: "mapping-row" }, h("legend", { class: "sr-only" }, `Custom claim ${index + 1}`),
          h("div", { class: "field" }, h("label", { for: `${id}-claim` }, "Claim"), h("input", { id: `${id}-claim`, spellcheck: "false", autocomplete: "off", autocapitalize: "none", maxlength: "128", value: row.claim, placeholder: "department", oninput: (e) => { row.claim = e.target.value; } })),
          h("div", { class: "field" }, h("label", { for: source.id }, "From"), source),
          valueField,
          h("div", { class: "field" }, h("label", { for: scope.id }, "Sent with scope"), scope),
          h("button", { class: "text-button", type: "button", "aria-label": `Remove custom claim ${index + 1}`, onclick: () => { rows.splice(index, 1); draw(); addButton.focus(); } }, "Remove"));
      }));
      if (!rows.length) list.append(h("p", { class: "field-hint" }, "No custom claims. Standard claims come from the scopes above."));
    };
    const addButton = h("button", { class: "button secondary small", type: "button", onclick: () => {
      rows.push({ claim: "", type: "attribute", value: "", scope: scopes().includes("profile") ? "profile" : scopes()[0] || "openid" });
      draw(); document.getElementById(`${prefix}-map-${rows.length - 1}-claim`).focus();
    } }, icon("plus"), "Add a claim");
    draw();
    return { node: h("div", {}, list, addButton), draw };
  }
  function deliveryChecks(prefix, settings) {
    return h("fieldset", {}, h("legend", {}, "Where claims are sent"),
      check(`${prefix}-groups-profile`, "Send groups with the profile scope", settings.groups_in_profile, "For apps that can't request the groups scope."),
      check(`${prefix}-access-claims`, "Add identity claims to access tokens", settings.claims_in_access_token, "For APIs that read the person from the access token instead of calling userinfo."),
      check(`${prefix}-userinfo-only`, "Send identity claims only from userinfo", settings.userinfo_only, "Keeps ID tokens small: they carry only sub."));
  }
  const readDelivery = (form, prefix) => ({
    groups_in_profile: checked(form, `${prefix}-groups-profile`),
    claims_in_access_token: checked(form, `${prefix}-access-claims`),
    userinfo_only: checked(form, `${prefix}-userinfo-only`),
  });
  // Mirrors validate_client so each line's problem is named; riAuth still decides.
  function uriProblem(uri, native, kind = "redirect") {
    let url;
    try { url = new URL(uri); } catch { return "isn't a complete URL"; }
    const loopback = ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname);
    const privateScheme = kind === "redirect" && native && !["http:", "https:", "file:", "data:", "javascript:"].includes(url.protocol) && url.protocol.slice(0, -1).includes(".");
    if (uri.includes("*")) return "can't contain wildcards";
    if (url.hash || uri.includes("#")) return "can't have a fragment";
    if (url.username || url.password) return "can't contain credentials";
    if (privateScheme) return null;
    if (url.protocol === "http:" && !loopback) return "must use HTTPS (plain HTTP only on localhost)";
    if (url.protocol !== "https:" && url.protocol !== "http:") return native ? "needs HTTPS, loopback HTTP, or a reverse-domain scheme like com.example.app:" : "must use HTTPS";
    if (kind === "origin" && (url.pathname !== "/" || url.search || !uri.match(/^[a-z]+:\/\/[^/]+$/i))) return `must be an origin only, like ${url.origin}`;
    if (kind === "redirect" && ["code", "state", "iss", "error", "error_description", "response", "session_state"].some((key) => url.searchParams.has(key))) return "can't contain code, state or other response parameters";
    return null;
  }
  function uriFeedback(textarea, native, kind) {
    const list = h("ul", { class: "uri-feedback", "aria-live": "polite" });
    const draw = () => {
      list.replaceChildren(...lines(textarea.value).map((uri) => {
        const problem = uriProblem(uri, native(), kind);
        return h("li", { class: problem ? "bad" : "good" }, h("code", {}, uri), " ", problem || (kind === "origin" ? "valid origin" : "valid"));
      }));
    };
    textarea.addEventListener("input", draw); draw();
    return { node: list, draw };
  }
  const originsOf = (uris) => [...new Set(uris.map((uri) => { try { const u = new URL(uri); return ["http:", "https:"].includes(u.protocol) ? u.origin : null; } catch { return null; } }).filter(Boolean))];
  function copyButton(text, label) {
    const button = h("button", { class: "icon-button", type: "button", "aria-label": `Copy ${label}`, title: "Copy", onclick: async () => {
      try { await navigator.clipboard.writeText(text); announce(`${label} copied`); } catch { announce("Select the value and copy it"); }
    } }, icon("copy"));
    return button;
  }
  const LEVELS = { error: ["Needs attention", "bad"], warn: ["Check", "warn"], info: ["Note", "info"], ok: ["OK", "ok"] };
  function checkList(checks) {
    const order = { error: 0, warn: 1, info: 2, ok: 3 };
    const sortedChecks = [...checks].sort((a, b) => order[a.level] - order[b.level]);
    return h("ul", { class: "check-list" }, sortedChecks.map((c) => h("li", { class: `check ${LEVELS[c.level][1]}` },
      h("span", { class: "check-level" }, LEVELS[c.level][0]),
      h("div", {}, h("strong", {}, c.title), h("p", {}, c.detail), c.last_issued_at ? h("p", {}, "Last token issued ", when(c.last_issued_at), ".") : null))));
  }
  function claimsPreview(claims) {
    const scopes = Object.keys(claims.by_scope);
    const places = [claims.id_token ? "ID token" : null, "userinfo", claims.access_token ? "access token" : null].filter(Boolean).join(", ");
    return h("div", {},
      scopes.length ? h("dl", { class: "facts claims" }, scopes.map((scope) => [h("dt", {}, h("code", {}, scope)), h("dd", {}, claims.by_scope[scope].map((c, i) => [i ? ", " : "", h("code", { title: `from ${c.source}` }, c.claim)]))])) : h("p", { class: "field-hint" }, "No identity claims."),
      h("p", { class: "field-hint" }, `Delivered in: ${places}.`));
  }
  function connectionFacts(connection, secret) {
    const row = (label, text, copyable = true) => [h("dt", {}, label), h("dd", {}, h("code", {}, text), copyable ? copyButton(text, label) : null)];
    const methods = connection.token_endpoint_auth_methods;
    return h("dl", { class: "facts connection-facts" },
      row("Issuer", connection.issuer), row("Discovery", connection.discovery_url), row("Client ID", connection.client_id),
      secret ? row("Client secret", secret) : null,
      [h("dt", {}, "Authentication"), h("dd", {}, methods.map((m, i) => [i ? " or " : "", h("code", {}, m)]), connection.pkce ? " · PKCE S256" : "")],
      connection.client_assertion_audience ? row("Assertion audience", connection.client_assertion_audience) : null,
      row("Authorize", connection.authorization_endpoint), row("Token", connection.token_endpoint),
      connection.pkce ? row("Userinfo", connection.userinfo_endpoint) : null, row("JWKS", connection.jwks_uri),
      connection.pkce ? row("Sign-out", connection.end_session_endpoint) : null);
  }
  function envSnippet(connection, secret) {
    const text = [`OIDC_ISSUER=${connection.issuer}`, `OIDC_CLIENT_ID=${connection.client_id}`, secret ? `OIDC_CLIENT_SECRET=${secret}` : null,
      connection.redirect_uris.length ? `OIDC_REDIRECT_URI=${connection.redirect_uris[0]}` : null, `OIDC_SCOPES="${connection.scopes.join(" ")}"`].filter(Boolean).join("\n");
    return h("div", { class: "snippet" }, h("div", { class: "section-heading" }, h("h3", {}, "Environment for the app"), copyButton(text, "environment settings")), h("pre", { class: "settings-json" }, text));
  }
  // riAuth's findings plus one only the browser can make: whether the issuer's discovery
  // document is served here. connect-src 'self' limits that to this origin.
  async function diagnose(clientId) {
    const report = await api("GET", `admin/clients/${seg(clientId)}/diagnostics`);
    const url = new URL(report.connection.discovery_url);
    if (url.origin === location.origin) {
      try {
        const response = await fetch(url.href, { credentials: "omit", cache: "no-store", mode: "same-origin" });
        const doc = response.ok ? await response.json() : null;
        report.checks.push(doc && doc.issuer === report.connection.issuer
          ? { level: "ok", title: "Discovery document reachable", detail: `${url.href} answers with this issuer from your browser.` }
          : { level: "error", title: "Discovery document not served", detail: `${url.href} ${response.ok ? "names another issuer" : `answered ${response.status}`}. Apps configured with this issuer can't start sign-in.` });
      } catch {
        report.checks.push({ level: "error", title: "Discovery document unreachable", detail: `${url.href} didn't answer from your browser.` });
      }
    } else {
      report.checks.push({ level: "info", title: "Discovery checked from riAuth only", detail: `The issuer is on another origin. Open ${url.href} from the app's network to confirm it is reachable.` });
    }
    return report;
  }
  // `withConnection` adds the issuer, endpoints and client settings the app is configured with.
  function diagnosticsCard(client, withConnection = false) {
    const connection = h("div", {});
    const body = h("div", { "aria-live": "polite" }, h("p", { class: "field-hint" }, "Checking…"));
    const status = h("p", { class: "form-error", role: "alert", tabindex: "-1", hidden: true });
    const run = async (button) => {
      if (button) { button.disabled = true; button.setAttribute("aria-busy", "true"); }
      status.hidden = true;
      try {
        const report = await diagnose(client.client_id);
        if (withConnection) connection.replaceChildren(connectionFacts(report.connection), envSnippet(report.connection), h("h3", {}, "Checks"));
        body.replaceChildren(checkList(report.checks), h("p", { class: "field-hint checked-at" }, "Checked ", when(report.checked_at), "."));
      } catch (error) {
        if (error.status === 401) { gate("signin"); return; }
        body.replaceChildren(); showError(status, error);
      } finally { if (button) { button.disabled = false; button.removeAttribute("aria-busy"); } }
    };
    const again = h("button", { class: "button secondary small", type: "button", onclick: () => run(again) }, icon("refresh"), "Check again");
    run();
    return card(withConnection ? "Connection" : "Diagnostics", h("p", { class: "field-hint" }, "Configuration and connection checks. Nothing is sent to the application."), connection, status, body, again);
  }
  // The policy engine's simulation (POST /policy/explain): no token is issued.
  function signInTest(client) {
    const users = [...data.users].sort((a, b) => byName(personName(a), personName(b)));
    const form = h("form", { class: "admin-form sign-in-test", novalidate: true },
      field("Person", h("select", { id: "test-user" }, users.map((u) => h("option", { value: u.username }, `${personName(u)} (${u.username})`)))),
      check("test-mfa", "Assume they used a passkey or authenticator code", client.require_mfa),
      actions(h("button", { class: "button secondary", type: "submit" }, "Test sign-in")));
    const result = h("div", { "aria-live": "polite" });
    bindForm(form, async () => {
      const username = value(form, "test-user");
      const report = await api("POST", `admin/clients/${seg(client.client_id)}/explain`, { username, scope: client.scopes, mfa: checked(form, "test-mfa") });
      const reasons = [...report.reasons, ...Object.values(report.scope_decisions).flat()];
      result.replaceChildren(
        h("p", { class: `test-result ${report.allowed ? "ok" : "bad"}` }, report.allowed ? `${username} can sign in.` : `${username} is refused:`),
        reasons.length ? h("ul", { class: "plain-list" }, [...new Set(reasons)].map((r) => h("li", {}, `${username} ${REASONS[r] || r.replaceAll("_", " ")}.`))) : null,
        report.allowed ? h("details", {}, h("summary", {}, "Claims the app would receive"), h("pre", { class: "settings-json" }, JSON.stringify(report.id_token_identity_claims, null, 2))) : null);
    }, { 403: "Testing needs permission to read both the application and the person." }, "Testing…");
    return client.service ? null : card("Test a sign-in", h("p", { class: "field-hint" }, "Simulates this application's policy for one person. No token is issued and nothing is recorded as a sign-in."), form, result);
  }

  // ---- Application setup wizard ------------------------------------------------------------
  // The draft outlives re-renders, so a refresh or a step change keeps what was entered, and
  // is dropped when the route leaves #/applications/new. Each Continue sends the whole draft
  // to /admin/client-checks: the create path's own authorization and validation, without a
  // write. Only Create writes, with one Idempotency-Key per attempt.
  const TYPES = {
    web: ["Web application", "Runs on a server and keeps a secret, like Next.js, Django, Rails or Spring."],
    spa: ["Single-page app", "Runs in the browser and can't keep a secret, like React, Vue or Angular. Signs in with PKCE."],
    native: ["Native or mobile app", "A desktop or phone app with a reverse-domain or loopback callback. Signs in with PKCE."],
    service: ["Service", "Machine-to-machine with client credentials. No people sign in."],
  };
  const STANDARD_SCOPES = [
    ["profile", "Profile", "name, preferred_username"],
    ["email", "Email", "email, email_verified"],
    ["groups", "Groups", "groups: the person's group names"],
    ["offline_access", "Stay signed in", "refresh tokens (offline_access)"],
  ];
  const AUTH = {
    client_secret_basic: ["Client secret in the Authorization header", "Recommended. riAuth generates the secret and shows it once."],
    client_secret_post: ["Client secret in the request body", "For libraries that can't send HTTP Basic authentication."],
    private_key_jwt: ["Private key JWT", "The app signs each token request with its own key. riAuth keeps only the public key, so there's no shared secret."],
  };
  const REDIRECT_EXAMPLES = { web: "https://app.example.com/auth/callback", spa: "https://app.example.com/callback", native: "com.example.app:/oauth/callback" };
  function freshDraft() {
    return { owner: data.me.user.id, step: 0, type: "web", name: "", id: "", idEdited: false, redirects: "", origins: "", logout: "", launch: "",
      groups: [], mfa: false, scopes: ["openid", "profile", "email"], extraScopes: "", apiScopes: "",
      delivery: { groups_in_profile: false, claims_in_access_token: false, userinfo_only: false }, mappings: [],
      auth: "client_secret_basic", jwks: "", checked: null, created: null };
  }
  const slug = (text) => text.toLowerCase().normalize("NFKD").replace(/[̀-ͯ]/g, "").replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 64);
  const confidentialType = (type) => type === "web" || type === "service";
  const draftScopes = (d) => d.type === "service" ? words(d.apiScopes) : [...new Set([...d.scopes, ...words(d.extraScopes)])];
  // validate_client requires at least one scope, but a service's scopes are collected on its
  // API access step. Until then the preflight checks this stand-in, so a step is checked only
  // on what it has collected. Create sends draftBody, which never contains it, and a stand-in
  // check is not kept for the review.
  const PENDING_SCOPE = "setup.pending";
  function preflightBody(d) {
    const body = draftBody(d);
    const pending = d.type === "service" && !wizardSteps(d).slice(0, d.step + 1).some(([key]) => key === "api");
    if (pending) body.scopes = [PENDING_SCOPE];
    return { body, pending };
  }
  function draftBody(d) {
    const service = d.type === "service";
    const settings = {};
    if (!service) {
      if (d.type === "native") settings.native = true;
      settings.origins = lines(d.origins);
      settings.post_logout_redirect_uris = lines(d.logout);
      if (d.launch.trim()) settings.app = { launch_url: d.launch.trim() };
      Object.assign(settings, d.delivery);
      settings.claim_mappings = d.mappings.map(toMapping);
    }
    if (confidentialType(d.type)) {
      settings.token_endpoint_auth_method = d.auth;
      if (d.auth === "private_key_jwt") settings.jwks = parseJwks(d.jwks);
    } else settings.token_endpoint_auth_method = "none";
    return {
      client_id: d.id.trim(), name: d.name.trim(), confidential: confidentialType(d.type), service,
      redirect_uris: service ? [] : lines(d.redirects), scopes: draftScopes(d),
      allowed_groups: service ? [] : d.groups, require_mfa: !service && d.mfa, settings,
    };
  }
  // Accepts a JWKS or one JWK. A private key is refused here so it never leaves the browser.
  function parseJwks(text) {
    let parsed;
    try { parsed = JSON.parse(text); } catch { throw invalid("Paste the app's public key as JSON: a JWKS ({\"keys\": [...]}) or a single JWK."); }
    const set = parsed && Array.isArray(parsed.keys) ? parsed : { keys: [parsed] };
    if (set.keys.some((key) => key && typeof key === "object" && ["d", "p", "q", "dp", "dq", "qi", "k"].some((member) => member in key))) {
      throw invalid("This is a private key. Paste only the public key; the private key stays with the app.");
    }
    return set;
  }
  function wizardSteps(d) {
    return d.type === "service"
      ? [["basics", "Application"], ["api", "API access"], ["credentials", "Credentials"], ["review", "Review"]]
      : [["basics", "Application"], ["urls", "Sign-in URLs"], ["access", "Access"], ["claims", "Claims"], ...(confidentialType(d.type) ? [["credentials", "Credentials"]] : []), ["review", "Review"]];
  }
  const STEP_VIEWS = {
    basics(d) {
      const typeOption = (key) => h("label", { class: "choice" }, h("input", { type: "radio", name: "wiz-type", value: key, checked: d.type === key }), h("span", {}, h("strong", {}, TYPES[key][0]), h("small", {}, TYPES[key][1])));
      const name = h("input", { id: "wiz-name", maxlength: "200", required: true, value: d.name, autocomplete: "off" });
      const id = h("input", { id: "wiz-id", maxlength: "64", required: true, spellcheck: "false", autocomplete: "off", autocapitalize: "none", value: d.id });
      name.addEventListener("input", () => { if (!d.idEdited) id.value = slug(name.value); });
      id.addEventListener("input", () => { d.idEdited = id.value.trim() !== "" && id.value !== slug(name.value); });
      return [card("Application",
        field("Name", name, "Shown to people when they sign in and on Your applications."),
        field("Client ID", id, "Used in protocol requests and can't be changed later. Letters, digits, dots, dashes and underscores."),
        h("fieldset", { class: "choices" }, h("legend", {}, "What are you connecting?"), Object.keys(TYPES).map(typeOption)))];
    },
    urls(d) {
      const native = () => d.type === "native";
      const redirects = h("textarea", { id: "wiz-redirects", rows: "3", spellcheck: "false", placeholder: REDIRECT_EXAMPLES[d.type], value: d.redirects });
      const redirectCheck = uriFeedback(redirects, native, "redirect");
      const origins = h("textarea", { id: "wiz-origins", rows: "2", spellcheck: "false", placeholder: "https://app.example.com", value: d.origins });
      const originCheck = uriFeedback(origins, native, "origin");
      const logout = h("textarea", { id: "wiz-logout", rows: "2", spellcheck: "false", placeholder: "https://app.example.com/signed-out", value: d.logout });
      const logoutCheck = uriFeedback(logout, () => false, "logout");
      const useOrigins = h("button", { class: "button secondary small", type: "button", onclick: () => {
        const next = [...new Set([...lines(origins.value), ...originsOf(lines(redirects.value))])];
        origins.value = next.join("\n"); originCheck.draw(); announce(next.length ? `Allowed origins: ${next.join(", ")}` : "Enter a redirect URI first");
      } }, "Use the callback origins");
      const redirectHint = { web: "One per line: the exact callback addresses of your server. HTTPS, or HTTP on localhost for development.", spa: "One per line: the exact page addresses the browser returns to.", native: "One per line: a reverse-domain scheme like com.example.app:/callback, or http://127.0.0.1/callback (any port is accepted)." }[d.type];
      const originHint = d.type === "spa" ? "Required for single-page apps: the token endpoint answers only these origins from a browser." : "Only needed if code in the browser calls riAuth's token or userinfo endpoints directly.";
      return [card("Callbacks",
          h("div", { class: "field" }, h("label", { for: "wiz-redirects" }, "Redirect URIs"), redirects, h("p", { class: "field-hint" }, redirectHint), redirectCheck.node),
          h("div", { class: "field" }, h("label", { for: "wiz-logout" }, "After sign-out, return to"), logout, h("p", { class: "field-hint" }, "Optional. Addresses the app may send people to after signing out."), logoutCheck.node)),
        card("Browser access",
          h("div", { class: "field" }, h("label", { for: "wiz-origins" }, "Allowed origins"), origins, h("p", { class: "field-hint" }, originHint), originCheck.node),
          native() ? null : useOrigins),
        card("Your applications page",
          field("Launch URL", h("input", { id: "wiz-launch", type: "url", spellcheck: "false", value: d.launch, placeholder: "https://app.example.com/" }), "Optional. The app's home or login page, not its callback. Without one, the app shows as Setup pending."))];
    },
    access(d) {
      const names = data.groups.map((g) => g.name).sort(byName);
      const enabled = (g) => g.members.filter((id) => { const u = userById(id); return u && u.enabled; }).length;
      return [card("Who can sign in",
          h("p", { class: "field-hint" }, "Choose groups to limit sign-in to their members. With no group selected, anyone with an enabled account can sign in."),
          names.length ? h("fieldset", { class: "group-checks" }, h("legend", { class: "sr-only" }, "Allowed groups"), names.map((name, index) => {
            const count = enabled(data.groups.find((g) => g.name === name));
            return check(`wiz-group-${index}`, `${name} · ${count} ${count === 1 ? "person" : "people"}`, d.groups.includes(name));
          })) : h("p", { class: "field-hint" }, "There are no groups yet. ", link(hash("groups"), "Create a group"), " to limit who can sign in; your draft is kept.")),
        card("Sign-in strength", check("wiz-mfa", "Require a passkey or authenticator code", d.mfa, "People without one are refused until they add it from Your applications."))];
    },
    claims(d) {
      const scopeChecks = STANDARD_SCOPES.map(([scope, label, hint]) => check(`wiz-scope-${scope}`, label, d.scopes.includes(scope), hint));
      const extra = h("input", { id: "wiz-extra", spellcheck: "false", autocomplete: "off", value: d.extraScopes, placeholder: "api.read" });
      // Read from this step's own inputs: the editor draws before the card is attached.
      const scopes = () => [...new Set(["openid", ...STANDARD_SCOPES.map(([s]) => s).filter((_, i) => scopeChecks[i].querySelector("input").checked), ...words(extra.value)])];
      const editor = mappingEditor("wiz", d.mappings, scopes);
      const node = card("Scopes and claims",
        h("p", { class: "field-hint" }, "The app asks for these scopes at sign-in; each adds claims about the person. openid is always included and adds sub."),
        h("fieldset", {}, h("legend", {}, "Standard scopes"), scopeChecks),
        field("Additional scopes", extra, "Optional, separated by spaces: API scopes the app may request for its own backends."),
        deliveryChecks("wiz", d.delivery),
        h("h3", {}, "Custom claims"), h("p", { class: "field-hint" }, "Add claims from a person's attributes or a fixed value. A claim is sent only when its scope is requested."),
        editor.node);
      node.addEventListener("change", (e) => { if (e.target.id.startsWith("wiz-scope-")) editor.draw(); });
      extra.addEventListener("change", () => editor.draw());
      return [node];
    },
    api(d) {
      return [card("API access", field("API scopes", h("input", { id: "wiz-api-scopes", spellcheck: "false", autocomplete: "off", value: d.apiScopes, placeholder: "api.read api.write" }), SERVICE_HINT))];
    },
    credentials(d) {
      const option = (key) => h("label", { class: "choice" }, h("input", { type: "radio", name: "wiz-auth", value: key, checked: d.auth === key }), h("span", {}, h("strong", {}, AUTH[key][0]), h("small", {}, AUTH[key][1])));
      const jwks = h("div", { class: "field", id: "wiz-jwks-field", hidden: d.auth !== "private_key_jwt" }, h("label", { for: "wiz-jwks" }, "Public key (JWKS)"),
        h("textarea", { id: "wiz-jwks", rows: "7", spellcheck: "false", value: d.jwks, placeholder: "{\"keys\": [{\"kty\": \"EC\", \"crv\": \"P-256\", \"kid\": \"app-2026\", \"alg\": \"ES256\", \"x\": \"…\", \"y\": \"…\"}]}" }),
        h("p", { class: "field-hint" }, "1–8 public signing keys, each with kid and alg (RS256, ES256 or EdDSA). Never paste the private key."));
      const node = card("How the app authenticates", h("fieldset", { class: "choices" }, h("legend", { class: "sr-only" }, "Token endpoint authentication"), Object.keys(AUTH).map(option)), jwks);
      node.addEventListener("change", (e) => { if (e.target.name === "wiz-auth") jwks.hidden = e.target.value !== "private_key_jwt"; });
      return [node];
    },
    review(d) {
      const body = draftBody(d);
      const service = d.type === "service";
      const row = (label, content) => [h("dt", {}, label), h("dd", {}, content)];
      const list = (items, empty) => items.length ? items.map((item, i) => [i ? h("br") : null, h("code", {}, item)]) : empty;
      const summary = card("Summary", h("dl", { class: "facts review-facts" },
        row("Name", body.name), row("Client ID", h("code", {}, body.client_id)), row("Type", TYPES[d.type][0]),
        service ? null : row("Redirect URIs", list(body.redirect_uris, "None")),
        service ? null : row("Allowed origins", list(body.settings.origins, "None")),
        service ? null : row("After sign-out", list(body.settings.post_logout_redirect_uris, "riAuth's signed-out page")),
        service ? null : row("Access", body.allowed_groups.length ? body.allowed_groups.join(", ") : "Anyone with an account"),
        service ? null : row("MFA", body.require_mfa ? "Required" : "Not required"),
        row("Scopes", list(body.scopes, "None")),
        row("Credentials", confidentialType(d.type) ? AUTH[d.auth][0] : "None: PKCE only")));
      const checked = d.checked;
      return [summary,
        checked ? card("What the app receives", claimsPreview(checked.claims)) : null,
        checked ? card("Checks", h("p", { class: "field-hint" }, "riAuth checked this configuration with the same rules as creating it."), checkList(checked.checks)) : null];
    },
  };
  // Reads the current step's inputs into the draft. Leaving a step backwards keeps partial
  // input; local problems are raised only when moving forward.
  function readStep(stepKey, form, d) {
    const q = (id) => form.querySelector(`#${id}`);
    if (stepKey === "basics") {
      d.name = q("wiz-name").value; d.id = q("wiz-id").value.trim();
      const type = form.querySelector("input[name=wiz-type]:checked").value;
      if (type !== d.type) {
        d.type = type;
        d.auth = "client_secret_basic";
        if (type === "service") d.groups = [];
      }
    } else if (stepKey === "urls") {
      d.redirects = q("wiz-redirects").value; d.origins = q("wiz-origins").value; d.logout = q("wiz-logout").value; d.launch = q("wiz-launch").value;
    } else if (stepKey === "access") {
      d.groups = selectedGroups(form, "wiz"); d.mfa = checked(form, "wiz-mfa");
    } else if (stepKey === "claims") {
      d.scopes = ["openid", ...STANDARD_SCOPES.map(([s]) => s).filter((s) => checked(form, `wiz-scope-${s}`))];
      d.extraScopes = q("wiz-extra").value; d.delivery = readDelivery(form, "wiz");
    } else if (stepKey === "api") {
      d.apiScopes = q("wiz-api-scopes").value;
    } else if (stepKey === "credentials") {
      d.auth = form.querySelector("input[name=wiz-auth]:checked").value; d.jwks = q("wiz-jwks").value;
    }
  }
  function localCheck(stepKey, d) {
    if (stepKey === "basics") {
      if (!d.name.trim()) throw invalid("Enter a name.");
      if (!d.id) throw invalid("Enter a client ID.");
    } else if (stepKey === "urls") {
      const redirects = lines(d.redirects);
      if (!redirects.length) throw invalid("Enter at least one redirect URI: riAuth only sends people back to registered callbacks.");
      for (const [kind, values] of [["redirect", redirects], ["origin", lines(d.origins)], ["logout", lines(d.logout)]]) {
        for (const uri of values) {
          const problem = uriProblem(uri, d.type === "native", kind);
          if (problem) throw invalid(`${uri} ${problem}.`);
        }
      }
    } else if (stepKey === "claims") {
      checkMappings(d.mappings, draftScopes(d));
    } else if (stepKey === "api") {
      checkScopes(words(d.apiScopes), true);
    } else if (stepKey === "credentials" && d.auth === "private_key_jwt") {
      if (!d.jwks.trim()) throw invalid("Paste the app's public key.");
      parseJwks(d.jwks);
    }
  }
  function wizardStepper(steps, d) {
    return h("ol", { class: "wizard-steps" }, steps.map(([key, title], index) => {
      const state = index === d.step ? "current" : index < d.step ? "done" : "todo";
      const label = h("span", { class: "step-label" }, h("span", { class: "step-number", "aria-hidden": "true" }, String(index + 1)), title);
      // Only completed steps are links: moving forward always goes through riAuth's check.
      const content = index < d.step
        ? h("button", { class: "step-link", type: "button", "data-step": String(index) }, label, h("span", { class: "sr-only" }, " (completed)"))
        : label;
      return h("li", { class: `wizard-step ${state}`, "aria-current": index === d.step ? "step" : null }, content);
    }));
  }
  function newApplication() {
    if (!draft || draft.owner !== data.me.user.id) draft = freshDraft();
    const d = draft;
    if (d.created) return created(d);
    const steps = wizardSteps(d);
    d.step = Math.min(d.step, steps.length - 1);
    const [stepKey, stepTitle] = steps[d.step];
    const last = stepKey === "review";
    const back = d.step > 0 ? h("button", { class: "button secondary", type: "button", onclick: () => { readStep(stepKey, form, d); go(d.step - 1); } }, "Back") : null;
    const form = h("form", { class: "admin-form wizard-form", novalidate: true, "aria-labelledby": "wizard-step-title" },
      h("h2", { class: "wizard-title", id: "wizard-step-title", tabindex: "-1" }, `Step ${d.step + 1} of ${steps.length}: ${stepTitle}`),
      STEP_VIEWS[stepKey](d),
      actions(h("button", { class: "button primary", type: "submit" }, last ? "Create application" : "Continue"), back,
        link(hash("applications"), "Cancel", { class: "button secondary", onclick: () => { draft = null; } })));
    const go = (step) => { d.step = step; render(); document.getElementById("wizard-step-title")?.focus(); };
    captureWizard = () => { if (form.isConnected) readStep(stepKey, form, d); };
    const stepper = wizardStepper(steps, d);
    stepper.addEventListener("click", (event) => {
      const target = event.target.closest("[data-step]");
      if (!target) return;
      readStep(stepKey, form, d); go(Number(target.dataset.step));
    });
    if (last) {
      bindForm(form, async (key) => {
        const body = draftBody(d);
        const result = await api("POST", "admin/clients", body, { key });
        d.created = { client: result.client, secret: result.client_secret || null, connection: d.checked ? d.checked.connection : null };
        await saved(`Created ${body.name}.`);
      }, { 409: (error) => /already exists/i.test(error.message) ? "An application with this client ID was created meanwhile. Go back and choose another client ID." : undefined }, "Creating…");
    } else {
      bindForm(form, async () => {
        readStep(stepKey, form, d);
        localCheck(stepKey, d);
        // Continue asks riAuth to check the draft as a create would, without a write, on the
        // fields collected so far.
        const preflight = preflightBody(d);
        try {
          const checked = await api("POST", "admin/client-checks", preflight.body);
          // A stand-in check says nothing about the final scopes, so the review never shows it.
          d.checked = preflight.pending ? null : checked;
        } catch (error) {
          if (error.status === 400) throw invalid(error.message);
          throw error;
        }
        go(d.step + 1);
      }, {
        409: (error) => /already exists/i.test(error.message) ? "An application with this client ID already exists. Choose another client ID." : undefined,
        403: "Your account isn't allowed to create this application.",
      }, "Checking…");
    }
    return { crumb: "New application", node: h("div", {}, heading("APPLICATIONS", "Set up an application", "Connect an app to riAuth step by step. Each step is checked by riAuth before you continue; nothing is created until you confirm."), stepper, form) };
  }
  // After Create: what the app needs, with the secret shown only until this page is left.
  function created(d) {
    const { client, secret } = d.created;
    const done = link(hash("applications", client.client_id), "Go to the application", { class: "button primary", onclick: () => { draft = null; } });
    const another = h("button", { class: "button secondary", type: "button", onclick: () => { draft = freshDraft(); render({ focus: true }); } }, "Set up another");
    const connection = d.created.connection;
    const node = h("div", {},
      heading("APPLICATIONS", `Connect ${client.name}`, "The application exists now. Configure the app with these values, then check the connection."),
      secret ? h("p", { class: "notice warn-notice", role: "note" }, "Copy the client secret now. riAuth stores only a hash, and this page erases it as soon as you switch to another tab or window. A lost secret can only be replaced by rotating.") : null,
      d.created.discarded ? h("p", { class: "notice warn-notice", role: "note" }, "The client secret was erased when this page lost focus. If it wasn't copied, rotate it on the application's page.") : null,
      h("div", { class: "detail-grid" },
        h("div", { class: "admin-form" },
          card("Connection", connection ? connectionFacts(connection, secret) : h("p", { class: "field-hint" }, "Reload to see the connection details."), connection ? envSnippet(connection, secret) : null)),
        h("div", { class: "detail-side" }, diagnosticsCard(client))),
      actions(done, another));
    return { crumb: "New application", node };
  }

  // ---- People ------------------------------------------------------------------------------
  // Invitations go through the account invitation API: an invited account stays disabled
  // until its owner accepts the emailed link, which sets a password and adds the groups.
  const invitationOf = (user) => data.invitations.find((entry) => entry.user.id === user.id);
  const INVITATION_BADGES = { pending: ["Invited", "info"], blocked: ["Needs a new invitation", "warn"], expired: ["Invitation expired", "warn"], inactive: ["No working invitation", "muted"] };
  const invitationBadge = (invitation) => badge(...INVITATION_BADGES[invitation.status]);
  const days = (seconds) => seconds % 86400 ? duration(seconds) : `${seconds / 86400} ${seconds === 86400 ? "day" : "days"}`;
  function linkState(invitation) {
    if (invitation.status === "pending") return ["Link expires ", when(invitation.expires_at)];
    if (invitation.status === "expired") return ["Link expired ", when(invitation.expires_at)];
    // Acceptance rechecks who sent it; the server doesn't say which check failed.
    if (invitation.status === "blocked") return "The link can't be accepted: whoever sent it can no longer invite this person, or a group it adds was removed. Send a new link to replace it.";
    return "The link was revoked, has expired, or the account changed after it was sent.";
  }
  // Queued and sent describe riAuth's outbox and the mail server, never the inbox.
  function deliveryState(invitation) {
    const delivery = invitation.delivery;
    // Only the current link's message is tracked; an ended link has none to report.
    if (!delivery) return invitation.status === "inactive" ? "—" : "No delivery record";
    if (delivery.status === "sent") return ["Accepted by the mail server ", when(delivery.delivered_at)];
    if (delivery.status === "stopped") return `Not delivered after ${delivery.attempts} ${delivery.attempts === 1 ? "attempt" : "attempts"}`;
    return delivery.attempts ? `Queued, retrying after ${delivery.attempts} ${delivery.attempts === 1 ? "attempt" : "attempts"}` : "Queued for delivery";
  }
  // The outbox changes delivery state after the page loads. While a shown message is queued,
  // re-read invitations and replace only the delivery text, so open forms keep their input.
  // Checks start every five seconds and slow to once a minute while a message keeps waiting.
  let inviteTimer = null, inviteChecks = 0;
  const inviteDelivery = (invitation) => h("span", { "data-delivery": invitation.user.id }, deliveryState(invitation));
  function watchInvitations(restart = false) {
    clearTimeout(inviteTimer);
    if (restart) inviteChecks = 0;
    if (!data.invitations.some((i) => i.delivery && i.delivery.status === "queued")) return;
    const run = generation;
    inviteTimer = setTimeout(async () => {
      if (run !== generation || !document.querySelector("#view [data-delivery]")) return;
      let latest;
      try { latest = await api("GET", "admin/invitations"); } catch { return; }
      if (run !== generation) return;
      // Only the same link's delivery is taken over; anything else waits for a refresh.
      let changed = latest.invitations.length !== data.invitations.length;
      for (const shown of data.invitations) {
        const next = latest.invitations.find((i) => i.user.id === shown.user.id);
        if (next && next.status === shown.status && next.expires_at === shown.expires_at) shown.delivery = next.delivery;
        else changed = true;
      }
      for (const node of document.querySelectorAll("#view [data-delivery]")) {
        const invitation = data.invitations.find((i) => i.user.id === node.dataset.delivery);
        if (invitation) node.replaceChildren(...[deliveryState(invitation)].flat());
      }
      if (changed) { connection("Newer changes available"); return; }
      inviteChecks += 1;
      watchInvitations();
    }, Math.min(5000 * 2 ** Math.floor(inviteChecks / 6), 60000));
  }
  function revokeInvitation(invitation) {
    const { username } = invitation.user;
    confirmAction({
      title: `Revoke the invitation for ${username}?`, ok: "Revoke invitation", danger: true,
      text: "The emailed link stops working now. The account stays disabled and keeps its username; you can send a new link from their page.",
      run: async (_, key) => {
        await api("DELETE", `admin/invitations/${seg(username)}`, undefined, { key });
        await saved(`Revoked the invitation for ${username}. Its link no longer works.`);
      },
    });
  }
  // Sending again is the invitation API's reissue: as the signed-in administrator, to the
  // reviewed address with the reviewed groups. The new link supersedes any earlier one, so a
  // resent pending link stops working as soon as the replacement is queued.
  function reissueForm(user, invitation) {
    if (!data.mail) return h("p", { class: "field-hint" }, "Email delivery isn't configured on this server, so a new invitation can't be sent. Set a password for them instead.");
    const resend = invitation.status === "pending";
    const names = data.groups.map((g) => g.name).sort(byName);
    const form = h("form", { class: "admin-form", novalidate: true, "aria-labelledby": "reissue-title" },
      h("h3", { id: "reissue-title" }, resend ? "Resend the invitation" : "Send a new invitation"),
      field("Send to", h("input", { id: "reissue-email", type: "email", maxlength: "320", spellcheck: "false", autocomplete: "off", value: user.email || "" })),
      h("fieldset", { class: "group-checks" }, h("legend", {}, "Adds groups"),
        names.length ? names.map((name, index) => check(`reissue-group-${index}`, name, invitation.groups.includes(name))) : h("p", { class: "field-hint" }, "There are no groups yet.")),
      h("p", { class: "field-hint" }, resend
        ? `Sent as you. The current link stops working as soon as the new one is queued; the new link works for ${days(data.lifetime)}.`
        : [`Sent as you. It replaces the earlier link and works for ${days(data.lifetime)}. `,
          invitation.status === "inactive" ? "The earlier link's groups are no longer known, so choose them again." : "Groups that no longer exist are left out."]),
      actions(h("button", { class: "button primary", type: "submit" }, resend ? "Resend invitation" : "Send new invitation")));
    bindForm(form, async (key) => {
      const email = value(form, "reissue-email");
      if (!email) throw invalid("Enter the email address the new invitation goes to.");
      const body = { username: user.username, email, display_name: user.display_name, groups: selectedGroups(form, "reissue") };
      await api("POST", "admin/invitations", body, { revision: data.revision, key });
      await saved(resend ? `Invitation resent: queued for ${email}. The earlier link no longer works.` : `New invitation queued for ${email}. The link works for ${days(data.lifetime)}.`);
    }, {
      404: (error) => /group/i.test(error.message) ? "A selected group no longer exists. Reload to see the current groups." : undefined,
      409: (error) => /already exists/i.test(error.message) ? "This account is no longer waiting for an invitation, so no new link was sent. Reload to see its current state." : undefined,
      503: (error) => error.code === "delivery_unavailable" ? "Email delivery isn't configured on this server, so no invitation was sent." : undefined,
    });
    return form;
  }
  function invitations() {
    if (!data.invitations.length) return null;
    const rows = [...data.invitations].sort((a, b) => byName(personName(a.user), personName(b.user)));
    const grid = table("Invitations waiting for acceptance", [
      { label: "Person", cell: (i) => h("span", { class: "cell-title" }, link(hash("people", i.user.username), personName(i.user)), h("small", {}, i.user.username)) },
      { label: "Email", cell: (i) => i.user.email || "—" },
      { label: "Status", cell: (i) => h("span", {}, invitationBadge(i), " ", linkState(i)) },
      { label: "Delivery", cell: inviteDelivery },
      { label: "Groups", cell: (i) => i.groups.length ? i.groups.join(", ") : "—" },
      { label: "Actions", cell: (i) => h("span", { class: "row-actions" },
        i.status === "pending"
          ? link(hash("people", i.user.username), "Resend", { class: "text-button", "aria-label": `Review and resend the invitation to ${i.user.username}` })
          : link(hash("people", i.user.username), "Send new link", { class: "text-button", "aria-label": `Review and send a new invitation to ${i.user.username}` }),
        i.status === "inactive" ? null : h("button", { class: "text-button", type: "button", "aria-label": `Revoke the invitation for ${i.user.username}`, onclick: () => revokeInvitation(i) }, "Revoke")) },
    ], rows, "No invitations are waiting.", (i) => i.user.username);
    return h("section", { class: "admin-section", "aria-labelledby": "invitations-title" },
      h("h2", { id: "invitations-title", tabindex: "-1" }, "Invitations"),
      h("p", { class: "field-hint" }, "Accepting sets their password, verifies the address, enables the account and adds the listed groups. Resend and Send new link review the address and groups first; the new link replaces the old one. Delivery updates here while a message is queued."),
      grid.node);
  }
  function people() {
    const rows = [...data.users].sort((a, b) => byName(personName(a), personName(b)));
    watchInvitations(true);
    return {
      node: listView({
        eyebrow: "PEOPLE", title: "People", description: "Accounts that can sign in to riAuth and the applications it protects.",
        action: link(hash("people", "new"), "New person", { class: "button primary" }),
        caption: "People", rows, filter: (u) => `${u.display_name} ${u.username} ${u.email || ""}`,
        empty: "No people yet.", before: invitations(),
        columns: [
          { label: "Person", cell: (u) => h("span", { class: "cell-title" }, link(hash("people", u.username), personName(u)), h("small", {}, u.username)) },
          { label: "Email", cell: (u) => u.email ? h("span", {}, u.email, u.email_verified ? null : [" ", badge("Unverified", "warn")]) : "—" },
          { label: "Role", cell: (u) => u.admin ? badge("Administrator", "info") : "Member" },
          { label: "MFA", cell: (u) => u.mfa_enabled ? badge("On", "ok") : badge("Not set up", u.admin ? "warn" : "muted") },
          { label: "Status", cell: (u) => { const invitation = invitationOf(u); return invitation ? invitationBadge(invitation) : u.enabled ? badge("Active", "ok") : badge("Disabled", "muted"); } },
        ],
      }),
    };
  }
  function person(username) {
    const user = data.users.find((u) => u.username === username);
    if (!user) return missing("people", "Person");
    const self = data.me.user.id === user.id;
    const selfNote = self ? " This includes your own sessions, so you'll need to sign in again." : "";
    const invitation = invitationOf(user);
    const form = h("form", { class: "admin-form", novalidate: true },
      card("Profile",
        field("Display name", h("input", { id: "person-name", maxlength: "200", value: user.display_name })),
        field("Email", h("input", { id: "person-email", type: "email", maxlength: "320", spellcheck: "false", value: user.email || "" })),
        check("person-verified", "Email address verified", user.email_verified, "A changed address is saved unverified; they confirm it themselves.")),
      card("Access",
        check("person-enabled", "Account enabled", user.enabled, self ? "Disabling your own account signs you out of this page." : invitation ? "Accepting the invitation enables the account. Enabling it here stops the invitation link from working." : "Disabling signs them out everywhere."),
        check("person-admin", "Administrator", user.admin, self ? "Removing your own administrator role ends your access to this page. riAuth keeps at least one enabled administrator." : "Administrators manage everything on these pages.")),
      actions(h("button", { class: "button primary", type: "submit" }, "Save changes"), link(hash("people"), "Cancel", { class: "button secondary" })));
    bindForm(form, async (key) => {
      const patch = {};
      const name = value(form, "person-name"), email = value(form, "person-email");
      if (name !== user.display_name) patch.display_name = name;
      if (email !== (user.email || "")) { if (!email) throw invalid("Email can't be removed here. Enter an address."); patch.email = email; }
      // A changed address is never marked verified here: Core resets verification when the
      // address changes, and the admin route refuses a payload that would override that.
      const verified = checked(form, "person-verified");
      if (!patch.email && verified !== user.email_verified) patch.email_verified = verified;
      if (checked(form, "person-enabled") !== user.enabled) patch.enabled = checked(form, "person-enabled");
      if (checked(form, "person-admin") !== user.admin) patch.admin = checked(form, "person-admin");
      if (!Object.keys(patch).length) throw invalid("There are no changes to save.");
      await api("PATCH", `admin/users/${seg(user.username)}`, patch, { revision: data.revision, key });
      await saved(`Saved ${personName({ ...user, ...patch })}.`);
    }, { 409: (error) => /last enabled administrator/i.test(error.message) ? "riAuth needs at least one enabled administrator, so this change was not saved." : undefined });
    // Editing the address clears and locks the verified box until it matches the stored one.
    const emailInput = form.querySelector("#person-email"), verifiedBox = form.querySelector("#person-verified");
    emailInput.addEventListener("input", () => {
      const changed = emailInput.value.trim() !== (user.email || "");
      verifiedBox.disabled = changed;
      verifiedBox.checked = changed ? false : user.email_verified;
    });
    const memberOf = data.groups.filter((g) => g.members.includes(user.id)).map((g) => g.name).sort(byName);
    const others = data.groups.map((g) => g.name).filter((name) => !memberOf.includes(name)).sort(byName);
    const membership = card("Groups",
      memberOf.length ? h("ul", { class: "chip-list" }, memberOf.map((name) => h("li", { class: "chip" }, link(hash("groups", name), name),
        h("button", { class: "text-button", type: "button", "aria-label": `Remove ${user.username} from ${name}`, onclick: (event) => member(event.currentTarget, name, user.username, false) }, "Remove"))))
        : h("p", { class: "field-hint" }, "Not in any group."),
      others.length ? addTo("person-add-group", "Add to group", others.map((name) => [name, name]), (button, name) => member(button, name, user.username, true)) : null);
    const passwordAction = user.password_available === false && user.mfa_enabled ? null : h("button", { class: "button secondary", type: "button", onclick: () => confirmAction({
      title: `Set a new password for ${user.username}?`, ok: "Set password", input: { label: "New password", type: "password" },
      text: `The password policy and history apply. Setting it signs them out everywhere.${selfNote}`,
      run: async (password, key) => { await api("PATCH", `admin/users/${seg(user.username)}`, { password }, { revision: data.revision, key }); await saved("Password set."); },
    }) }, "Set a new password");
    const mfaAction = user.mfa_enabled && user.password_available !== false ? h("button", { class: "button secondary", type: "button", onclick: () => confirmAction({
      title: `Reset MFA for ${user.username}?`, ok: "Reset MFA", danger: true,
      text: `Removes their passkeys, authenticator app and recovery codes and signs them out everywhere. Until they enroll again they sign in with a password only.${selfNote}`,
      run: async (_, key) => { await api("PATCH", `admin/users/${seg(user.username)}`, { reset_mfa: true }, { revision: data.revision, key }); await saved("MFA reset."); },
    }) }, "Reset MFA") : null;
    const signOutAction = h("button", { class: "button secondary", type: "button", onclick: () => confirmAction({
      title: `Sign ${user.username} out everywhere?`, ok: "Sign out everywhere", danger: true,
      text: `Ends their browser and terminal sessions and application grants.${selfNote}`,
      run: async (_, key) => { await api("PATCH", `admin/users/${seg(user.username)}`, { revoke_sessions: true }, { revision: data.revision, key }); await saved("Signed out everywhere."); },
    }) }, "Sign out everywhere");
    const invitationCard = invitation ? card("Invitation",
      h("dl", { class: "facts" },
        h("dt", {}, "Status"), h("dd", {}, invitationBadge(invitation)),
        h("dt", {}, "Link"), h("dd", {}, linkState(invitation)),
        h("dt", {}, "Delivery"), h("dd", {}, inviteDelivery(invitation)),
        h("dt", {}, "Adds groups"), h("dd", {}, invitation.groups.length ? invitation.groups.join(", ") : "None"),
        h("dt", {}, "Invited by"), h("dd", {}, invitation.invited_by ? actorName(invitation.invited_by) : "—")),
      h("p", { class: "field-hint" }, "Queued means the message is in riAuth's outbox; accepted by the mail server doesn't confirm it reached the inbox."),
      invitation.status === "inactive" ? null : h("div", { class: "stack" }, h("button", { class: "button secondary", type: "button", onclick: () => revokeInvitation(invitation) }, "Revoke invitation")),
      reissueForm(user, invitation)) : null;
    if (invitation) watchInvitations(true);
    const node = h("div", {},
      heading("PERSON", personName(user), null),
      h("p", { class: "badges" }, h("code", {}, user.username), self ? badge("You", "info") : null, user.admin ? badge("Administrator", "info") : null,
        invitation ? invitationBadge(invitation) : user.enabled ? badge("Active", "ok") : badge("Disabled", "muted"), user.mfa_enabled ? badge("MFA on", "ok") : badge("MFA not set up", user.admin ? "warn" : "muted")),
      h("div", { class: "detail-grid" }, form,
        h("div", { class: "detail-side" }, invitationCard, membership,
          card("Sign-in and security",
            h("p", { class: "field-hint" }, user.password_available === false && user.admin ? "Passkey-only administrator. Keep two passkeys on separate devices or security keys. Password recovery requires the offline operator procedure." : user.mfa_enabled ? "Signs in with a passkey or an authenticator code." : "Has no passkey or authenticator app. Only they can add one, after signing in."),
            h("div", { class: "stack" }, passwordAction, mfaAction, signOutAction)),
          card("Record", h("dl", { class: "facts" }, h("dt", {}, "Created"), h("dd", {}, when(user.created_at)), h("dt", {}, "ID"), h("dd", {}, h("code", {}, user.id)))))));
    return { crumb: personName(user), node };
  }
  function newPerson() {
    const passkeyFields = h("div", { hidden: true },
      field("Primary passkey name", h("input", { id: "new-primary-passkey", maxlength: "200", autocomplete: "off", value: "Primary device" })),
      field("Backup passkey name", h("input", { id: "new-backup-passkey", maxlength: "200", autocomplete: "off", value: "Backup security key" })),
      h("p", { class: "notice" }, "Enroll two different passkeys. Use a separate device or security key for the backup. riAuth can verify distinct credentials, but cannot prove that synced passkeys are stored independently."));
    const passwordField = field("Initial password", h("input", { id: "new-password", type: "password", maxlength: "1024", required: true, autocomplete: "new-password" }), "Share it through a secure channel. They can add a passkey after signing in.");
    const cancelEnrollment = h("button", { class: "button secondary", type: "button", hidden: true }, "Cancel passkey enrollment");
    // Without configured email delivery the server refuses invitations, so only the
    // initial-password and passkey paths are offered.
    const setupOption = (value, label, hint, isChecked, disabled) => h("label", { class: "choice" },
      h("input", { type: "radio", name: "new-setup", value, checked: isChecked, disabled }), h("span", {}, h("strong", {}, label), h("small", {}, hint)));
    const roles = h("div", {},
      check("new-admin", "Administrator", false, "Administrators manage everything on these pages."),
      check("new-passkey-only", "Passkey-only administrator", false, "Requires your recent passkey or authenticator sign-in. Both passkeys must be enrolled before the account is created."));
    const names = data.groups.map((g) => g.name).sort(byName);
    const inviteGroups = card("Groups", h("fieldset", { class: "group-checks" }, h("legend", {}, "Add to groups"),
      h("p", { class: "field-hint" }, "Optional. They join these groups when they accept the invitation."),
      names.length ? names.map((name, index) => check(`invite-group-${index}`, name, false)) : h("p", { class: "field-hint" }, "There are no groups yet.")));
    const form = h("form", { class: "admin-form", novalidate: true },
      card("Account",
        h("fieldset", { class: "choices" }, h("legend", {}, "How they get access"),
          setupOption("invite", "Email an invitation", data.mail ? `They set their own password from an emailed link that works for ${days(data.lifetime)}. Invitations create ordinary accounts, not administrators.` : "Email delivery isn't configured on this server, so invitations can't be sent.", data.mail, !data.mail),
          setupOption("password", "Set an initial password", "Share it through a secure channel. Works without email delivery.", !data.mail, false)),
        field("Username", h("input", { id: "new-username", maxlength: "64", required: true, spellcheck: "false", autocomplete: "off", autocapitalize: "none" })),
        field("Display name", h("input", { id: "new-display", maxlength: "200", autocomplete: "off" })),
        field("Email", h("input", { id: "new-email", type: "email", maxlength: "320", spellcheck: "false", autocomplete: "off" })),
        passwordField, passkeyFields, roles),
      inviteGroups,
      actions(h("button", { class: "button primary", type: "submit" }, "Create person"), cancelEnrollment, link(hash("people"), "Cancel", { class: "button secondary" })));
    const mode = form.querySelector("#new-passkey-only"), admin = form.querySelector("#new-admin");
    const inviting = () => form.querySelector("input[name=new-setup]:checked").value === "invite";
    const sync = () => {
      const invite = inviting();
      passwordField.hidden = invite || mode.checked; passkeyFields.hidden = invite || !mode.checked;
      roles.hidden = invite; inviteGroups.hidden = !invite;
      form.querySelector("#new-password").required = !invite && !mode.checked;
      form.querySelector("#new-email").required = invite;
      for (const id of ["new-primary-passkey", "new-backup-passkey"]) form.querySelector(`#${id}`).required = !invite && mode.checked;
      form.dataset.submitLabel = invite ? "Send invitation" : mode.checked ? "Enroll primary passkey" : "Create person";
      form.querySelector("[type=submit]").textContent = form.dataset.submitLabel;
    };
    mode.addEventListener("change", () => { if (mode.checked) admin.checked = true; sync(); });
    for (const radio of form.querySelectorAll("input[name=new-setup]")) radio.addEventListener("change", sync);
    sync();
    admin.addEventListener("change", () => { if (!admin.checked && mode.checked) { mode.checked = false; mode.dispatchEvent(new Event("change")); } });
    let firstFlow = null, backupFlow = null, backupStart = null, finishKey = null;
    cancelEnrollment.addEventListener("click", async () => {
      cancelEnrollment.disabled = true;
      try {
        if (backupStart && backupFlow && !await backupFlow.cancel()) return;
        if (backupStart) await api("POST", "admin/users/passkey/cancel", { ceremony: backupStart.ceremony });
        if (!backupStart && firstFlow && !await firstFlow.cancel()) return;
      } catch { /* Pending enrollment expires without creating an account. */ }
      finally { cancelEnrollment.disabled = false; }
      await refresh({ focus: true });
    });
    bindForm(form, async (key) => {
      const username = value(form, "new-username"), password = form.querySelector("#new-password").value;
      if (!username) throw invalid("Enter a username.");
      if (inviting()) {
        const email = value(form, "new-email");
        if (!email) throw invalid("Enter the email address the invitation goes to.");
        // Same default as an account created with a password: the username.
        const body = { username, email, display_name: value(form, "new-display") || username, groups: selectedGroups(form, "invite") };
        await api("POST", "admin/invitations", body, { key });
        await saved(`Invitation queued for ${email}. The link works for ${days(data.lifetime)}.`, hash("people", username));
        return;
      }
      if (mode.checked) {
        if (!RiAuth.passkeysAvailable()) throw invalid("Use a browser that supports passkeys on a secure connection.");
        if (!backupStart && !firstFlow) {
          const input = { username, display_name: value(form, "new-display"), email: value(form, "new-email") || null,
            primary_name: value(form, "new-primary-passkey"), backup_name: value(form, "new-backup-passkey") };
          if (!input.primary_name || !input.backup_name || input.primary_name === input.backup_name) throw invalid("Name the primary and backup passkeys differently.");
          firstFlow = RiAuth.passkeyFlow(
            () => api("POST", "admin/users/passkey/start", input),
            (credential, started) => api("POST", "admin/users/passkey/first", { ceremony: started.ceremony, credential }),
            true, (started) => api("POST", "admin/users/passkey/cancel", { ceremony: started.ceremony }));
          for (const input of form.querySelectorAll("input")) { if (input.type === "checkbox" || input.type === "radio") input.disabled = true; else input.readOnly = true; }
          cancelEnrollment.hidden = false;
        }
        try {
          if (!backupStart) {
            backupStart = await firstFlow();
            form.dataset.submitLabel = "Enroll backup passkey and create administrator";
            announce("Primary passkey verified. Enroll the backup on another device or security key.");
            return;
          }
          finishKey = key;
          backupFlow ||= RiAuth.passkeyFlow(() => Promise.resolve(backupStart),
            (credential, started) => api("POST", "admin/users/passkey/finish", { ceremony: started.ceremony, credential }, { revision: data.revision, key: finishKey }),
            true, (started) => api("POST", "admin/users/passkey/cancel", { ceremony: started.ceremony }));
          await backupFlow();
          await saved(`Created passkey-only administrator ${username}.`, hash("people", username));
        } catch (error) {
          if (error.name === "NotAllowedError") throw invalid("Passkey prompt closed. Select the enrollment button again to retry.");
          throw error;
        }
        return;
      }
      if (!password) throw invalid("Enter an initial password.");
      const body = { username, password, display_name: value(form, "new-display"), admin: checked(form, "new-admin"), email: value(form, "new-email") || null };
      await api("POST", "admin/users", body, { key });
      await saved(`Created ${username}.`, hash("people", username));
    }, {
      403: (error) => error.code === "mfa_required" ? "Sign in with your passkey or authenticator code in this browser, then try again." :
        error.code === "reauthentication_required" ? "Sign in again in this browser, then start a new passkey enrollment." : undefined,
      404: (error) => inviting() && /group/i.test(error.message) ? "A selected group no longer exists. Reload to see the current groups." : undefined,
      409: (error) => /already exists/i.test(error.message) ? (inviting() ? "That username belongs to an account that isn't waiting for an invitation, so it can't be invited." : "That username is already taken.") : undefined,
      503: (error) => error.code === "delivery_unavailable" ? "Email delivery isn't configured on this server, so no invitation was sent. Set an initial password instead." : undefined,
    });
    return { crumb: "New person", node: h("div", {}, heading("PEOPLE", "New person", "Invite someone by email, create an account with an initial password, or enroll a passkey-only administrator."), form) };
  }

  // ---- Groups ------------------------------------------------------------------------------
  async function member(button, groupName, username, add) {
    button.disabled = true; button.setAttribute("aria-busy", "true");
    try {
      await api(add ? "PUT" : "DELETE", `admin/groups/${seg(groupName)}/members/${seg(username)}`, undefined, { revision: data.revision, key: requestKey() });
      await saved(add ? `Added ${username} to ${groupName}.` : `Removed ${username} from ${groupName}.`);
    } catch (error) {
      if (error.status === 401) { gate("signin"); return; }
      button.disabled = false; button.removeAttribute("aria-busy");
      toast(explain(error, { 0: "riAuth didn't answer, so the membership may or may not have changed. Reload to see the current members; adding or removing a member again is safe." }));
    }
  }
  function addTo(id, label, options, onAdd) {
    const select = h("select", { id }, options.map(([valueText, text]) => h("option", { value: valueText }, text)));
    const button = h("button", { class: "button secondary", type: "button", onclick: () => onAdd(button, select.value) }, h("span", {}, "Add"));
    return h("div", { class: "add-row" }, h("div", { class: "field" }, h("label", { for: id }, label), select), button);
  }
  function groups() {
    const rows = [...data.groups].sort((a, b) => byName(a.name, b.name));
    const create = h("form", { class: "inline-form", novalidate: true },
      h("div", { class: "field" }, h("label", { for: "new-group" }, "New group"), h("input", { id: "new-group", maxlength: "64", spellcheck: "false", autocomplete: "off", autocapitalize: "none", placeholder: "engineering" })),
      actions(h("button", { class: "button primary", type: "submit" }, "Create group")));
    bindForm(create, async (key) => {
      const name = value(create, "new-group");
      if (!name) throw invalid("Enter a group name.");
      await api("POST", "admin/groups", { name }, { key });
      await saved(`Created ${name}.`, hash("groups", name));
    }, { 409: (error) => /already exists/i.test(error.message) ? "A group with this name already exists." : undefined });
    const apps = (name) => data.clients.filter((c) => c.allowed_groups.includes(name)).map((c) => c.name).sort(byName);
    return {
      node: listView({
        eyebrow: "GROUPS", title: "Groups", description: "Groups collect people. Applications can limit sign-in to members of chosen groups.",
        before: create, caption: "Groups", rows, filter: (g) => g.name, empty: "No groups yet. Create one above.",
        columns: [
          { label: "Group", cell: (g) => link(hash("groups", g.name), g.name) },
          { label: "Members", cell: (g) => String(g.members.length) },
          { label: "Applications", cell: (g) => apps(g.name).join(", ") || "—" },
        ],
      }),
    };
  }
  function group(name) {
    const entry = data.groups.find((g) => g.name === name);
    if (!entry) return missing("groups", "Group");
    const members = entry.members.map((id) => userById(id)).filter(Boolean).sort((a, b) => byName(personName(a), personName(b)));
    const others = data.users.filter((u) => !entry.members.includes(u.id)).sort((a, b) => byName(personName(a), personName(b)));
    const grid = table(`Members of ${name}`, [
      { label: "Person", cell: (u) => link(hash("people", u.username), personName(u)) },
      { label: "Username", cell: (u) => u.username },
      { label: "Status", cell: (u) => u.enabled ? badge("Active", "ok") : badge("Disabled", "muted") },
      { label: "Actions", cell: (u) => h("button", { class: "text-button", type: "button", "aria-label": `Remove ${u.username} from ${name}`, onclick: (event) => member(event.currentTarget, name, u.username, false) }, "Remove") },
    ], members, "No members yet.", (u) => `${u.display_name} ${u.username}`);
    const apps = data.clients.filter((c) => c.allowed_groups.includes(name)).sort((a, b) => byName(a.name, b.name));
    const temporary = platform() ? activeGrants().filter((grant) => grant.group === name) : [];
    const node = h("div", {},
      heading("GROUP", name, `${members.length} ${members.length === 1 ? "member" : "members"}`),
      h("div", { class: "detail-grid" },
        h("div", {}, card("Members", others.length ? addTo("group-add-member", "Add a person", others.map((u) => [u.username, `${personName(u)} (${u.username})`]), (button, username) => member(button, name, username, true)) : null, grid.node)),
        h("div", { class: "detail-side" },
          card("Applications", apps.length ? h("ul", { class: "plain-list" }, apps.map((c) => h("li", {}, link(hash("applications", c.client_id), c.name))))
            : h("p", { class: "field-hint" }, "No application is limited to this group. Choose groups on an application's page.")),
          temporary.length ? card("Temporary access", h("ul", { class: "plain-list" }, temporary.map((grant) => h("li", {}, `${personName(userById(grant.user_id))} until `, when(grant.expires_at))))) : null)));
    return { crumb: name, node };
  }

  // ---- Security ----------------------------------------------------------------------------
  function accessSecurity() {
    const waiting = pending();
    const grants = activeGrants();
    const decide = (request, approve) => confirmAction({
      title: approve ? `Approve access to ${request.group}?` : `Deny access to ${request.group}?`,
      ok: approve ? "Approve" : "Deny", danger: !approve,
      text: approve ? `${request.username} joins ${request.group} for ${duration(request.ttl)}, starting now. Reason given: “${request.reason}”.` : `${request.username} does not get access. They can send a new request.`,
      keyed: false,
      run: async () => {
        await api("POST", `admin/access/requests/${seg(request.id)}/${approve ? "approve" : "deny"}`);
        await saved(approve ? `Approved ${request.username}'s access to ${request.group}.` : `Denied ${request.username}'s request.`);
      },
      overrides: {
        0: "riAuth didn't answer, so the decision may or may not have been recorded. Reload to check: a request can only be decided once, so a repeated decision is refused rather than applied twice.",
        403: "Only an approver configured for this group can decide, and never on their own request.",
        409: "This request was already decided. Reload to see the outcome.",
      },
    });
    const requests = table("Access requests waiting for review", [
      { label: "Person", cell: (r) => link(hash("people", r.username), r.username) },
      { label: "Group", cell: (r) => link(hash("groups", r.group), r.group) },
      { label: "Reason", cell: (r) => r.reason },
      { label: "Duration", cell: (r) => duration(r.ttl) },
      { label: "Requested", cell: (r) => when(r.created_at) },
      { label: "Decision", cell: (r) => h("span", { class: "row-actions" },
        h("button", { class: "button primary small", type: "button", "aria-label": `Approve ${r.username} for ${r.group}`, onclick: () => decide(r, true) }, "Approve"),
        h("button", { class: "button secondary small", type: "button", "aria-label": `Deny ${r.username} for ${r.group}`, onclick: () => decide(r, false) }, "Deny")) },
    ], waiting, "No access requests are waiting for review.", (r) => `${r.username} ${r.group} ${r.reason}`);
    const active = table("Active temporary access", [
      { label: "Person", cell: (g) => personName(userById(g.user_id)) },
      { label: "Group", cell: (g) => link(hash("groups", g.group), g.group) },
      { label: "Ends", cell: (g) => when(g.expires_at) },
      { label: "Actions", cell: (g) => h("button", { class: "text-button", type: "button", "aria-label": `Revoke ${personName(userById(g.user_id))}'s access to ${g.group}`, onclick: () => confirmAction({
        title: `Revoke temporary access to ${g.group}?`, ok: "Revoke", danger: true,
        text: `${personName(userById(g.user_id))} loses the group's access now.`,
        keyed: false,
        run: async () => { await api("POST", `admin/access/grants/${seg(g.id)}/revoke`); await saved("Access revoked."); },
        overrides: {
          0: "riAuth didn't answer, so the access may or may not have been revoked. Reload to check; revoking again is refused if it already happened.",
          409: "This access was already revoked.",
        },
      }) }, "Revoke") },
    ], grants, "No temporary access is active.", (g) => `${g.group}`);
    return { waiting, grants, requests, active };
  }
  function security() {
    const access = platform() ? accessSecurity() : null;
    const admins = data.users.filter((u) => u.admin && u.enabled);
    const unprotected = admins.filter((u) => !u.mfa_enabled);
    const actor = (id) => {
      if (id.startsWith("agent:")) return `Agent ${id.slice(6)}`;
      const user = userById(id); return user ? user.username : id;
    };
    const activity = table("Recent activity", [
      { label: "When", cell: (e) => when(e.at) },
      { label: "Who", cell: (e) => actor(e.actor) },
      { label: "Action", cell: (e) => h("code", {}, e.action) },
      { label: "Target", cell: (e) => { const user = userById(e.target); return user ? user.username : e.target; } },
    ], data.audit, "No recorded activity.", (e) => `${actor(e.actor)} ${e.action} ${e.target}`);
    const stat = (label, count, tone, target) => h("button", { class: `stat ${tone}`.trim(), type: "button", onclick: () => {
      const section = document.getElementById(target); section.scrollIntoView({ block: "start" }); section.querySelector("h2").focus();
    } }, h("strong", {}, String(count)), h("span", {}, label));
    const node = h("div", {},
      heading("SECURITY", "Security", platform() ? "Reviews waiting on an administrator, temporary access and recent changes." : "Administrator sign-in and recent changes."),
      h("div", { class: "stats" },
        platform() ? stat("Waiting for review", access.waiting.length, access.waiting.length ? "attention" : "", "review") : null,
        platform() ? stat("Temporary access", access.grants.length, "", "temporary") : null,
        stat("Administrators", admins.length, "", "admins"),
        stat("Administrators without MFA", unprotected.length, unprotected.length ? "warn" : "ok", "admins")),
      platform() ? h("section", { class: "admin-section", id: "review", "aria-labelledby": "review-title" }, h("h2", { id: "review-title", tabindex: "-1" }, "Waiting for review"),
        h("p", { class: "field-hint" }, "People asking for temporary group access. Only approvers configured for a group can decide."), access.requests.node) : null,
      platform() ? h("section", { class: "admin-section", id: "temporary", "aria-labelledby": "temporary-title" }, h("h2", { id: "temporary-title", tabindex: "-1" }, "Temporary access"), access.active.node) : null,
      h("section", { class: "admin-section", id: "admins", "aria-labelledby": "admins-title" }, h("h2", { id: "admins-title", tabindex: "-1" }, "Administrators"),
        unprotected.length ? h("p", { class: "notice warn-notice" }, `${unprotected.length} ${unprotected.length === 1 ? "administrator has" : "administrators have"} no passkey or authenticator app. Ask them to add one from Your applications.`) : null,
        h("ul", { class: "chip-list" }, admins.map((u) => h("li", { class: "chip" }, link(hash("people", u.username), personName(u)), u.mfa_enabled ? badge("MFA on", "ok") : badge("No MFA", "warn"))))),
      h("section", { class: "admin-section", "aria-labelledby": "activity-title" }, h("div", { class: "section-heading" }, h("h2", { id: "activity-title" }, "Recent activity"),
        platform() ? h("a", { class: "text-button", href: `${base}events` }, "Open the event map") : null), activity.node));
    return { node };
  }

  // ---- Wiring ------------------------------------------------------------------------------
  window.addEventListener("hashchange", () => { if (isRoute()) render({ focus: true }); });
  document.querySelector(".skip-link").addEventListener("click", (event) => { event.preventDefault(); $("main").focus(); });
  $("refresh").addEventListener("click", () => refresh());
  $("gate-retry").addEventListener("click", () => refresh({ focus: true }));
  $("sign-out").addEventListener("click", async () => {
    $("sign-out").disabled = true;
    try {
      const result = await api("POST", "portal/sign-out", {});
      data.me = null; gate("signin");
      if (result && typeof result.saml_logout_url === "string") {
        const target = new URL(result.saml_logout_url, location.origin);
        if (target.origin === location.origin && target.pathname.startsWith(`${base}saml/logout/`)) location.assign(target.href);
      }
    } catch (error) {
      if (error.status === 401) gate("signin"); else toast(explain(error));
    } finally { $("sign-out").disabled = false; }
  });
  window.addEventListener("blur", eraseSecrets);
  window.addEventListener("pagehide", eraseSecrets);
  document.addEventListener("visibilitychange", () => { if (document.visibilityState === "hidden") eraseSecrets(); });
  // A session that ends, or becomes another account, in another tab is noticed on return.
  // Any one-time secret is erased first, synchronously, so no answer or stall can expose it.
  window.addEventListener("focus", async () => {
    eraseSecrets();
    if (!loaded) return;
    const owner = data.me.user.id;
    try {
      const me = await api("GET", "admin/session");
      if (me.user.id !== owner) { forget(); loaded = false; refresh({ focus: true }); }
    } catch (error) { if (error.status === 401 || error.status === 403) refresh(); }
  });
  document.addEventListener("keydown", (event) => {
    if (event.key !== "/" || event.ctrlKey || event.metaKey || event.altKey) return;
    const target = event.target;
    if (target instanceof HTMLElement && (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))) return;
    const box = $("search");
    if (box) { event.preventDefault(); box.focus(); }
  });
  refresh();
})();
