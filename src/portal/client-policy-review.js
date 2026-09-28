"use strict";
// Presentation only. The existing reviewed policy service remains the sole writer.
(() => {
  const route = (id = "") => `#/client-policy-review${id ? `/${encodeURIComponent(id)}` : ""}`;
  const idPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  const name = (v) => typeof v === "string" && /^[A-Za-z0-9_.@-]{1,64}$/.test(v) && ![".", ".."].includes(v);
  const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
  const policy = (v) => v && typeof v.require_mfa === "boolean" && Array.isArray(v.allowed_groups)
    && v.allowed_groups.length <= 64 && v.allowed_groups.every(name) && new Set(v.allowed_groups).size === v.allowed_groups.length
    && Object.keys(v).length === 2;
  const content = (client) => ({ allowed_groups: [...client.allowed_groups].sort(), require_mfa: client.require_mfa });
  function checkedChange(value, expectedId) {
    const p = value?.proposal;
    const text = (v) => typeof v === "string" && v.length > 0 && v.length <= 256;
    const number = (v) => Number.isSafeInteger(v) && v >= 0;
    const time = (v) => number(v) && v < 8640000000000;
    const authority = (v) => text(v?.id) && number(v.epoch);
    if (!p || !idPattern.test(p.id) || expectedId && p.id !== expectedId || !text(value.digest)
        || !["pending", "approved", "executed", "cancelled"].includes(value.status)
        || !name(p.client_id) || p.resource !== `client/${p.client_id}/access-policy` || !authority(p.author)
        || !policy(p.before) || !policy(p.after) || !number(p.base_revision)
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
  function unavailable(client) {
    if (!client) return "The application is unavailable. Refresh to load its current state.";
    if (client.service) return "Service clients do not have human group or MFA access policies.";
    // Configuration may be retained in an edition without its provider. Do not
    // offer a write the running build cannot support. Runtime validation is server-owned.
    for (const [setting, feature] of [["saml", "saml.idp_signed_browser_sso"], ["proxy", "proxy.forward_auth_sso"],
      ["ldap", "directory.ldap_provider"], ["radius", "radius.pap"]]) {
      if (client.settings?.[setting] && !RiAuthCapabilities.compiled(feature)) return "This application's provider is unavailable in the running build.";
    }
    if (client.settings?.require_device_trust && !RiAuthCapabilities.usable("identity.device_trust")) return "Device trust must be available before this application's policy can change.";
    return null;
  }
  let generation = 0, clock, savedDraft = null, decision = null;
  function reset(forget = false) {
    generation += 1; clearInterval(clock);
    if (forget) { savedDraft = null; decision = null; }
  }
  function message(e) {
    if (!e.status || e.status >= 500) return "The response was lost or incomplete. The action may have completed. Refresh or recover the same request before taking another action.";
    if (e.status === 403) return "This action is not allowed. The author, every reviewer and executor must be distinct, currently authorized full administrators. Refresh and check the participants.";
    if (e.status === 404) return "This application or change is unavailable. Check the exact client or change ID.";
    if ([409, 428].includes(e.status)) return "The server refused this change because its content, authority, dependencies or status changed. Refresh to inspect it; stale proposals need a new review.";
    if (e.status === 429) return "Too many requests. Wait a minute before retrying.";
    return "The server rejected this policy. Use at most 64 distinct existing group names and a supported human sign-in application. The policy must change; all other settings and credentials remain unchanged.";
  }
  function view({ id, api, h, me, users, clients, identityChanged, sessionLost }) {
    const run = ++generation, owner = me?.user?.id;
    const sessionMarker = typeof me?.session_marker === "string" && /^[A-Za-z0-9_-]{43}$/.test(me.session_marker) ? me.session_marker : null;
    const selected = id?.startsWith("client:") ? id.slice(7) : "";
    if (selected) id = null;
    if (!sessionMarker || savedDraft?.sessionMarker !== sessionMarker || savedDraft?.owner !== owner || !me?.user?.admin) savedDraft = null;
    if (!sessionMarker || decision?.sessionMarker !== sessionMarker || decision?.owner !== owner || !me?.user?.admin) decision = null;
    if (selected && savedDraft?.client_id !== selected && !savedDraft?.pending) savedDraft = null;
    const root = h("div", { class: "grant-review client-policy-review" });
    const active = () => run === generation && root.isConnected;
    const status = h("p", { class: "form-error", role: "alert", tabindex: "-1", hidden: true });
    root.append(h("section", { class: "page-heading" }, h("div", {}, h("p", { class: "eyebrow" }, "APPLICATIONS"),
      h("h1", { tabindex: "-1" }, id ? "Review application access policy" : "Reviewed access policies"),
      h("p", { class: "page-description" }, "Stage exact group and MFA rules. A second administrator reviews them and a third applies them once. No credential is issued."))), status);
    const localError = (text) => { status.textContent = text; status.hidden = false; status.focus(); };
    const error = (e) => { if (active()) { if (e.status === 401) sessionLost(401); else localError(message(e)); } };
    const button = (text, click, attrs = {}) => h("button", { type: "button", class: "button secondary", onclick: click, ...attrs }, text);
    const field = (label, input, hint) => h("div", { class: "field" }, h("label", { for: input.id }, label), input,
      hint ? h("p", { class: "field-hint" }, hint) : null);
    const facts = (rows) => h("dl", { class: "facts grant-facts" }, rows.map(([label, value]) => [h("dt", {}, label), h("dd", {}, value)]));
    const actor = (id) => h("span", { class: "grant-identity" }, h("strong", {}, users.find((u) => u.id === id)?.username || "Administrator"), h("code", {}, id));
    const date = (at) => h("time", { datetime: new Date(at * 1000).toISOString() }, new Date(at * 1000).toLocaleString(undefined, { timeZoneName: "short" }));
    const policyCard = (title, value, id) => h("section", { class: "admin-card" }, h("h2", {}, title),
      facts([["Allowed groups", value.allowed_groups.join(", ") || "No group restriction"], ["Require MFA", value.require_mfa ? "Yes" : "No"]]),
      h("pre", { class: "creation-content", id }, JSON.stringify(value, null, 2)));
    async function session() {
      await RiAuthCapabilities.refresh();
      if (!active()) return null;
      const live = await api("GET", "admin/session");
      if (!active()) return null;
      if (live?.user?.id !== owner || live?.session_marker !== sessionMarker) { identityChanged(); return null; }
      if (!live.user.admin) { sessionLost(403); return null; }
      return live;
    }
    if (!me?.user?.admin || !sessionMarker) {
      root.append(h("p", { class: "notice" }, "A verified browser session with a full administrator account is required to review access policies."));
      return { node: root };
    }
    const otherDecision = decision && decision.id !== id;
    if (otherDecision) root.append(h("p", { class: "notice warn-notice" }, "An earlier action is unconfirmed. ", h("a", { href: route(decision.id) }, "Open that change to resolve it first.")));
    if (!id) {
      const lookup = h("input", { id: "policy-lookup", maxlength: "36", required: true, autocomplete: "off" });
      const open = h("form", { class: "inline-form" }, field("Change ID", lookup), h("button", { type: "submit", class: "button secondary" }, "Open change"));
      open.addEventListener("submit", (event) => {
        event.preventDefault();
        if (!idPattern.test(lookup.value.trim())) { localError("Enter the complete change ID from the author."); return; }
        location.hash = route(lookup.value.trim());
      });
      const chosen = savedDraft?.client_id || selected;
      const select = h("select", { id: "policy-client" }, h("option", { value: "" }, "Choose an application"),
        clients.map((c) => h("option", { value: c.client_id }, `${c.name} · ${c.client_id}`)),
        chosen && !clients.some((c) => c.client_id === chosen) ? h("option", { value: chosen }, chosen) : null);
      select.value = chosen;
      const editor = h("div", {}), hint = h("p", { class: "field-hint" });
      let loading = false;
      const load = button("Load current policy", async () => {
        if (loading || savedDraft?.pending || !name(select.value)) return;
        const clientId = select.value;
        loading = true; updateSelection();
        try {
          const live = await session(); if (!live) return;
          const current = (await api("GET", "admin/clients")).find((c) => c.client_id === clientId);
          const verified = await session(); if (!verified) return;
          if (live.revision !== verified.revision) throw { status: 409 };
          const reason = unavailable(current); if (reason) { localError(reason); return; }
          const before = content(current); if (!policy(before)) throw { status: 502 };
          savedDraft = { owner, sessionMarker, client_id: clientId, client: current, before, revision: live.revision,
            groups: before.allowed_groups.join("\n"), mfa: before.require_mfa, pending: null, blocked: false };
          status.hidden = true; draft(savedDraft);
        } catch (e) { error(e); }
        finally { loading = false; if (active()) updateSelection(); }
      });
      function updateSelection() {
        const reason = unavailable(clients.find((c) => c.client_id === select.value));
        hint.textContent = reason || "Loading current policy replaces the draft and its revision. Refresh preserves the existing draft until you explicitly reload it.";
        select.disabled = loading || !!savedDraft?.pending;
        load.disabled = loading || !!savedDraft?.pending || !!reason || !!otherDecision;
      }
      select.addEventListener("change", () => { savedDraft = null; editor.replaceChildren(); updateSelection(); });
      root.append(open, h("section", { class: "admin-card" }, field("Application", select), hint, load), editor);
      function draft(record) {
        const groups = h("textarea", { id: "policy-groups", rows: "4", maxlength: "8192", autocomplete: "off", spellcheck: "false", value: record.groups });
        const mfa = h("input", { id: "policy-mfa", type: "checkbox", checked: record.mfa });
        const ack = h("input", { id: "policy-stage-ack", type: "checkbox" });
        const preview = h("pre", { id: "policy-preview", class: "creation-content" });
        const stage = h("button", { type: "submit", class: "button primary" }, "Stage exact policy");
        const notice = h("p", { class: "field-hint" });
        const form = h("form", { class: "admin-form admin-card", novalidate: true }, h("h2", {}, "After · proposed access policy"),
          field("Allowed groups", groups, "One exact group name per line. Empty removes the group restriction; other policies still apply."),
          h("label", { class: "checkbox", for: mfa.id }, mfa, "Require MFA"), preview,
          h("p", { class: "field-hint", id: "policy-draft-revision" }, `Based on management revision ${record.revision}. Refresh keeps this draft and its original revision.`), notice,
          h("label", { class: "checkbox", for: ack.id }, ack, "I checked both complete policy fields, including removed restrictions."),
          h("div", { class: "form-actions" }, stage));
        let busy = false;
        const body = () => ({ allowed_groups: record.groups.split("\n").map((s) => s.trim()).filter(Boolean).sort(), require_mfa: record.mfa });
        function update() {
          const locked = busy || !!record.pending;
          groups.disabled = mfa.disabled = locked;
          updateSelection(); select.disabled ||= busy; load.disabled ||= busy;
          ack.disabled = locked || record.blocked;
          stage.disabled = busy || record.blocked || otherDecision || !record.pending && (!ack.checked || same(body(), record.before));
          stage.textContent = record.pending ? "Retry same staging request" : "Stage exact policy";
          preview.textContent = JSON.stringify(record.pending?.body || body(), null, 2);
          notice.textContent = record.pending ? "Staging is unconfirmed. Content, revision and request key are locked. Retry the same request; a page reload discards this recovery intent."
            : "Both tightening and removing access restrictions require independent review. Other settings and client credentials are preserved.";
        }
        groups.addEventListener("input", () => { record.groups = groups.value; ack.checked = false; update(); });
        mfa.addEventListener("change", () => { record.mfa = mfa.checked; ack.checked = false; update(); });
        ack.addEventListener("change", update);
        editor.replaceChildren(policyCard("Before · loaded snapshot", record.before, "policy-draft-before"), form); update();
        form.addEventListener("submit", async (event) => {
          event.preventDefault();
          if (stage.disabled || savedDraft !== record || loading) return;
          const after = body(); if (!policy(after)) { localError("Enter at most 64 distinct, exact group names, one per line."); return; }
          const retry = !!record.pending;
          record.pending ||= { body: after, revision: record.revision, key: crypto.randomUUID() };
          busy = true; status.hidden = true; update();
          try {
            if (!await session()) return;
            if (!retry && unavailable(record.client)) throw { status: 409 };
            const change = checkedChange(await api("POST", `admin/clients/${encodeURIComponent(record.client_id)}/policy-changes`, record.pending.body, record.pending));
            if (!await session()) return;
            if (change.proposal.author.id !== owner || change.proposal.client_id !== record.client_id
                || !same(content(change.proposal.before), record.before) || !same(content(change.proposal.after), record.pending.body)
                || change.proposal.base_revision !== record.revision) throw { status: 502 };
            savedDraft = null; location.hash = route(change.proposal.id);
          } catch (e) {
            if (!active()) return;
            if (e.status && e.status < 500) { record.pending = null; record.blocked = [403, 409, 428].includes(e.status); }
            error(e);
          } finally { busy = false; if (active()) update(); }
        });
      }
      if (savedDraft) draft(savedDraft);
      updateSelection();
      if (chosen && !savedDraft) queueMicrotask(() => { if (active()) load.click(); });
      return { node: root };
    }
    if (!idPattern.test(id)) {
      root.append(h("p", { class: "notice" }, "This change ID is invalid."), h("a", { href: route() }, "Open another change"));
      return { node: root, crumb: "Invalid change" };
    }
    const endpoint = `admin/client-policy-changes/${encodeURIComponent(id)}`;
    const refresh = button("Refresh change", () => reload());
    const detail = h("div", { class: "grant-detail", "aria-busy": "true" });
    root.append(h("div", { class: "form-actions" }, refresh, h("a", { class: "button secondary", href: route() }, "Stage or open another change")), detail);
    let row, currentSession, busy = false, rejected = false, read = 0;
    const pending = () => decision?.id === id ? decision : null;
    async function reload() {
      if (busy) return;
      clearInterval(clock); const request = ++read;
      refresh.disabled = true; detail.setAttribute("aria-busy", "true"); detail.replaceChildren(h("p", {}, "Loading immutable proposal…"));
      try {
        const live = await session(); if (!live) return;
        const result = checkedChange(await api("GET", endpoint), id);
        const verified = await session(); if (!verified || request !== read) return;
        if (pending() && (!same(result.proposal, decision.proposal) || result.digest !== decision.digest)) throw { status: 502 };
        row = result; currentSession = verified; status.hidden = true; draw();
      } catch (e) { if (active() && request === read) { detail.replaceChildren(); error(e); } }
      finally { if (active() && request === read) { refresh.disabled = false; detail.removeAttribute("aria-busy"); } }
    }
    function state() {
      if (pending()) return ["Outcome unknown", "warn", "Recover the original request to confirm its result. Its content, revision and request key cannot change."];
      if (row.status === "executed") return ["Executed", "ok", "The exact policy was applied once. No secret was issued or changed."];
      if (row.status === "cancelled") return ["Cancelled", "muted", "This change cannot be approved or executed."];
      if (row.proposal.expires_at <= Date.now() / 1000) return ["Expired", "warn", "The approval window ended. Stage a new proposal."];
      if (rejected || row.proposal.base_revision !== currentSession.revision) return ["Stale", "warn", "Authority, policy, dependencies or revision changed. Stage a new proposal for a new review."];
      return row.status === "approved" ? ["Approved", "info", "An independent administrator may apply this exact policy once."]
        : ["Awaiting review", "info", "An administrator other than the author must approve both exact policy fields."];
    }
    function draw() {
      const p = row.proposal, client = clients.find((c) => c.client_id === p.client_id);
      const label = h("strong", { class: "badge", id: "policy-status", role: "status" }), explanation = h("p", { class: "field-hint" });
      const ack = h("input", { id: "policy-review-ack", type: "checkbox" });
      const approve = button("Approve exact policy", () => act("approve"), { class: "button primary" });
      const execute = button("Apply policy once", () => act("execute"), { class: "button primary" });
      const cancel = button("Cancel change", () => act("cancel"));
      const recover = button("Recover same request", () => act(pending()?.action, true));
      const hint = h("p", { class: "field-hint" }), capability = h("p", { class: "notice" });
      function update() {
        if (!active()) { clearInterval(clock); return; }
        const [name, tone, text] = state(), reason = unavailable(client);
        label.textContent = name; label.className = `badge ${tone}`; explanation.textContent = text;
        const ended = ["Executed", "Cancelled", "Expired"].includes(name), author = p.author.id === owner;
        const reviewer = row.approvals.some((a) => a.reviewer.id === owner);
        const available = !busy && !pending() && !otherDecision && !ended && name !== "Stale" && !reason;
        approve.disabled = !available || !ack.checked || author || reviewer || row.approvals.length >= 8;
        execute.disabled = !available || !ack.checked || author || reviewer || row.status !== "approved";
        cancel.disabled = busy || !!pending() || !!otherDecision || ended;
        ack.disabled = !available; refresh.disabled = busy;
        recover.hidden = !pending(); recover.disabled = busy;
        capability.hidden = !reason; capability.textContent = reason || "";
        hint.textContent = author ? "You are the author. Other administrators must review and execute."
          : reviewer ? "You approved this policy. Another administrator must execute."
          : "The author, all reviewers and executor must be distinct. The server rechecks live authority and dependencies.";
        if (currentSession.expires_at <= Date.now() / 1000) sessionLost(401);
      }
      ack.addEventListener("change", update);
      const reviewers = row.approvals.length ? h("ul", { class: "plain-list" }, row.approvals.map((a) => h("li", {}, actor(a.reviewer.id), " · ", date(a.at)))) : "No reviewers yet";
      detail.replaceChildren(
        h("section", { class: "admin-card" }, h("h2", {}, `Access policy · ${p.client_id}`), label, explanation,
          facts([["Resource", h("code", {}, p.resource)], ["Author", actor(p.author.id)], ["Reviewers", reviewers],
            ["Executor", row.executor ? actor(row.executor.id) : "Not executed"], ["Created", date(p.created_at)], ["Expires", date(p.expires_at)],
            ...(row.executed_at ? [["Executed", date(row.executed_at)]] : [])])),
        h("div", { class: "grant-comparison" }, policyCard("Before · exact policy", p.before, "policy-before"), policyCard("After · exact policy", p.after, "policy-after")),
        h("p", { class: "notice" }, "Empty allowed groups remove the group restriction. Require MFA: No removes this client's MFA requirement. Other access rules still apply. Credentials and all other settings are preserved."),
        h("section", { class: "admin-card" }, h("h2", {}, "Immutable approval binding"),
          facts([["Change ID", h("code", { id: "policy-id" }, p.id)], ["Canonical digest", h("code", { id: "policy-digest" }, row.digest)],
            ["Management revision", String(p.base_revision)], ["Resource dependencies", h("code", {}, p.resource_revision)], ["Policy dependencies", h("code", {}, p.policy_revision)]]),
          h("p", { class: "field-hint" }, "Approval binds both fields, dependencies and expiry. Freshness is rechecked by the server. Changed intent requires a new proposal."),
          field("Review link", h("input", { id: "policy-review-link", readOnly: true, value: new URL(route(id), location.href).href }), "Share with the next administrator; the link contains no credential.")),
        h("section", { class: "admin-card" }, h("h2", {}, "Your action"), hint, capability,
          h("label", { class: "checkbox", for: ack.id }, ack, "I checked both policy fields, digest and dependencies."),
          h("div", { class: "form-actions" }, approve, execute, cancel, recover),
          h("a", { class: "button secondary", href: `#/applications/${encodeURIComponent(p.client_id)}` }, "Go to the application")));
      clearInterval(clock); clock = setInterval(update, 1000); update();
      async function act(action, retry = false) {
        if (busy || (retry ? recover : { approve, execute, cancel }[action]).disabled || retry && !pending()) return;
        busy = true; status.hidden = true; update();
        try {
          const live = await session(); if (!live) return;
          currentSession = live;
          if (!retry && action !== "cancel" && (["Stale", "Expired"].includes(state()[0]) || unavailable(client))) return;
          decision ||= { owner, sessionMarker, id, action, digest: row.digest, proposal: row.proposal, revision: live.revision, key: crypto.randomUUID() };
          const request = decision;
          const result = checkedChange(await api("POST", `${endpoint}/${request.action}`, { digest: request.digest }, request), id);
          if (result.digest !== request.digest || !same(result.proposal, request.proposal)
              || result.status !== { approve: "approved", execute: "executed", cancel: "cancelled" }[action]
              || action === "execute" && result.executor.id !== owner
              || action === "approve" && !result.approvals.some((a) => a.reviewer.id === owner)) throw { status: 502 };
          if (!await session()) return;
          row = result; decision = null; rejected = false; busy = false; draw();
        } catch (e) {
          if (!active()) return;
          if (e.status && e.status < 500) decision = null;
          rejected = [403, 409, 428].includes(e.status); error(e);
        } finally { busy = false; if (active()) update(); }
      }
    }
    queueMicrotask(() => { if (active()) reload(); });
    return { node: root, crumb: "Review policy" };
  }
  window.RiAuthClientPolicyReview = Object.freeze({ view, reset, unavailable });
})();
