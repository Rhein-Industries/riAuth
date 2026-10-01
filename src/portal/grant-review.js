"use strict";
// Browser adapter only. The existing grant service owns policy, authority,
// canonical content, one-time consumption and audit for every transport.
(() => {
  const ROLES = {
    help_desk: "Help desk", application_owner: "Application owner", directory_operator: "Directory operator",
    auditor: "Auditor", security_administrator: "Security administrator",
  };
  // The service decides what needs review: a change that adds, removes or alters a
  // directory-operator or security-administrator grant is staged; every other
  // change is the immediate grant write. This only picks the endpoint, and the
  // server still refuses a wrong choice with its own error.
  const PRIVILEGED = new Set(["directory_operator", "security_administrator"]);
  const privileged = (grants) => grants.filter((grant) => PRIVILEGED.has(grant.role)).map((grant) => `${grant.role}\u0000${grant.scope}`).sort();
  const route = (id = "") => `#/grant-review${id ? `/${encodeURIComponent(id)}` : ""}`;
  const idPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  const namePattern = /^[A-Za-z0-9_.@-]{1,64}$/;
  function checkedChange(value, expectedId) {
    const p = value?.proposal;
    const text = (v) => typeof v === "string" && v.length > 0 && v.length <= 256;
    const authority = (v) => text(v?.id) && Number.isSafeInteger(v.epoch);
    const grants = (v) => Array.isArray(v) && v.length <= 32 && v.every((g) => text(g.role) && text(g.scope) && text(g.target_id));
    if (!p || !idPattern.test(p.id) || expectedId && p.id !== expectedId || !text(value.digest)
        || !["pending", "approved", "executed", "cancelled"].includes(value.status)
        || !text(p.username) || !text(p.resource) || !text(p.holder_id) || !authority(p.author)
        || !grants(p.before) || !grants(p.after) || !Number.isSafeInteger(p.base_revision)
        || !text(p.resource_revision) || !text(p.policy_revision)
        || !Number.isSafeInteger(p.created_at) || !Number.isSafeInteger(p.expires_at)
        || !Array.isArray(value.approvals) || value.approvals.length > 8
        || !value.approvals.every((a) => authority(a.reviewer) && text(a.digest) && Number.isSafeInteger(a.at))
        || value.executor && !authority(value.executor)
        || value.executed_at !== null && !Number.isSafeInteger(value.executed_at)) {
      throw { status: 502 }; // A malformed response never enables an action.
    }
    return value;
  }
  let generation = 0, clock;
  function reset() { generation += 1; clearInterval(clock); }
  function message(error) {
    if (!error.status || error.status >= 500) return "The response was lost or incomplete. The change may have been recorded. Refresh to check its status before taking another action.";
    if (error.status === 403) return "This action is not allowed. The author, each reviewer and executor must be distinct, currently authorized administrators. Refresh and check the participants.";
    if (error.status === 404) return "This change or recipient is unavailable. It may have expired or been removed. Check the ID; no proposal can be recovered from this page.";
    if (error.status === 409) return "The server refused this change because its content, authority, dependencies or status no longer permits the action. Refresh to inspect it; stale proposals need a new review.";
    if (error.status === 429) return "Too many requests. Wait a minute, then refresh before trying again.";
    return "The request was rejected. Check the exact recipient, roles and target scopes, then reload the current grants.";
  }
  function view({ id, api, h, me, users, identityChanged, sessionLost }) {
    const run = ++generation;
    const owner = me?.user?.id;
    const root = h("div", { class: "grant-review" });
    const active = () => run === generation && root.isConnected;
    const title = h("h1", { tabindex: "-1" }, id ? "Review grant change" : "Reviewed grants");
    const status = h("p", { class: "form-error", role: "alert", tabindex: "-1", hidden: true });
    root.append(h("section", { class: "page-heading" }, h("div", {}, h("p", { class: "eyebrow" }, "SECURITY"), title,
      h("p", { class: "page-description" }, "Change a person's delegated grants. Help desk, application owner and auditor changes apply immediately. Directory-operator and security-administrator changes are staged: a separate administrator reviews and another executes."))), status);
    const localError = (text) => { status.textContent = text; status.hidden = false; status.focus(); };
    const error = (e) => {
      if (!active()) return;
      if (e.status === 401) sessionLost(401); else localError(message(e));
    };
    const button = (text, click, attrs = {}) => h("button", { type: "button", class: "button secondary", onclick: click, ...attrs }, text);
    const field = (label, input, hint) => h("div", { class: "field" }, h("label", { for: input.id }, label), input,
      hint ? h("p", { class: "field-hint" }, hint) : null);
    const key = () => crypto.randomUUID();
    const actor = (value) => {
      const user = users.find((user) => user.id === value);
      return h("span", { class: "grant-identity" }, user ? h("strong", {}, user.username) : null, h("code", {}, value));
    };
    const date = (seconds) => h("time", { datetime: new Date(seconds * 1000).toISOString(), title: new Date(seconds * 1000).toISOString() }, new Date(seconds * 1000).toLocaleString(undefined, { timeZoneName: "short" }));
    const facts = (rows) => h("dl", { class: "facts grant-facts" }, rows.map(([label, value]) => [h("dt", {}, label), h("dd", {}, value)]));
    function grants(title, rows) {
      return h("section", { class: "admin-card" }, h("h2", {}, title), rows.length
        ? h("ul", { class: "grant-list" }, rows.map((grant) => h("li", {},
          h("strong", {}, ROLES[grant.role] || grant.role), h("code", {}, grant.scope),
          h("span", { class: "field-hint" }, "Bound target: ", h("code", {}, grant.target_id)))))
        : h("p", { class: "field-hint" }, "No delegated grants."));
    }
    // A different account in another tab must inspect the proposal afresh. A
    // cookie change never silently makes an old form act as the new account.
    async function session() {
      const current = await api("GET", "admin/session");
      if (!active()) return null;
      if (current.user.id !== owner) { identityChanged(); return null; }
      if (!current.user.admin) { sessionLost(403); return null; }
      return current;
    }
    if (!me?.user?.admin) {
      root.append(h("p", { class: "notice" }, "A full administrator account is required to review delegated grants."));
      return { node: root };
    }

    if (!id) {
      const lookup = h("input", { id: "grant-change-id", required: true, maxlength: "36", autocomplete: "off" });
      const open = h("form", { class: "inline-form" }, field("Change ID", lookup),
        h("button", { type: "submit", class: "button secondary" }, "Open change"));
      open.addEventListener("submit", (event) => {
        event.preventDefault();
        if (!idPattern.test(lookup.value.trim())) { localError("Enter the complete change ID from the author."); return; }
        location.hash = route(lookup.value.trim());
      });
      root.append(open);
      const recipient = h("input", { id: "grant-recipient", required: true, maxlength: "64", autocomplete: "off", list: "grant-recipients" });
      const load = h("button", { type: "submit", class: "button secondary" }, "Load current grants");
      const select = h("form", { class: "inline-form" }, field("Recipient username", recipient),
        h("datalist", { id: "grant-recipients" }, users.filter((u) => !u.admin && u.enabled && u.id !== owner).map((u) => h("option", { value: u.username }))), load);
      const editor = h("div");
      root.append(h("h2", {}, "Change a recipient's grants"), h("p", { class: "field-hint" }, "Load one recipient's current grants, then give the complete set they should retain. Removing every row revokes all of them."), select, editor);
      let loading = false, selection = 0;
      recipient.addEventListener("input", () => { selection += 1; editor.replaceChildren(); });
      select.addEventListener("submit", async (event) => {
        event.preventDefault();
        if (loading) return;
        const username = recipient.value.trim();
        if (!namePattern.test(username) || [".", ".."].includes(username)) { localError("Enter one exact recipient username."); return; }
        const chosen = ++selection;
        loading = true; load.disabled = true; editor.replaceChildren(); status.hidden = true;
        try {
          const current = await session();
          if (!current) return;
          const result = await api("GET", `admin/users/${encodeURIComponent(username)}/delegated-grants`);
          if (active() && chosen === selection) draft(username, result.grants, current.revision, chosen);
        } catch (e) { error(e); }
        finally { loading = false; load.disabled = false; }
      });
      function draft(username, before, revision, chosen) {
        const rows = h("div", { class: "grant-rows" });
        const acknowledgement = h("input", { id: "grant-stage-ack", type: "checkbox", required: true });
        const stage = h("button", { type: "submit", class: "button primary" }, "Stage exact change");
        const modeHint = h("p", { class: "field-hint", id: "grant-mode-hint" });
        const held = JSON.stringify(privileged(before));
        const add = button("Add grant", () => addRow());
        const empty = h("p", { class: "notice warn-notice" }, "No grants will remain. Execution will revoke all delegated grants for this recipient.");
        const form = h("form", { class: "admin-form admin-card" }, h("h2", {}, `Proposed grants for ${username}`), rows, empty, add,
          h("label", { class: "checkbox", for: acknowledgement.id }, acknowledgement, "I checked the complete replacement, including every grant being removed."),
          modeHint, h("div", { class: "form-actions" }, stage));
        let next = 0, pending = null, busy = false, uncertain = false;
        const proposed = () => [...rows.children].map((row) => ({ role: row.querySelector("select").value, scope: row.querySelector("input").value.trim() }));
        const needsReview = () => JSON.stringify(privileged(proposed())) !== held;
        const forgetSaved = () => editor.querySelector("#grant-saved")?.remove();
        const changed = () => { acknowledgement.checked = false; pending = null; forgetSaved(); update(); };
        const update = () => {
          empty.hidden = !!rows.children.length; add.disabled = busy || uncertain || rows.children.length >= 32; stage.disabled = busy || (!uncertain && !acknowledgement.checked);
          const staged = pending ? pending.mode === "stage" : needsReview();
          empty.textContent = staged ? "No grants will remain. Execution will revoke all delegated grants for this recipient."
            : "No grants will remain. Saving will revoke all delegated grants for this recipient.";
          stage.textContent = uncertain ? (staged ? "Retry same staging request" : "Retry same request") : (staged ? "Stage exact change" : "Apply change now");
          modeHint.textContent = staged ? "This change touches directory-operator or security-administrator grants. It is staged for an independent reviewer and a separate executor."
            : "This change touches only help desk, application owner and auditor grants. It takes effect immediately and is audited.";
        };
        function addRow(grant = { role: "", scope: "" }) {
          if (rows.children.length >= 32 || busy || uncertain) return;
          const n = ++next;
          const role = h("select", { id: `grant-role-${n}`, required: true }, h("option", { value: "" }, "Choose a role"),
            Object.entries(ROLES).map(([value, label]) => h("option", { value, selected: value === grant.role }, label)));
          const scope = h("input", { id: `grant-scope-${n}`, value: grant.scope, required: true, maxlength: "80", autocomplete: "off", spellcheck: "false", placeholder: "key/signing or directory/staff" });
          const remove = button("Remove grant", () => { row.remove(); changed(); }, { "aria-label": `Remove grant ${n}` });
          const row = h("div", { class: "grant-row" }, field("Role", role), field("Exact target scope", scope), remove);
          role.addEventListener("change", changed); scope.addEventListener("input", changed);
          rows.append(row); changed();
        }
        for (const grant of before) addRow(grant);
        acknowledgement.addEventListener("change", update); update();
        editor.replaceChildren(grants("Current grants", before), form);
        form.addEventListener("submit", async (event) => {
          event.preventDefault();
          if (busy || (!uncertain && !acknowledgement.checked) || chosen !== selection) return;
          const after = proposed();
          if (!pending) pending = { body: after, key: key(), mode: needsReview() ? "stage" : "set" };
          busy = true; status.hidden = true; forgetSaved();
          for (const control of form.querySelectorAll("input, select, button")) control.disabled = true;
          recipient.disabled = true; load.disabled = true;
          const immediate = pending.mode === "set";
          const target = `admin/users/${encodeURIComponent(username)}/delegated-grants`;
          try {
            const current = await session();
            if (!current) return;
            if (immediate) {
              const saved = await api("PUT", target, pending.body, { revision, key: pending.key });
              if (!Array.isArray(saved?.grants) || saved.username !== username) throw { status: 502 };
              const latest = await session();
              const result = latest && await api("GET", target);
              if (active() && result && chosen === selection) {
                draft(username, result.grants, latest.revision, chosen);
                const done = h("p", { class: "notice", id: "grant-saved", role: "status", tabindex: "-1" }, `Grants saved for ${username}. The change took effect immediately and was audited.`);
                editor.prepend(done); done.focus();
              }
              return;
            }
            const change = checkedChange(await api("POST", `${target}/changes`, pending.body, { revision, key: pending.key }));
            if (active()) location.hash = route(change.proposal.id);
          } catch (e) {
            uncertain = !e.status || e.status >= 500;
            // The server's own refusal of an immediate write is shown as it was sent.
            if (immediate && (e.status === 400 || e.status === 409) && e.message) localError(e.message); else error(e);
            if (active() && uncertain) localError(immediate ? "The response was lost. Retry the same request to learn its outcome; the same request key applies it at most once. The proposed grants are locked until it is resolved."
              : "The staging response was lost. Retry the same staging request to retrieve its result. The proposed grants are locked until it is resolved.");
          } finally {
            busy = false;
            for (const control of form.querySelectorAll("input, select, button")) control.disabled = uncertain;
            recipient.disabled = uncertain; load.disabled = uncertain;
            update();
          }
        });
      }
      return { node: root };
    }

    if (!idPattern.test(id)) {
      root.append(h("p", { class: "notice" }, "This change ID is invalid."), h("a", { href: route(), class: "button secondary" }, "Open another change"));
      return { node: root, crumb: "Invalid change" };
    }
    const refresh = button("Refresh change", () => reload());
    root.append(h("div", { class: "form-actions" }, refresh, h("a", { class: "button secondary", href: route() }, "Stage or open another change")));
    const detail = h("div", { class: "grant-detail", "aria-busy": "true" }, h("p", {}, "Loading immutable proposal…"));
    root.append(detail);
    let row, currentSession, busy = false, uncertain = false, rejected = false, read = 0;
    const endpoint = `admin/delegated-grant-changes/${encodeURIComponent(id)}`;
    async function reload() {
      if (busy) return;
      const request = ++read;
      refresh.disabled = true; detail.setAttribute("aria-busy", "true");
      clearInterval(clock);
      // Old action controls cannot be used while a refreshed status is pending.
      detail.replaceChildren(h("p", {}, "Loading immutable proposal…"));
      try {
        const live = await session();
        if (!live) return;
        const result = checkedChange(await api("GET", endpoint), id);
        if (!active() || request !== read) return;
        row = result; currentSession = live; uncertain = false; status.hidden = true;
        draw();
      } catch (e) { if (active() && request === read) { detail.replaceChildren(); error(e); } }
      finally { if (active() && request === read) { refresh.disabled = false; detail.removeAttribute("aria-busy"); } }
    }
    function state() {
      if (row.status === "executed") return ["Executed", "ok", "This change was consumed once. It cannot execute again."];
      if (row.status === "cancelled") return ["Cancelled", "muted", "This change cannot be approved or executed."];
      if (row.proposal.expires_at <= Date.now() / 1000) return ["Expired", "warn", "The approval window ended. Stage a new proposal for a new review."];
      if (row.proposal.base_revision !== currentSession.revision || rejected) return ["Stale", "warn", "A dependency, authority or management revision changed. Stage a new proposal; this approval cannot be reused."];
      if (uncertain) return ["Outcome unknown", "warn", "The response was lost. Refresh this change to confirm whether the action completed."];
      if (row.status === "approved") return ["Approved", "info", "An independent administrator can execute this exact change once."];
      return ["Awaiting review", "info", "At least one administrator other than the author must approve this exact change."];
    }
    function draw() {
      const proposal = row.proposal;
      const label = h("strong", { class: "badge", id: "grant-status", role: "status" });
      const explanation = h("p", { class: "field-hint", id: "grant-status-detail" });
      const approvalList = row.approvals.length ? h("ul", { class: "plain-list" }, row.approvals.map((a) => h("li", {}, actor(a.reviewer.id), " · ", date(a.at)))) : "No reviewers yet";
      const link = h("input", { id: "grant-review-link", readOnly: true, value: new URL(route(id), location.href).href });
      const ack = h("input", { id: "grant-review-ack", type: "checkbox" });
      const approve = button("Approve exact change", () => act("approve"), { class: "button primary" });
      const execute = button("Execute once", () => act("execute"), { class: "button primary" });
      const cancel = button("Cancel change", () => act("cancel"));
      const actionHint = h("p", { class: "field-hint" });
      const update = () => {
        if (!active()) { clearInterval(clock); return; }
        const [name, tone, text] = state();
        label.textContent = name; label.className = `badge ${tone}`; explanation.textContent = text;
        const ended = ["Executed", "Cancelled", "Expired"].includes(name);
        const available = !busy && !uncertain && !ended && name !== "Stale";
        const author = proposal.author.id === owner;
        const reviewer = row.approvals.some((a) => a.reviewer.id === owner);
        const recipient = proposal.holder_id === owner;
        approve.disabled = !available || !ack.checked || author || reviewer || recipient || row.approvals.length >= 8;
        execute.disabled = !available || !ack.checked || author || reviewer || recipient || row.status !== "approved";
        cancel.disabled = busy || uncertain || ended;
        ack.disabled = !available;
        refresh.disabled = busy;
        actionHint.textContent = author ? "You are the author. Other administrators must review and execute."
          : reviewer ? "You have approved this change. Another administrator must execute."
          : "The author, every reviewer and executor must be distinct. The server checks live authority and dependencies again for each action.";
        if (currentSession.expires_at <= Date.now() / 1000) sessionLost(401);
      };
      ack.addEventListener("change", update);
      detail.replaceChildren(...[
        h("section", { class: "admin-card" }, h("h2", {}, `Change for ${proposal.username}`), label, explanation,
          facts([["Resource", h("code", {}, proposal.resource)], ["Recipient ID", h("code", {}, proposal.holder_id)],
            ["Author", actor(proposal.author.id)], ["Reviewers", approvalList],
            ["Executor", row.executor ? actor(row.executor.id) : "Not executed"],
            ["Created", date(proposal.created_at)], ["Expires", date(proposal.expires_at)],
            ...(row.executed_at ? [["Executed", date(row.executed_at)]] : [])])),
        h("div", { class: "grant-comparison" }, grants("Before · exact grants", proposal.before), grants("After · exact grants", proposal.after)),
        !proposal.after.length ? h("p", { class: "notice warn-notice" }, "Execution will revoke all delegated grants for this recipient.") : null,
        h("section", { class: "admin-card" }, h("h2", {}, "Immutable approval binding"),
          facts([["Change ID", h("code", { id: "grant-id" }, proposal.id)], ["Canonical digest", h("code", { id: "grant-digest" }, row.digest)],
            ["Management revision", String(proposal.base_revision)], ["Resource dependencies", h("code", {}, proposal.resource_revision)],
            ["Policy dependencies", h("code", {}, proposal.policy_revision)]]),
          h("p", { class: "field-hint" }, "Approval covers only this content, these dependencies and this expiry. To change any grants, stage a new proposal."),
          field("Review link", link, "Share this link with the next administrator. It contains no credential and requires an authorized session.")),
        h("section", { class: "admin-card" }, h("h2", {}, "Your action"), actionHint,
          h("label", { class: "checkbox", for: ack.id }, ack, "I checked the exact grants, digest and dependencies."),
          h("div", { class: "form-actions" }, approve, execute, cancel))].filter(Boolean));
      clearInterval(clock); clock = setInterval(update, 1000); update();
      async function act(action) {
        if (busy || ({ approve, execute, cancel })[action].disabled) return;
        busy = true; status.hidden = true; update();
        const digest = row.digest;
        try {
          const live = await session();
          if (!live) return;
          currentSession = live;
          if (action !== "cancel" && ["Stale", "Expired"].includes(state()[0])) return;
          const result = checkedChange(await api("POST", `${endpoint}/${action}`, { digest }, { revision: live.revision, key: key() }), id);
          if (!active()) return;
          row = result; rejected = false;
        } catch (e) {
          if (!active()) return;
          uncertain = !e.status || e.status >= 500;
          rejected = e.status === 409 || e.status === 403;
          error(e);
        } finally { busy = false; if (active()) { draw(); } }
      }
    }
    // Start after the parent mounts the view, including tests with immediate responses.
    queueMicrotask(() => { if (active()) reload(); });
    return { node: root, crumb: "Review change" };
  }
  window.RiAuthGrantReview = Object.freeze({ view, reset });
})();
