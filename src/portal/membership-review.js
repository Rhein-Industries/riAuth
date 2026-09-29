"use strict";
// Browser adapter only: the shared membership service owns exact content,
// authority, dependency validation, audit and one-time execution.
(() => {
  const route = (id = "") => `#/membership-review${id ? `/${encodeURIComponent(id)}` : ""}`;
  const idPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  const namePattern = /^[A-Za-z0-9_.@-]{1,64}$/;
  const name = (value) => typeof value === "string" && namePattern.test(value) && ![".", ".."].includes(value);
  function checkedChange(value, expectedId) {
    const p = value?.proposal;
    const text = (v) => typeof v === "string" && v.length > 0 && v.length <= 256;
    const number = (v) => Number.isSafeInteger(v) && v >= 0;
    const time = (v) => number(v) && v < 8640000000000;
    const authority = (v) => text(v?.id) && number(v.epoch);
    const members = (v) => Array.isArray(v) && v.length <= 128
      && v.every((m) => text(m?.user_id) && (m.username === null || name(m.username)))
      && new Set(v.map((m) => m.user_id)).size === v.length;
    if (!p || !idPattern.test(p.id) || expectedId && p.id !== expectedId || !text(value.digest)
        || !["pending", "approved", "executed", "cancelled"].includes(value.status)
        || !name(p.group) || p.resource !== `group/${p.group}/members` || !authority(p.author)
        || !members(p.before) || !members(p.after) || !number(p.base_revision)
        || !text(p.resource_revision) || !text(p.policy_revision) || !time(p.created_at) || !time(p.expires_at)
        || !Array.isArray(value.approvals) || value.approvals.length > 8
        || !value.approvals.every((a) => authority(a.reviewer) && a.digest === value.digest && time(a.at))
        || value.executor && !authority(value.executor)
        || value.executed_at !== null && !time(value.executed_at)) throw { status: 502 };
    return value; // Malformed responses never enable actions.
  }
  let generation = 0, clock, savedDraft = null;
  function reset(forget = false) { generation += 1; clearInterval(clock); if (forget) savedDraft = null; }
  function message(error) {
    if (!error.status || error.status >= 500) return "The response was lost or incomplete. The change may have been recorded. Refresh to check its status before taking another action.";
    if (error.status === 403) return "This action is not allowed. The author, each reviewer and executor must be distinct, currently authorized administrators. A person whose membership changes cannot participate. Refresh and check the participants.";
    if (error.status === 404) return "This group or change is unavailable. It may have expired or been removed. Check the exact group name or change ID.";
    if (error.status === 409 || error.status === 428) return "The server refused this change because its content, authority, dependencies or status no longer permits the action. Refresh to inspect it; stale proposals need a new review.";
    if (error.status === 429) return "Too many requests. Wait a minute, then refresh before trying again.";
    return "The request was rejected. Choose an existing protected group and up to 128 distinct, enabled usernames. Retained members also need independent credential provenance. Ordinary groups use immediate membership changes.";
  }
  function view({ id, api, h, me, users, groups, identityChanged, sessionLost }) {
    const run = ++generation, owner = me?.user?.id;
    const selected = id?.startsWith("group:") ? id.slice(6) : "";
    if (selected) id = null;
    if (savedDraft?.owner !== owner || !me?.user?.admin) savedDraft = null;
    if (selected && savedDraft?.group !== selected && !savedDraft?.pending) savedDraft = null;
    const root = h("div", { class: "grant-review membership-review" });
    const active = () => run === generation && root.isConnected;
    const title = h("h1", { tabindex: "-1" }, id ? "Review membership change" : "Reviewed membership");
    const status = h("p", { class: "form-error", role: "alert", tabindex: "-1", hidden: true });
    root.append(h("section", { class: "page-heading" }, h("div", {}, h("p", { class: "eyebrow" }, "GROUPS"), title,
      h("p", { class: "page-description" }, "Propose the complete durable membership of a protected group. A separate administrator reviews and another executes."))), status);
    const localError = (text) => { status.textContent = text; status.hidden = false; status.focus(); };
    const error = (e) => { if (active()) { if (e.status === 401) sessionLost(401); else localError(message(e)); } };
    const button = (text, click, attrs = {}) => h("button", { type: "button", class: "button secondary", onclick: click, ...attrs }, text);
    const field = (label, input, hint) => h("div", { class: "field" }, h("label", { for: input.id }, label), input,
      hint ? h("p", { class: "field-hint" }, hint) : null);
    const key = () => crypto.randomUUID();
    const actor = (value) => {
      const user = users.find((u) => u.id === value);
      return h("span", { class: "grant-identity" }, user ? h("strong", {}, user.username) : null, h("code", {}, value));
    };
    const date = (seconds) => h("time", { datetime: new Date(seconds * 1000).toISOString() }, new Date(seconds * 1000).toLocaleString(undefined, { timeZoneName: "short" }));
    const facts = (rows) => h("dl", { class: "facts grant-facts" }, rows.map(([label, value]) => [h("dt", {}, label), h("dd", {}, value)]));
    function members(title, rows, other) {
      return h("section", { class: "admin-card" }, h("h2", {}, title), rows.length
        ? h("ul", { class: "grant-list" }, rows.map((member) => h("li", {},
          h("strong", {}, member.username || "Missing identity"), h("code", {}, member.user_id),
          other ? h("span", { class: "field-hint" }, other.some((m) => m.user_id === member.user_id) ? "Retained" : title.startsWith("Before") ? "Removed" : "Added") : null)))
        : h("p", { class: "field-hint" }, "No durable members."));
    }
    async function session() {
      const current = await api("GET", "admin/session");
      if (!active()) return null;
      if (current?.user?.id !== owner) { identityChanged(); return null; }
      if (!current.user.admin) { sessionLost(403); return null; }
      return current;
    }
    if (!me?.user?.admin) {
      root.append(h("p", { class: "notice" }, "A full administrator account is required to review protected membership."));
      return { node: root };
    }
    if (!id) {
      const lookup = h("input", { id: "membership-change-id", required: true, maxlength: "36", autocomplete: "off" });
      const open = h("form", { class: "inline-form" }, field("Change ID", lookup), h("button", { type: "submit", class: "button secondary" }, "Open change"));
      open.addEventListener("submit", (event) => {
        event.preventDefault();
        if (!idPattern.test(lookup.value.trim())) { localError("Enter the complete change ID from the author."); return; }
        location.hash = route(lookup.value.trim());
      });
      const group = h("input", { id: "membership-group", required: true, maxlength: "64", autocomplete: "off", list: "membership-groups", value: savedDraft?.group || selected });
      const load = h("button", { type: "submit", class: "button secondary" }, "Load current members");
      const select = h("form", { class: "inline-form" }, field("Group name", group),
        h("datalist", { id: "membership-groups" }, groups.map((g) => h("option", { value: g.name }))), load);
      const editor = h("div");
      root.append(open, h("h2", {}, "Stage a replacement"),
        h("p", { class: "field-hint" }, "Use an existing group protected by reviewed membership or PAM configuration. Ordinary groups keep their immediate actions. Drafts stay in memory in this tab; a full page reload discards unsent edits. Staged changes reopen by their review link."), select, editor);
      let loading = false, selection = 0;
      group.addEventListener("input", () => { selection += 1; savedDraft = null; editor.replaceChildren(); });
      select.addEventListener("submit", async (event) => {
        event.preventDefault();
        if (loading || savedDraft?.pending) return;
        const chosen = ++selection, target = group.value.trim();
        if (!name(target)) { localError("Enter one exact group name."); return; }
        loading = true; load.disabled = true; status.hidden = true;
        try {
          const current = await session();
          if (!current) return;
          const [currentGroups, currentUsers] = await Promise.all([api("GET", "admin/groups"), api("GET", "admin/users")]);
          if (!active() || chosen !== selection) return;
          const found = currentGroups.find((g) => g.name === target);
          if (!found) throw { status: 404 };
          if (found.members.length > 128) { localError("This group exceeds the review limit of 128 current members."); editor.replaceChildren(); return; }
          const before = found.members.map((id) => ({ user_id: id, username: currentUsers.find((u) => u.id === id)?.username || null }));
          const old = savedDraft?.group === target ? savedDraft : null;
          savedDraft = { owner, group: target, before, revision: current.revision,
            text: old ? old.text : before.map((m) => m.username).filter(Boolean).sort().join("\n"), pending: null };
          draft(savedDraft, chosen);
        } catch (e) { error(e); }
        finally { loading = false; if (active()) load.disabled = false; }
      });
      function draft(record, chosen) {
        const content = h("textarea", { id: "membership-members", rows: "8", maxlength: "8320", spellcheck: "false", autocomplete: "off", value: record.text });
        const acknowledgement = h("input", { id: "membership-stage-ack", type: "checkbox", required: true });
        const stage = h("button", { type: "submit", class: "button primary" });
        const empty = h("p", { class: "notice warn-notice" });
        const form = h("form", { class: "admin-form admin-card" }, h("h2", {}, `Proposed members of ${record.group}`),
          field("Complete proposed usernames", content, "One exact username per line. Leave empty to remove every member."), empty,
          h("p", { class: "field-hint" }, `Draft based on management revision ${record.revision}. Refresh keeps this content and its original revision. Load current members to rebase explicitly, then check the complete replacement again.`),
          h("label", { class: "checkbox", for: acknowledgement.id }, acknowledgement, "I checked the complete replacement, including every member being removed."), h("div", { class: "form-actions" }, stage));
        let busy = false, blocked = false;
        const update = () => {
          const pending = !!record.pending;
          stage.textContent = pending ? "Retry same staging request" : "Stage exact change";
          content.disabled = pending || busy; acknowledgement.disabled = pending || busy || blocked;
          group.disabled = pending || busy; load.disabled = pending || busy;
          stage.disabled = busy || blocked || (!pending && !acknowledgement.checked);
          const count = record.text.split("\n").filter((line) => line.trim()).length;
          empty.textContent = pending ? "Staging is unconfirmed. Retry the same request to recover its result; proposed members and revision are locked. Avoid reloading this page until it is resolved."
            : count ? `${count} proposed members. Every current member omitted here will be removed.` : "Execution will remove every durable member from this group.";
        };
        content.addEventListener("input", () => { record.text = content.value; acknowledgement.checked = false; update(); });
        acknowledgement.addEventListener("change", update);
        editor.replaceChildren(members("Current members · loaded snapshot", record.before), form); update();
        form.addEventListener("submit", async (event) => {
          event.preventDefault();
          if (busy || blocked || chosen !== selection || savedDraft !== record || (!record.pending && !acknowledgement.checked)) return;
          const after = record.text.split("\n").map((line) => line.trim()).filter(Boolean);
          if (after.length > 128 || after.some((value) => !name(value)) || new Set(after).size !== after.length) { localError("Enter at most 128 distinct, exact usernames, one per line."); return; }
          record.pending ||= { body: { members: after }, revision: record.revision, key: key() };
          busy = true; status.hidden = true; update();
          try {
            if (!await session()) return;
            const change = checkedChange(await api("POST", `admin/groups/${encodeURIComponent(record.group)}/membership-changes`, record.pending.body, record.pending));
            if (active()) { savedDraft = null; location.hash = route(change.proposal.id); }
          } catch (e) {
            if (e.status && e.status < 500) { record.pending = null; blocked = [403, 409, 428].includes(e.status); }
            error(e);
          } finally { busy = false; if (active()) update(); }
        });
      }
      if (savedDraft) draft(savedDraft, selection);
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
    const endpoint = `admin/group-membership-changes/${encodeURIComponent(id)}`;
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
      if (uncertain) return ["Outcome unknown", "warn", "The response was lost. Refresh this change to confirm whether the action completed."];
      if (rejected) return ["Stale", "warn", "The server rejected this action because authority, membership, policy or a dependency changed. Stage a new proposal; this approval cannot be reused."];
      if (row.status === "approved") return ["Approved", "info", "An independent administrator can execute this exact change once."];
      return ["Awaiting review", "info", "At least one administrator other than the author must approve this exact change."];
    }
    function draw() {
      const proposal = row.proposal;
      const label = h("strong", { class: "badge", id: "membership-status", role: "status" });
      const explanation = h("p", { class: "field-hint", id: "membership-status-detail" });
      const approvalList = row.approvals.length ? h("ul", { class: "plain-list" }, row.approvals.map((a) => h("li", {}, actor(a.reviewer.id), " · ", date(a.at)))) : "No reviewers yet";
      const link = h("input", { id: "membership-review-link", readOnly: true, value: new URL(route(id), location.href).href });
      const ack = h("input", { id: "membership-review-ack", type: "checkbox" });
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
        const affected = proposal.before.some((m) => m.user_id === owner) !== proposal.after.some((m) => m.user_id === owner);
        approve.disabled = !available || !ack.checked || author || reviewer || affected || row.approvals.length >= 8;
        execute.disabled = !available || !ack.checked || author || reviewer || affected || row.status !== "approved";
        cancel.disabled = busy || uncertain || ended;
        ack.disabled = !available;
        refresh.disabled = busy;
        actionHint.textContent = affected ? "Your membership changes in this proposal. You cannot author, review or execute it." : author ? "You are the author. Other administrators must review and execute."
          : reviewer ? "You have approved this change. Another administrator must execute."
          : "The author, every reviewer and executor must be distinct. The server checks live authority and dependencies again for each action.";
        if (currentSession.expires_at <= Date.now() / 1000) sessionLost(401);
      };
      ack.addEventListener("change", update);
      detail.replaceChildren(...[
        h("section", { class: "admin-card" }, h("h2", {}, `Change for ${proposal.group}`), label, explanation,
          facts([["Resource", h("code", {}, proposal.resource)],
            ["Author", actor(proposal.author.id)], ["Reviewers", approvalList],
            ["Executor", row.executor ? actor(row.executor.id) : "Not executed"],
            ["Created", date(proposal.created_at)], ["Expires", date(proposal.expires_at)],
            ...(row.executed_at ? [["Executed", date(row.executed_at)]] : [])])),
        h("div", { class: "grant-comparison" }, members("Before · exact members", proposal.before, proposal.after), members("After · exact members", proposal.after, proposal.before)),
        !proposal.after.length ? h("p", { class: "notice warn-notice" }, "Execution will remove every durable member from this group.") : null,
        h("section", { class: "admin-card" }, h("h2", {}, "Immutable approval binding"),
          facts([["Change ID", h("code", { id: "membership-id" }, proposal.id)], ["Canonical digest", h("code", { id: "membership-digest" }, row.digest)],
            ["Management revision", String(proposal.base_revision)], ["Resource dependencies", h("code", {}, proposal.resource_revision)],
            ["Policy dependencies", h("code", {}, proposal.policy_revision)]]),
          h("p", { class: "field-hint" }, "Approval covers only this content, these dependencies and this expiry. To change any members, stage a new proposal."),
          field("Review link", link, "Share this link with the next administrator. It contains no credential and requires an authorized session.")),
        h("section", { class: "admin-card" }, h("h2", {}, "Your action"), actionHint,
          h("label", { class: "checkbox", for: ack.id }, ack, "I checked the exact members, digest and dependencies."),
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
          if (result.digest !== digest || JSON.stringify(result.proposal) !== JSON.stringify(row.proposal)) throw { status: 502 };
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
  window.RiAuthMembershipReview = Object.freeze({ view, reset });
})();
