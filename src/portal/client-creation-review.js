"use strict";
// Browser adapter only. Authorization, canonical binding and the sole client writer
// remain in the reviewed creation service. No draft or credential uses browser storage.
(() => {
  const route = (id = "") => `#/client-creation-review${id ? `/${encodeURIComponent(id)}` : ""}`;
  const idPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  const name = (v) => typeof v === "string" && /^[A-Za-z0-9_.@-]{1,64}$/.test(v) && ![".", ".."].includes(v);
  const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
  function checkedChange(value, expectedId) {
    const p = value?.proposal, c = p?.after;
    const text = (v) => typeof v === "string" && v.length > 0 && v.length <= 256;
    const number = (v) => Number.isSafeInteger(v) && v >= 0;
    const time = (v) => number(v) && v < 8640000000000;
    const authority = (v) => text(v?.id) && number(v.epoch);
    const strings = (v, max, length) => Array.isArray(v) && v.length <= max && v.every((s) => typeof s === "string" && s.length <= length);
    if (!p || !idPattern.test(p.id) || expectedId && p.id !== expectedId || !text(value.digest)
        || !["pending", "approved", "executed", "cancelled"].includes(value.status)
        || !name(p.client_id) || p.resource !== `client/${p.client_id}` || !authority(p.author)
        || p.before !== null || p.enabled !== true || !c || c.client_id !== p.client_id
        || !text(c.name) || typeof c.confidential !== "boolean" || typeof c.service !== "boolean"
        || typeof c.require_mfa !== "boolean" || p.generate_client_secret !== c.confidential
        || !strings(c.redirect_uris, 32, 2048) || !strings(c.scopes, 32, 64)
        || !strings(c.allowed_groups, 64, 64) || !c.settings || Array.isArray(c.settings) || typeof c.settings !== "object"
        || Object.hasOwn(c, "client_secret") || Object.hasOwn(c, "secret_hash")
        || JSON.stringify(c).length > 65536 || !number(p.base_revision)
        || !text(p.resource_revision) || !text(p.policy_revision) || !time(p.created_at) || !time(p.expires_at)
        || !Array.isArray(value.approvals) || value.approvals.length > 8
        || !value.approvals.every((a) => authority(a.reviewer) && a.digest === value.digest && time(a.at))
        || value.executor !== null && !authority(value.executor)
        || value.executed_at !== null && !time(value.executed_at)
        || ["approved", "executed"].includes(value.status) && !value.approvals.length
        || value.status === "executed" && (!value.executor || value.executed_at === null)) throw { status: 502 };
    const actors = [p.author.id, ...value.approvals.map((a) => a.reviewer.id), ...(value.executor ? [value.executor.id] : [])];
    if (new Set(actors).size !== actors.length) throw { status: 502 };
    return value;
  }
  let generation = 0, clock, savedDraft = null, decision = null, secretEpoch = 0, erase = null;
  function eraseSecrets() { secretEpoch += 1; erase?.(); erase = null; }
  function reset(forget = false) {
    generation += 1; clearInterval(clock); eraseSecrets();
    if (forget) { savedDraft = null; decision = null; }
  }
  function message(e) {
    if (!e.status || e.status >= 500) return "The response was lost or incomplete. The action may have completed. Check its status or recover the same request before taking another action.";
    if (e.status === 403) return "This action is not allowed. The author, every reviewer and executor must be distinct, currently authorized administrators. Refresh and check the participants.";
    if (e.status === 404) return "This change is unavailable. It may have expired or been removed. Check the complete change ID.";
    if ([409, 428].includes(e.status)) return "The server refused this change because its content, authority, dependencies or status changed. Refresh to inspect it; stale proposals need a new review.";
    if (e.status === 429) return "Too many requests. Wait a minute before retrying.";
    return "The server rejected this content. Check the client ID, URLs, scopes and existing group names. Creation review must be enabled; only the supported OIDC and service settings are accepted.";
  }
  function view({ id, api, h, me, users, identityChanged, sessionLost }) {
    const run = ++generation, owner = me?.user?.id;
    if (savedDraft?.owner !== owner || !me?.user?.admin) savedDraft = null;
    if (decision?.owner !== owner || !me?.user?.admin) decision = null;
    const root = h("div", { class: "grant-review client-creation-review" });
    const active = () => run === generation && root.isConnected;
    const status = h("p", { class: "form-error", role: "alert", tabindex: "-1", hidden: true });
    root.append(h("section", { class: "page-heading" }, h("div", {}, h("p", { class: "eyebrow" }, "APPLICATIONS"),
      h("h1", { tabindex: "-1" }, id ? "Review application creation" : "Reviewed applications"),
      h("p", { class: "page-description" }, "Stage an exact application. A second administrator reviews it and a third creates it once."))), status);
    const localError = (text) => { status.textContent = text; status.hidden = false; status.focus(); };
    const error = (e) => { if (active()) { if (e.status === 401) sessionLost(401); else localError(message(e)); } };
    const button = (text, click, attrs = {}) => h("button", { type: "button", class: "button secondary", onclick: click, ...attrs }, text);
    const field = (label, input, hint) => h("div", { class: "field" }, h("label", { for: input.id }, label), input,
      hint ? h("p", { class: "field-hint" }, hint) : null);
    const facts = (rows) => h("dl", { class: "facts grant-facts" }, rows.map(([label, value]) => [h("dt", {}, label), h("dd", {}, value)]));
    const actor = (value) => h("span", { class: "grant-identity" }, h("strong", {}, users.find((u) => u.id === value)?.username || "Administrator"), h("code", {}, value));
    const date = (seconds) => h("time", { datetime: new Date(seconds * 1000).toISOString() }, new Date(seconds * 1000).toLocaleString(undefined, { timeZoneName: "short" }));
    const key = () => crypto.randomUUID();
    async function session() {
      const live = await api("GET", "admin/session");
      if (!active()) return null;
      if (live?.user?.id !== owner) { identityChanged(); return null; }
      if (!live.user.admin) { sessionLost(403); return null; }
      return live;
    }
    if (!me?.user?.admin) {
      root.append(h("p", { class: "notice" }, "A full administrator account is required to review application creation."));
      return { node: root };
    }
    const otherDecision = decision && decision.id !== id;
    if (otherDecision) root.append(h("p", { class: "notice warn-notice" }, "An earlier action is unconfirmed. ", h("a", { href: route(decision.id) }, "Open that change to resolve it first.")));
    if (!id) {
      const lookup = h("input", { id: "creation-lookup", maxlength: "36", required: true, autocomplete: "off" });
      const open = h("form", { class: "inline-form" }, field("Change ID", lookup), h("button", { type: "submit", class: "button secondary" }, "Open change"));
      open.addEventListener("submit", (event) => {
        event.preventDefault();
        if (!idPattern.test(lookup.value.trim())) { localError("Enter the complete change ID from the author."); return; }
        location.hash = route(lookup.value.trim());
      });
      root.append(open);
      if (!me.reviewed_client_creation) {
        root.append(h("p", { class: "notice" }, "Application creation review is not enabled. You can still inspect existing changes."),
          h("a", { href: "#/applications/new", class: "button secondary" }, "Set up an application"));
        return { node: root };
      }
      savedDraft ||= { owner, revision: null, pending: null, blocked: false,
        values: { name: "", client_id: "", type: "web", redirects: "", scopes: "openid profile", groups: "", mfa: false, origins: "", logout: "" } };
      const record = savedDraft, inputs = {};
      const input = (key, attrs = {}) => inputs[key] = h("input", { id: `creation-${key}`, value: record.values[key], autocomplete: "off", ...attrs });
      const area = (key) => inputs[key] = h("textarea", { id: `creation-${key}`, rows: "3", maxlength: "8192", spellcheck: "false", autocomplete: "off", value: record.values[key] });
      const type = inputs.type = h("select", { id: "creation-type" }, [
        ["web", "Web application · generated secret"], ["spa", "Browser application · public"],
        ["native", "Native application · public"], ["service", "Service · generated secret"],
      ].map(([value, text]) => h("option", { value, selected: value === record.values.type }, text)));
      const mfa = inputs.mfa = h("input", { id: "creation-mfa", type: "checkbox", checked: record.values.mfa });
      const ack = h("input", { id: "creation-stage-ack", type: "checkbox" });
      const stage = h("button", { type: "submit", class: "button primary" }, "Stage exact application");
      const preview = h("pre", { class: "creation-content", id: "creation-preview" });
      const revision = h("p", { class: "field-hint" });
      const rebase = button("Use current revision", async () => {
        if (busy || record.pending) return;
        busy = true; update();
        try {
          const live = await session();
          if (!live) return;
          if (!live.reviewed_client_creation) throw { status: 409 };
          record.revision = live.revision; record.blocked = false; ack.checked = false; status.hidden = true;
        } catch (e) { error(e); }
        finally { busy = false; if (active()) update(); }
      });
      const form = h("form", { class: "admin-form admin-card", novalidate: true }, h("h2", {}, "Stage an application"),
        h("p", { class: "field-hint" }, "This form supports enabled OIDC and service clients. Staging creates no application or secret. Drafts stay in this tab's memory; a full page reload discards unsent edits."),
        field("Application name", input("name", { maxlength: "200", required: true })),
        field("Client ID", input("client_id", { maxlength: "64", required: true, spellcheck: "false" }), "Choose a new, exact identifier."),
        field("Application type", type), field("Redirect URIs", area("redirects"), "One exact callback per line; unused for services."),
        field("Scopes", input("scopes", { maxlength: "2048", required: true }), "Space-separated. Interactive clients need openid; services use API scopes."),
        field("Allowed groups", area("groups"), "One existing group name per line. Empty allows any otherwise eligible user; unused for services."),
        h("label", { class: "checkbox", for: mfa.id }, mfa, "Require MFA"),
        field("Allowed browser origins", area("origins"), "One origin per line; unused for services."),
        field("After sign-out, return to", area("logout"), "Optional exact URLs, one per line; unused for services."),
        h("h3", {}, "Exact content to stage"), preview, revision, rebase,
        h("label", { class: "checkbox", for: ack.id }, ack, "I checked the complete application content."),
        h("div", { class: "form-actions" }, stage));
      root.append(form);
      let busy = false;
      const lines = (text) => [...new Set(text.split(/\n/).map((s) => s.trim()).filter(Boolean))];
      function body() {
        const v = record.values, service = v.type === "service", confidential = service || v.type === "web";
        return { client_id: v.client_id.trim(), name: v.name.trim(), confidential, service,
          redirect_uris: service ? [] : lines(v.redirects), scopes: [...new Set(v.scopes.split(/\s+/).filter(Boolean))].sort(),
          allowed_groups: service ? [] : lines(v.groups).sort(), require_mfa: !service && v.mfa,
          settings: { token_endpoint_auth_method: confidential ? "client_secret_basic" : "none", native: v.type === "native",
            origins: service ? [] : lines(v.origins).sort(), post_logout_redirect_uris: service ? [] : lines(v.logout) } };
      }
      function update() {
        const locked = busy || !!record.pending, service = record.values.type === "service";
        for (const [key, node] of Object.entries(inputs)) node.disabled = locked || service && ["redirects", "groups", "mfa", "origins", "logout"].includes(key);
        ack.disabled = locked || record.blocked || record.revision === null;
        stage.disabled = busy || record.blocked || record.revision === null || otherDecision || !record.pending && !ack.checked;
        stage.textContent = record.pending ? "Retry same staging request" : "Stage exact application";
        rebase.disabled = locked;
        preview.textContent = JSON.stringify(record.pending?.body || body(), null, 2);
        revision.textContent = record.revision === null ? "Load the current revision before staging."
          : record.pending ? "Staging is unconfirmed. Content and revision are locked. Retry the same request; avoid reloading this page until it is resolved."
          : `Based on management revision ${record.revision}. Refresh preserves the draft and its original revision. Use current revision explicitly, then check the content again.`;
      }
      for (const [key, node] of Object.entries(inputs)) node.addEventListener("input", () => {
        record.values[key] = key === "mfa" ? node.checked : node.value;
        if (key === "type") {
          if (node.value === "service" && record.values.scopes === "openid profile") inputs.scopes.value = record.values.scopes = "api";
          if (node.value !== "service" && record.values.scopes === "api") inputs.scopes.value = record.values.scopes = "openid profile";
        }
        ack.checked = false; update();
      });
      ack.addEventListener("change", update); update();
      // A new draft needs a fresh revision, even when arriving by hash navigation
      // after another administrator executed a change. Existing drafts never rebase
      // implicitly on refresh, including an unconfirmed staging request.
      if (record.revision === null) queueMicrotask(() => { if (active()) rebase.click(); });
      form.addEventListener("submit", async (event) => {
        event.preventDefault();
        if (stage.disabled || savedDraft !== record) return;
        const content = body();
        if (!name(content.client_id) || !content.name || !content.scopes.length) { localError("Enter a client ID, application name and at least one scope."); return; }
        record.pending ||= { body: content, revision: record.revision, key: key() };
        busy = true; status.hidden = true; update();
        try {
          if (!await session()) return;
          const change = checkedChange(await api("POST", "admin/client-creation-changes", record.pending.body, record.pending));
          if (!await session()) return;
          if (change.proposal.author.id !== owner) throw { status: 403 };
          if (active()) { savedDraft = null; location.hash = route(change.proposal.id); }
        } catch (e) {
          if (e.status && e.status < 500) { record.pending = null; record.blocked = [403, 409, 428].includes(e.status); }
          error(e);
        } finally { busy = false; if (active()) update(); }
      });
      return { node: root };
    }
    if (!idPattern.test(id)) {
      root.append(h("p", { class: "notice" }, "This change ID is invalid."), h("a", { href: route() }, "Open another change"));
      return { node: root, crumb: "Invalid change" };
    }
    const endpoint = `admin/client-creation-changes/${encodeURIComponent(id)}`;
    const refresh = button("Refresh change", () => reload());
    const detail = h("div", { class: "grant-detail", "aria-busy": "true" });
    root.append(h("div", { class: "form-actions" }, refresh, h("a", { href: route(), class: "button secondary" }, "Stage or open another change")), detail);
    let row, currentSession, busy = false, rejected = false, read = 0;
    const pending = () => decision?.id === id ? decision : null;
    async function reload() {
      if (busy) return;
      eraseSecrets(); clearInterval(clock);
      const request = ++read;
      refresh.disabled = true; detail.setAttribute("aria-busy", "true"); detail.replaceChildren(h("p", {}, "Loading immutable proposal…"));
      try {
        const live = await session();
        if (!live) return;
        const result = checkedChange(await api("GET", endpoint), id);
        if (!active() || request !== read) return;
        if (pending() && (!same(result.proposal, decision.proposal) || result.digest !== decision.digest)) throw { status: 502 };
        if (pending() && decision.action !== "execute") decision = null;
        row = result; currentSession = live; status.hidden = true; draw();
      } catch (e) { if (active() && request === read) { detail.replaceChildren(); error(e); } }
      finally { if (active() && request === read) { refresh.disabled = false; detail.removeAttribute("aria-busy"); } }
    }
    function state() {
      if (pending()) return ["Outcome unknown", "warn", "Refresh to inspect status. If execution is unconfirmed, recover its original receipt with the same request; do not create a second request."];
      if (row.status === "executed") return ["Executed", "ok", "This application was created once. This change cannot execute again."];
      if (row.status === "cancelled") return ["Cancelled", "muted", "This change cannot be approved or executed."];
      if (row.proposal.expires_at <= Date.now() / 1000) return ["Expired", "warn", "The approval window ended. Stage a new proposal for a new review."];
      if (rejected || !currentSession.reviewed_client_creation || row.proposal.base_revision !== currentSession.revision) return ["Stale", "warn", "A dependency, authority, policy or management revision changed. Stage a new proposal for a new review."];
      return row.status === "approved" ? ["Approved", "info", "An independent administrator may execute this exact creation once."]
        : ["Awaiting review", "info", "An administrator other than the author must approve the exact content."];
    }
    function showSecret(secret, eligible) {
      if (!secret) return;
      const panel = h("section", { class: "admin-card creation-secret" });
      detail.prepend(panel);
      const clear = () => {
        const value = panel.querySelector("textarea"); if (value) value.value = "";
        panel.replaceChildren(h("p", { class: "notice warn-notice" }, "The one-time secret was erased. If it was not saved, rotate it on the application's page."));
      };
      if (!eligible || document.visibilityState === "hidden" || !document.hasFocus()) { clear(); return; }
      const value = h("textarea", { id: "creation-secret", rows: "2", readOnly: true, autocomplete: "off", spellcheck: "false", value: secret });
      panel.append(h("h2", {}, "Save the client secret now"), field("One-time client secret", value),
        h("p", { class: "field-hint" }, "Shown only in this execution response. Refresh, navigation, changing tabs or losing focus erases it. It is never saved in browser storage."),
        button("I saved the secret · erase it", eraseSecrets));
      erase = clear;
    }
    function draw() {
      const p = row.proposal;
      const label = h("strong", { class: "badge", id: "creation-status", role: "status" });
      const explanation = h("p", { class: "field-hint" });
      const ack = h("input", { id: "creation-review-ack", type: "checkbox" });
      const approve = button("Approve exact application", () => act("approve"), { class: "button primary" });
      const execute = button("Create application once", () => act("execute"), { class: "button primary" });
      const cancel = button("Cancel change", () => act("cancel"));
      const recover = button("Recover execution result", () => act("execute", true));
      const hint = h("p", { class: "field-hint" });
      const update = () => {
        if (!active()) { clearInterval(clock); return; }
        const [name, tone, text] = state();
        label.textContent = name; label.className = `badge ${tone}`; explanation.textContent = text;
        const ended = ["Executed", "Cancelled", "Expired"].includes(name), author = p.author.id === owner;
        const reviewer = row.approvals.some((a) => a.reviewer.id === owner);
        const available = !busy && !pending() && !otherDecision && !ended && name !== "Stale";
        approve.disabled = !available || !ack.checked || author || reviewer || row.approvals.length >= 8;
        execute.disabled = !available || !ack.checked || author || reviewer || row.status !== "approved";
        cancel.disabled = busy || !!pending() || !!otherDecision || ended;
        ack.disabled = !available; refresh.disabled = busy;
        recover.hidden = pending()?.action !== "execute"; recover.disabled = busy;
        hint.textContent = author ? "You are the author. Other administrators must review and execute."
          : reviewer ? "You approved this application. Another administrator must execute."
          : "The author, all reviewers and executor must be distinct. The server checks live authority and dependencies for every action.";
        if (currentSession.expires_at <= Date.now() / 1000) sessionLost(401);
      };
      ack.addEventListener("change", update);
      const reviewers = row.approvals.length ? h("ul", { class: "plain-list" }, row.approvals.map((a) => h("li", {}, actor(a.reviewer.id), " · ", date(a.at)))) : "No reviewers yet";
      detail.replaceChildren(
        h("section", { class: "admin-card" }, h("h2", {}, p.after.name), label, explanation,
          facts([["Resource", h("code", {}, p.resource)], ["Author", actor(p.author.id)], ["Reviewers", reviewers],
            ["Executor", row.executor ? actor(row.executor.id) : "Not executed"], ["Created", date(p.created_at)], ["Expires", date(p.expires_at)],
            ...(row.executed_at ? [["Executed", date(row.executed_at)]] : [])])),
        h("section", { class: "admin-card" }, h("h2", {}, "Before"), h("p", {}, "No application exists at this client ID in the staged snapshot.")),
        h("section", { class: "admin-card" }, h("h2", {}, "After · exact application"),
          h("p", {}, p.generate_client_secret ? "Initially enabled. A secret will be generated only during execution." : "Initially enabled. Public client; no secret will be generated."),
          facts([["Client ID", p.client_id], ["Type", p.after.service ? "Service" : p.after.confidential ? "Confidential OIDC" : "Public OIDC"],
            ["Redirect URIs", p.after.redirect_uris.join("\n") || "None"], ["Scopes", p.after.scopes.join(", ")],
            ["Allowed groups", p.after.allowed_groups.join(", ") || "No group restriction"], ["Require MFA", p.after.require_mfa ? "Yes" : "No"]]),
          h("details", {}, h("summary", {}, "Complete client content, including all settings"),
            h("pre", { class: "creation-content", id: "creation-content" }, JSON.stringify(p.after, null, 2)))),
        h("section", { class: "admin-card" }, h("h2", {}, "Immutable approval binding"),
          facts([["Change ID", h("code", { id: "creation-id" }, p.id)], ["Canonical digest", h("code", { id: "creation-digest" }, row.digest)],
            ["Management revision", String(p.base_revision)], ["Resource dependencies", h("code", {}, p.resource_revision)], ["Policy dependencies", h("code", {}, p.policy_revision)]]),
          h("p", { class: "field-hint" }, "Approval binds all content, dependencies and expiry. Refresh does not certify freshness; the server rechecks it. Changed intent needs a new proposal."),
          field("Review link", h("input", { id: "creation-review-link", readOnly: true, value: new URL(route(id), location.href).href }), "Share with the next administrator. This link contains no secret.")),
        h("section", { class: "admin-card" }, h("h2", {}, "Your action"), hint,
          h("label", { class: "checkbox", for: ack.id }, ack, "I checked the exact application, digest and dependencies."),
          h("div", { class: "form-actions" }, approve, execute, cancel, recover),
          row.status === "executed" ? h("a", { href: `#/applications/${encodeURIComponent(p.client_id)}`, class: "button secondary" }, "Go to the application") : null));
      clearInterval(clock); clock = setInterval(update, 1000); update();
      async function act(action, retry = false) {
        if (busy || (retry ? recover : { approve, execute, cancel }[action]).disabled) return;
        if (retry && pending()?.action !== "execute") return;
        busy = true; status.hidden = true; const epoch = secretEpoch; update();
        let secret = null;
        try {
          const live = await session();
          if (!live) return;
          currentSession = live;
          if (!retry && action !== "cancel" && ["Stale", "Expired"].includes(state()[0])) return;
          decision ||= { owner, id, action, digest: row.digest, proposal: row.proposal, revision: live.revision, key: key() };
          const request = decision;
          const response = await api("POST", `${endpoint}/${request.action}`, { digest: request.digest }, request);
          const result = checkedChange(action === "execute" ? response?.change : response, id);
          if (result.digest !== request.digest || !same(result.proposal, request.proposal)) throw { status: 502 };
          if (action === "execute") {
            secret = response.client_secret;
            response.client_secret = null;
            if (result.status !== "executed" || result.executor.id !== owner
                || (p.generate_client_secret ? typeof secret !== "string" || !/^ri_client_[A-Za-z0-9_-]{1,256}$/.test(secret) : secret !== null)) throw { status: 502 };
          }
          // Do not display a credential to a session switched during the POST.
          if (!await session() || !active()) return;
          row = result; decision = null; rejected = false;
          busy = false; draw(); showSecret(secret, epoch === secretEpoch);
        } catch (e) {
          if (!active()) return;
          if (e.status && e.status < 500) decision = null;
          rejected = [403, 409, 428].includes(e.status); error(e);
        } finally { secret = null; busy = false; if (active()) update(); }
      }
    }
    queueMicrotask(() => { if (active()) reload(); });
    return { node: root, crumb: "Review creation" };
  }
  window.RiAuthClientCreationReview = Object.freeze({ view, reset, eraseSecrets });
})();
