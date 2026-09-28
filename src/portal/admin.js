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
  const data = { me: null, revision: 0, clients: [], users: [], groups: [], requests: [], grants: [], audit: [] };
  let generation = 0, loaded = false, toastTimer, confirmRun = null, confirmOpener = null;

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
    for (const child of children.flat()) {
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
        credentials: "same-origin", mode: "same-origin", cache: "no-store", referrerPolicy: "no-referrer",
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
  function bindForm(form, action, overrides) {
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
      form.setAttribute("aria-busy", "true"); button.disabled = true; button.textContent = "Saving…"; status.hidden = true;
      try {
        await action(key);
        key = null;
      } catch (error) {
        if (error.status === 401) { gate("signin"); return; }
        if (error.status === 422) { status.replaceChildren(error.message); status.hidden = false; status.focus(); return; }
        showError(status, error, overrides);
      } finally {
        form.removeAttribute("aria-busy"); button.disabled = false; button.textContent = label;
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
  function gate(kind) {
    generation += 1; loaded = false;
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
      const [clients, users, groups, requests, grants, audit] = await Promise.all([
        api("GET", "admin/clients"), api("GET", "admin/users"), api("GET", "admin/groups"),
        api("GET", "admin/access/requests"), api("GET", "admin/access/grants"), api("GET", "admin/audit?limit=50"),
      ]);
      if (run !== generation) return;
      Object.assign(data, { me, revision: me.revision, clients, users, groups, requests, grants, audit });
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
    $("count-security").textContent = waiting;
    $("count-security").classList.toggle("attention", waiting > 0);
    $("count-security").setAttribute("aria-label", `${waiting} access ${waiting === 1 ? "request" : "requests"} waiting for review`);
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
  const OIDC_DEFAULT = "openid profile email";
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
    const form = h("form", { class: "admin-form", novalidate: true },
      card("Details",
        field("Name", h("input", { id: "app-name", maxlength: "200", required: true, value: client.name })),
        check("app-enabled", "Enabled", client.enabled, "Disabling signs this application out: its existing grants are revoked and are not restored by enabling it again."),
        client.service ? null : check("app-mfa", "Require a passkey or authenticator code", client.require_mfa)),
      client.service
        ? card("API access", field("API scopes", h("input", { id: "app-scopes", spellcheck: "false", autocomplete: "off", value: sorted(client.scopes).join(" ") }), SERVICE_HINT))
        : card("Access", groupChecks("app", sorted(client.allowed_groups))),
      client.service ? null : card("Sign-in",
        field("Redirect URIs", h("textarea", { id: "app-redirects", rows: "3", spellcheck: "false", value: client.redirect_uris.join("\n") }), "One per line. riAuth sends people back only to these exact addresses."),
        field("Scopes", h("input", { id: "app-scopes", spellcheck: "false", autocomplete: "off", value: sorted(client.scopes).join(" ") }), OIDC_HINT)),
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
        const redirects = lines(form.querySelector("#app-redirects").value);
        if (!same(redirects, client.redirect_uris)) patch.redirect_uris = redirects;
      }
      const scopes = sorted(words(value(form, "app-scopes")));
      if (!same(scopes, sorted(client.scopes))) { checkScopes(scopes, client.service); patch.scopes = scopes; }
      const next = { ...app, description: value(form, "app-description"), category: value(form, "app-category"), launch_url: value(form, "app-launch") || null, icon: value(form, "app-icon"), accent: value(form, "app-accent"), hidden: checked(form, "app-hidden") };
      const before = { description: "", category: "", launch_url: null, icon: "", accent: "", hidden: false, launch_scopes: [], ...app };
      // A settings update replaces the whole object, so the current settings travel with it.
      if (!same({ ...before, ...next }, before)) patch.settings = { ...client.settings, app: { ...before, ...next } };
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
          card("Credentials", secret),
          card("Protocol settings", h("p", { class: "field-hint" }, "Change these with riauth client update or a desired-state manifest; saving this page keeps them."),
            h("details", {}, h("summary", {}, "Show current settings"), h("pre", { class: "settings-json" }, JSON.stringify(protocol, null, 2)))))));
    return { crumb: client.name, node };
  }
  function newApplication() {
    const typeOption = (value, label, hint, isChecked) => h("label", { class: "choice" },
      h("input", { type: "radio", name: "app-type", value, checked: isChecked }), h("span", {}, h("strong", {}, label), h("small", {}, hint)));
    const redirects = field("Redirect URIs", h("textarea", { id: "new-redirects", rows: "3", spellcheck: "false", placeholder: "https://app.example.com/oauth/callback" }), "One per line: the exact callback addresses of the app.");
    const scopeInput = h("input", { id: "new-scopes", spellcheck: "false", autocomplete: "off", value: OIDC_DEFAULT });
    const scopes = field("Scopes", scopeInput, OIDC_HINT);
    const mfa = check("new-mfa", "Require a passkey or authenticator code", false);
    const access = card("Access", groupChecks("new", []));
    let scopesEdited = false;
    scopeInput.addEventListener("input", () => { scopesEdited = true; });
    const form = h("form", { class: "admin-form", novalidate: true },
      card("Application",
        field("Client ID", h("input", { id: "new-id", maxlength: "64", required: true, spellcheck: "false", autocomplete: "off", autocapitalize: "none" }), "Used in protocol requests. Letters, digits, dots, dashes and underscores."),
        field("Name", h("input", { id: "new-name", maxlength: "200", required: true })),
        h("fieldset", { class: "choices" }, h("legend", {}, "Type"),
          typeOption("confidential", "Web application", "Runs on a server and keeps a client secret.", true),
          typeOption("public", "Browser or mobile app", "Can't keep a secret; signs in with PKCE.", false),
          typeOption("service", "Service", "Machine-to-machine, with no people signing in.", false))),
      card("Sign-in", redirects, scopes, mfa),
      access,
      actions(h("button", { class: "button primary", type: "submit" }, "Create application"), link(hash("applications"), "Cancel", { class: "button secondary" })));
    const type = () => form.querySelector("input[name=app-type]:checked").value;
    // Services sign in no people: swap the untouched OIDC default for API scopes and hide the
    // redirect, group and MFA controls validate_client rejects for them.
    form.addEventListener("change", (event) => {
      if (event.target.name !== "app-type") return;
      const service = type() === "service";
      redirects.hidden = mfa.hidden = access.hidden = service;
      scopes.querySelector("label").textContent = service ? "API scopes" : "Scopes";
      scopes.querySelector(".field-hint").textContent = service ? SERVICE_HINT : OIDC_HINT;
      scopeInput.placeholder = service ? "api.read api.write" : "";
      if (!scopesEdited) scopeInput.value = service ? "" : OIDC_DEFAULT;
    });
    bindForm(form, async (key) => {
      const id = value(form, "new-id"), name = value(form, "new-name");
      if (!id || !name) throw invalid("Enter a client ID and a name.");
      const service = type() === "service";
      const body = {
        client_id: id, name, confidential: type() !== "public", service,
        redirect_uris: service ? [] : lines(form.querySelector("#new-redirects").value),
        scopes: words(value(form, "new-scopes")),
        allowed_groups: service ? [] : selectedGroups(form, "new"), require_mfa: !service && checked(form, "new-mfa"),
      };
      checkScopes(body.scopes, service);
      const result = await api("POST", "admin/clients", body, { key });
      if (result.client_secret) showSecret(id, result.client_secret);
      await saved(`Created ${name}.`, hash("applications", id));
    }, { 409: (error) => /already exists/i.test(error.message) ? "An application with this client ID already exists." : undefined });
    return { crumb: "New application", node: h("div", {}, heading("APPLICATIONS", "New application", "Connect an app to riAuth. You can add portal details after it is created."), form) };
  }

  // ---- People ------------------------------------------------------------------------------
  function people() {
    const rows = [...data.users].sort((a, b) => byName(personName(a), personName(b)));
    return {
      node: listView({
        eyebrow: "PEOPLE", title: "People", description: "Accounts that can sign in to riAuth and the applications it protects.",
        action: link(hash("people", "new"), "New person", { class: "button primary" }),
        caption: "People", rows, filter: (u) => `${u.display_name} ${u.username} ${u.email || ""}`,
        empty: "No people yet.",
        columns: [
          { label: "Person", cell: (u) => h("span", { class: "cell-title" }, link(hash("people", u.username), personName(u)), h("small", {}, u.username)) },
          { label: "Email", cell: (u) => u.email ? h("span", {}, u.email, u.email_verified ? null : [" ", badge("Unverified", "warn")]) : "—" },
          { label: "Role", cell: (u) => u.admin ? badge("Administrator", "info") : "Member" },
          { label: "MFA", cell: (u) => u.mfa_enabled ? badge("On", "ok") : badge("Not set up", u.admin ? "warn" : "muted") },
          { label: "Status", cell: (u) => u.enabled ? badge("Active", "ok") : badge("Disabled", "muted") },
        ],
      }),
    };
  }
  function person(username) {
    const user = data.users.find((u) => u.username === username);
    if (!user) return missing("people", "Person");
    const self = data.me.user.id === user.id;
    const selfNote = self ? " This includes your own sessions, so you'll need to sign in again." : "";
    const form = h("form", { class: "admin-form", novalidate: true },
      card("Profile",
        field("Display name", h("input", { id: "person-name", maxlength: "200", value: user.display_name })),
        field("Email", h("input", { id: "person-email", type: "email", maxlength: "320", spellcheck: "false", value: user.email || "" })),
        check("person-verified", "Email address verified", user.email_verified, "A changed address is saved unverified; they confirm it themselves.")),
      card("Access",
        check("person-enabled", "Account enabled", user.enabled, self ? "Disabling your own account signs you out of this page." : "Disabling signs them out everywhere."),
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
    const passwordAction = h("button", { class: "button secondary", type: "button", onclick: () => confirmAction({
      title: `Set a new password for ${user.username}?`, ok: "Set password", input: { label: "New password", type: "password" },
      text: `The password policy and history apply. Setting it signs them out everywhere.${selfNote}`,
      run: async (password, key) => { await api("PATCH", `admin/users/${seg(user.username)}`, { password }, { revision: data.revision, key }); await saved("Password set."); },
    }) }, "Set a new password");
    const mfaAction = user.mfa_enabled ? h("button", { class: "button secondary", type: "button", onclick: () => confirmAction({
      title: `Reset MFA for ${user.username}?`, ok: "Reset MFA", danger: true,
      text: `Removes their passkeys, authenticator app and recovery codes and signs them out everywhere. Until they enroll again they sign in with a password only.${selfNote}`,
      run: async (_, key) => { await api("PATCH", `admin/users/${seg(user.username)}`, { reset_mfa: true }, { revision: data.revision, key }); await saved("MFA reset."); },
    }) }, "Reset MFA") : null;
    const signOutAction = h("button", { class: "button secondary", type: "button", onclick: () => confirmAction({
      title: `Sign ${user.username} out everywhere?`, ok: "Sign out everywhere", danger: true,
      text: `Ends their browser and terminal sessions and application grants.${selfNote}`,
      run: async (_, key) => { await api("PATCH", `admin/users/${seg(user.username)}`, { revoke_sessions: true }, { revision: data.revision, key }); await saved("Signed out everywhere."); },
    }) }, "Sign out everywhere");
    const node = h("div", {},
      heading("PERSON", personName(user), null),
      h("p", { class: "badges" }, h("code", {}, user.username), self ? badge("You", "info") : null, user.admin ? badge("Administrator", "info") : null,
        user.enabled ? badge("Active", "ok") : badge("Disabled", "muted"), user.mfa_enabled ? badge("MFA on", "ok") : badge("MFA not set up", user.admin ? "warn" : "muted")),
      h("div", { class: "detail-grid" }, form,
        h("div", { class: "detail-side" }, membership,
          card("Sign-in and security",
            h("p", { class: "field-hint" }, user.mfa_enabled ? "Signs in with a passkey or an authenticator code." : "Has no passkey or authenticator app. Only they can add one, after signing in."),
            h("div", { class: "stack" }, passwordAction, mfaAction, signOutAction)),
          card("Record", h("dl", { class: "facts" }, h("dt", {}, "Created"), h("dd", {}, when(user.created_at)), h("dt", {}, "ID"), h("dd", {}, h("code", {}, user.id)))))));
    return { crumb: personName(user), node };
  }
  function newPerson() {
    const form = h("form", { class: "admin-form", novalidate: true },
      card("Account",
        field("Username", h("input", { id: "new-username", maxlength: "64", required: true, spellcheck: "false", autocomplete: "off", autocapitalize: "none" })),
        field("Display name", h("input", { id: "new-display", maxlength: "200", autocomplete: "off" })),
        field("Email", h("input", { id: "new-email", type: "email", maxlength: "320", spellcheck: "false", autocomplete: "off" })),
        field("Initial password", h("input", { id: "new-password", type: "password", maxlength: "1024", required: true, autocomplete: "new-password" }), "Share it through a secure channel. They can add a passkey after signing in."),
        check("new-admin", "Administrator", false, "Administrators manage everything on these pages.")),
      actions(h("button", { class: "button primary", type: "submit" }, "Create person"), link(hash("people"), "Cancel", { class: "button secondary" })));
    bindForm(form, async (key) => {
      const username = value(form, "new-username"), password = form.querySelector("#new-password").value;
      if (!username || !password) throw invalid("Enter a username and an initial password.");
      const body = { username, password, display_name: value(form, "new-display"), admin: checked(form, "new-admin"), email: value(form, "new-email") || null };
      await api("POST", "admin/users", body, { key });
      await saved(`Created ${username}.`, hash("people", username));
    }, { 409: (error) => /already exists/i.test(error.message) ? "That username is already taken." : undefined });
    return { crumb: "New person", node: h("div", {}, heading("PEOPLE", "New person", "Create an account with an initial password."), form) };
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
    const temporary = activeGrants().filter((grant) => grant.group === name);
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
  function security() {
    const waiting = pending();
    const grants = activeGrants();
    const admins = data.users.filter((u) => u.admin && u.enabled);
    const unprotected = admins.filter((u) => !u.mfa_enabled);
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
      heading("SECURITY", "Security", "Reviews waiting on an administrator, temporary access and recent changes."),
      h("div", { class: "stats" },
        stat("Waiting for review", waiting.length, waiting.length ? "attention" : "", "review"),
        stat("Temporary access", grants.length, "", "temporary"),
        stat("Administrators", admins.length, "", "admins"),
        stat("Administrators without MFA", unprotected.length, unprotected.length ? "warn" : "ok", "admins")),
      h("section", { class: "admin-section", id: "review", "aria-labelledby": "review-title" }, h("h2", { id: "review-title", tabindex: "-1" }, "Waiting for review"),
        h("p", { class: "field-hint" }, "People asking for temporary group access. Only approvers configured for a group can decide."), requests.node),
      h("section", { class: "admin-section", id: "temporary", "aria-labelledby": "temporary-title" }, h("h2", { id: "temporary-title", tabindex: "-1" }, "Temporary access"), active.node),
      h("section", { class: "admin-section", id: "admins", "aria-labelledby": "admins-title" }, h("h2", { id: "admins-title", tabindex: "-1" }, "Administrators"),
        unprotected.length ? h("p", { class: "notice warn-notice" }, `${unprotected.length} ${unprotected.length === 1 ? "administrator has" : "administrators have"} no passkey or authenticator app. Ask them to add one from Your applications.`) : null,
        h("ul", { class: "chip-list" }, admins.map((u) => h("li", { class: "chip" }, link(hash("people", u.username), personName(u)), u.mfa_enabled ? badge("MFA on", "ok") : badge("No MFA", "warn"))))),
      h("section", { class: "admin-section", "aria-labelledby": "activity-title" }, h("div", { class: "section-heading" }, h("h2", { id: "activity-title" }, "Recent activity"),
        h("a", { class: "text-button", href: `${base}events` }, "Open the event map")), activity.node));
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
  // A session that ends in another tab shows the sign-in state on return.
  window.addEventListener("focus", async () => {
    if (!loaded) return;
    try { await api("GET", "admin/session"); } catch (error) { if (error.status === 401 || error.status === 403) refresh(); }
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
