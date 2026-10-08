"use strict";
// My agents: the signed-in person's own agents over /api/portal/agents. Writes carry the page's
// account and session binding; approval and rotation also need a recent sign-in in this browser.
// A credential is shown once and kept only in this page's memory until the person is done.
(() => {
  const $ = (id) => document.getElementById(id);
  // ---- Helpers (tools/browser/agents.test.js runs this block on its own) -------------------
  const PERSONAL = [
    ["profile.read", "See your name and email"],
    ["profile.write", "Change your display name"],
    ["sessions.read", "See where you are signed in"],
    ["sessions.revoke", "Sign out your sessions"],
    ["consents.read", "See applications you approved"],
    ["consents.revoke", "Withdraw application approvals"],
    ["agents.read", "See your agents"],
    ["agents.revoke", "Revoke your agents"],
  ];
  const LIFETIMES = [[3600, "1 hour"], [86400, "1 day"], [604800, "7 days"], [2592000, "30 days"]];
  const STATUS = { active: "Active", expired: "Expired", revoked: "Revoked" };
  // Mirrors validate_name: 1–64 ASCII letters, digits, dots, hyphens, underscores or @.
  const validAgentId = (id) => /^[A-Za-z0-9._@-]{1,64}$/.test(id) && id !== "." && id !== "..";
  // Checked personal actions on `self`, then one exact `action=resource` per advanced line.
  // Duplicates collapse; the server decides whether each is within the person's authority.
  function parsePermissions(selected, advanced) {
    const permissions = [], seen = new Set();
    const add = (action, resource) => {
      const key = `${action}=${resource}`;
      if (!seen.has(key)) { seen.add(key); permissions.push({ action, resource }); }
    };
    for (const action of selected) add(action, "self");
    const lines = String(advanced ?? "").split(/\r?\n/);
    for (const [index, raw] of lines.entries()) {
      const line = raw.trim();
      if (!line) continue;
      const match = /^([^\s=]+)=([^\s=]+)$/.exec(line);
      if (!match) return { error: `Line ${index + 1} is not one action=resource permission.` };
      add(match[1], match[2]);
    }
    return { permissions };
  }
  function agentStatus(agent, now) {
    if (agent.enabled !== true) return "revoked";
    return agent.expires_at <= now ? "expired" : "active";
  }
  // Active agents first, then by name.
  function sortAgents(agents, now) {
    const rank = { active: 0, expired: 1, revoked: 2 };
    return [...agents].sort((a, b) => rank[agentStatus(a, now)] - rank[agentStatus(b, now)] || a.id.localeCompare(b.id));
  }
  // A proposal for a name that is already issued can no longer be approved.
  function openProposals(proposals, agents, now) {
    const taken = new Set(agents.map((agent) => agent.id));
    return proposals.filter((proposal) => proposal.expires_at > now && !taken.has(proposal.agent.id));
  }
  // The exact bytes `riauth agent create --out` and `me agents create --out` write.
  const credentialFile = (credential) => JSON.stringify(credential);
  // ---- Page ---------------------------------------------------------------------------------
  let snapshot = null, generation = 0, pending = null, issued = null, verifying = false;
  const passkey = RiAuth.passkeyFlow(
    () => RiAuth.post("api/portal/login/passkey/start", { reauthenticate: true }, { retry: true }),
    (credential, started) => RiAuth.post("api/portal/login/passkey/finish", { ceremony: started.ceremony, credential }),
    false,
    (started) => RiAuth.post("api/portal/login/passkey/cancel", { ceremony: started.ceremony })
  );
  const labels = Object.fromEntries(PERSONAL);
  const now = () => Date.now() / 1000;

  function node(tag, className, text) {
    const item = document.createElement(tag);
    if (className) item.className = className;
    if (text !== undefined) item.textContent = text;
    return item;
  }
  function showStatus(message) {
    $("status").textContent = message;
    $("status").hidden = false;
  }
  function date(seconds) {
    return new Date(seconds * 1000).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "long" });
  }
  function binding() {
    return { expected_user_id: snapshot.user.id, expected_session_id: snapshot.current_session_id };
  }
  function signedOut(message) {
    snapshot = null; pending = null; clearCredential();
    $("agents-admin-link").hidden = $("agents-events-link").hidden = true;
    $("loading").hidden = $("agents-content").hidden = true;
    $("signed-out").hidden = false;
    if (message) showStatus(message); else $("status").hidden = true;
  }
  function permissionList(list, permissions, empty) {
    // Personal actions read as words; any other exact permission is shown as it is approved.
    const items = (permissions || []).map((permission) => {
      const item = node("li");
      if (labels[permission.action]) item.append(node("span", "", labels[permission.action]));
      item.append(node("code", "", `${permission.action}=${permission.resource}`));
      return item;
    });
    list.replaceChildren(...(items.length ? items : [node("li", "none", empty)]));
  }
  function lifetimeSelect(id) {
    const field = node("div", "field"), label = node("label", "", "New lifetime"), select = node("select");
    label.htmlFor = select.id = id;
    for (const [seconds, text] of LIFETIMES) {
      const option = node("option", "", text); option.value = String(seconds); option.selected = seconds === 86400;
      select.append(option);
    }
    field.append(label, select);
    return { field, select };
  }

  function agentRow(agent, index, at) {
    const status = agentStatus(agent, at);
    const row = node("li", "security-row agent-row"), details = node("div");
    const title = node("h3", "", agent.id);
    title.append(node("span", `agent-status ${status}`, STATUS[status]));
    const lifetime = status === "active" ? `Expires ${date(agent.expires_at)}`
      : status === "expired" ? `Expired ${date(agent.expires_at)}` : `Revoked. It would have expired ${date(agent.expires_at)}`;
    details.append(title, node("p", "", `${lifetime} · Created ${date(agent.created_at)}`));
    const approved = node("ul", "permission-list");
    permissionList(approved, agent.permissions, "None");
    details.append(node("p", "review-heading", "Approved permissions"), approved);
    if (status === "active") {
      const effective = node("ul", "permission-list");
      permissionList(effective, agent.effective_permissions, "Nothing right now: your account no longer holds these permissions.");
      details.append(node("p", "review-heading", "What they allow now"), effective);
    }
    // Activity loads only when asked for.
    const toggle = node("button", "text-button", "Show recent activity"), region = node("div", "activity-region");
    toggle.type = "button"; region.id = `agent-activity-${index}`; region.hidden = true;
    region.setAttribute("aria-live", "polite");
    toggle.setAttribute("aria-expanded", "false"); toggle.setAttribute("aria-controls", region.id);
    toggle.setAttribute("aria-label", `Show recent activity of ${agent.id}`);
    toggle.addEventListener("click", () => activity(toggle, region, agent));
    details.append(toggle, region);
    row.append(details);
    if (status !== "active") return row;

    const actions = node("div", "agent-actions");
    const replace = node("button", "button secondary", "Replace credential"), revokeButton = node("button", "button secondary", "Revoke");
    replace.type = revokeButton.type = "button";
    replace.setAttribute("aria-label", `Replace credential of ${agent.id}`);
    revokeButton.setAttribute("aria-label", `Revoke ${agent.id}`);
    const form = node("div", "rotate-form"), { field, select } = lifetimeSelect(`agent-rotate-${index}`);
    const confirmButton = node("button", "button primary", "Replace and show credential"), cancel = node("button", "button secondary", "Cancel");
    confirmButton.type = cancel.type = "button"; form.hidden = true;
    form.append(field, confirmButton, cancel);
    replace.setAttribute("aria-expanded", "false");
    replace.addEventListener("click", () => {
      form.hidden = false; replace.setAttribute("aria-expanded", "true"); RiAuth.arm(); select.focus();
    });
    cancel.addEventListener("click", () => { form.hidden = true; replace.setAttribute("aria-expanded", "false"); replace.focus(); });
    RiAuth.guard(confirmButton, () => rotate(confirmButton, agent, Number(select.value)));
    RiAuth.guard(revokeButton, () => revoke(revokeButton, agent));
    details.append(form);
    actions.append(replace, revokeButton);
    row.append(actions);
    return row;
  }
  function proposalRow(proposal) {
    const row = node("li", "security-row"), details = node("div");
    details.append(node("strong", "", proposal.agent.id),
      node("p", "", `Approve by ${date(proposal.expires_at)} · The agent would expire ${date(proposal.agent.expires_at)}`),
      node("p", "", proposal.agent.permissions.map((p) => `${p.action}=${p.resource}`).join(", ")));
    const button = node("button", "button secondary", "Review");
    button.type = "button"; button.setAttribute("aria-label", `Review ${proposal.agent.id}`);
    button.addEventListener("click", () => showReview(proposal));
    row.append(details, button);
    return row;
  }
  function render(security, list) {
    snapshot = security;
    $("agents-admin-link").hidden = security.user.admin !== true;
    $("agents-events-link").hidden = security.user.admin !== true || !RiAuthCapabilities.usable("audit.self_hosted_event_map");
    $("loading").hidden = $("signed-out").hidden = true;
    $("agents-content").hidden = false;
    $("account-line").textContent = `Signed in as ${security.user.display_name} (@${security.user.username})`;
    $("verify-panel").hidden = !verifying;
    $("verify-hint").textContent = security.browser_owned
      ? "Issuing an agent credential needs a sign-in in this browser within the last five minutes. Confirm it is you, then choose your action again."
      : "This browser shares a terminal sign-in. Confirm with a passkey or password here first; your terminal sign-in remains available until you revoke it.";
    const passkeyAvailable = RiAuthCapabilities.usable("identity.passkeys") && RiAuth.passkeysAvailable();
    $("verify-passkey").hidden = !passkeyAvailable;
    $("verify-form").hidden = !security.password_available;
    if (!security.password_available && !passkeyAvailable) {
      $("verify-hint").textContent = "This account has no browser sign-in method available here. Use an enrolled passkey in a supported browser, or contact your administrator.";
    }
    const at = now();
    const rows = sortAgents(list.agents, at).map((agent, index) => agentRow(agent, index, at));
    $("agent-list").replaceChildren(...rows);
    $("agents-empty").hidden = rows.length > 0;
    const open = openProposals(list.proposals, list.agents, at).map(proposalRow);
    $("proposal-list").replaceChildren(...open);
    $("proposals").hidden = open.length === 0;
    RiAuthCapabilities.apply();
  }
  // A quiet reload keeps the current status and any credential on screen.
  async function load(quiet = false) {
    const mine = ++generation;
    if (!quiet) { $("status").hidden = true; $("loading").hidden = snapshot !== null; }
    try {
      await RiAuthCapabilities.refresh().catch(() => {});
      const [security, list] = await Promise.all([RiAuth.get("api/portal/security"), RiAuth.get("api/portal/agents")]);
      if (mine === generation) render(security, list);
    } catch (error) {
      if (mine !== generation) return;
      if (error.status === 401) { signedOut(); return; }
      $("loading").hidden = true;
      if (!snapshot) $("agents-content").hidden = true;
      showStatus("Could not load your agents. Reload this page to try again.");
    }
  }
  async function activity(toggle, region, agent) {
    if (toggle.getAttribute("aria-expanded") === "true") {
      toggle.setAttribute("aria-expanded", "false"); toggle.textContent = "Show recent activity";
      toggle.setAttribute("aria-label", `Show recent activity of ${agent.id}`); region.hidden = true; return;
    }
    await RiAuth.inFlight(toggle, async () => {
      region.hidden = false; region.replaceChildren(node("p", "field-hint", "Loading activity…"));
      try {
        const result = await RiAuth.get(`api/portal/agents/${encodeURIComponent(agent.id)}/activity`);
        const events = [...result.events].reverse().map((event) => {
          const item = node("li", "", `${date(event.at)} · `);
          item.append(node("code", "", event.action), ` ${event.target}`);
          return item;
        });
        const list = node("ol", "activity-list");
        list.append(...events);
        region.replaceChildren(events.length ? list : node("p", "field-hint", "No recorded actions yet."));
        if (result.truncated) region.append(node("p", "field-hint", "Only the most recent part of the audit log was searched."));
        toggle.setAttribute("aria-expanded", "true"); toggle.textContent = "Hide recent activity";
        toggle.setAttribute("aria-label", `Hide recent activity of ${agent.id}`);
      } catch (error) {
        if (error.status === 401) { signedOut(); return; }
        region.replaceChildren(node("p", "field-hint", error.description || "Could not load this agent's activity. Try again."));
      }
    });
  }

  function showVerify(code) {
    verifying = true;
    $("verify-panel").hidden = false;
    if (code === "mfa_required") {
      $("verify-hint").textContent = "Confirm with your passkey, or your password and authenticator code. Then choose your action again.";
    }
    $("verify-error").hidden = true;
    $("verify-panel").scrollIntoView({ block: "nearest", behavior: "smooth" });
    ($("verify-passkey").hidden ? $("verify-password") : $("verify-passkey")).focus();
  }
  function clearVerifyInvalid() {
    for (const id of ["verify-password", "verify-otp"]) {
      $(id).removeAttribute("aria-invalid");
      $(id).removeAttribute("aria-describedby");
    }
    $("verify-otp").setAttribute("aria-describedby", "verify-otp-hint");
  }
  function showVerifyError(message, fields) {
    clearVerifyInvalid();
    for (const id of fields) {
      $(id).setAttribute("aria-invalid", "true");
      $(id).setAttribute("aria-describedby", id === "verify-otp" ? "verify-otp-hint verify-error" : "verify-error");
    }
    $("verify-error").textContent = message;
    $("verify-error").hidden = false;
    $("verify-error").focus();
  }
  async function verified() {
    verifying = false;
    await load();
    showStatus(pending ? "Identity confirmed. Choose Approve and issue again." : "Identity confirmed. Choose your action again.");
    if (pending) $("review-approve").focus();
  }
  // Errors every write shares. Anything else is shown as the server described it.
  async function failed(error) {
    if (error.status === 401) { signedOut(); return; }
    if (["account_changed", "session_changed"].includes(error.code)) {
      await load(); showStatus("Your sign-in changed. Review the current account and choose your action again."); return;
    }
    if (["reauthentication_required", "mfa_required"].includes(error.code)) { showVerify(error.code); return; }
    if (error.code === "credential_already_issued") {
      showStatus("This credential was already issued and cannot be shown again. If you did not save it, replace the agent's credential."); return;
    }
    showStatus(error.description || "The change could not be completed. Try again.");
  }

  function showReview(proposal) {
    pending = proposal;
    $("review-agent").textContent = proposal.agent.id;
    $("review-expires").textContent = date(proposal.agent.expires_at);
    $("review-deadline").textContent = date(proposal.expires_at);
    permissionList($("review-permissions"), proposal.agent.permissions, "None");
    permissionList($("review-effective"), proposal.agent.effective_permissions, "Nothing right now.");
    $("review-panel").hidden = false;
    RiAuth.arm();
    $("review-panel").scrollIntoView({ block: "start", behavior: "smooth" });
    $("review-title").focus({ preventScroll: true });
  }
  function hideReview() { pending = null; $("review-panel").hidden = true; }
  async function approve() {
    if (!snapshot || !pending) return;
    const proposal = pending;
    if (proposal.expires_at <= now()) {
      hideReview(); await load(true); showStatus("That proposal expired before it was approved. Prepare the agent again."); return;
    }
    await RiAuth.inFlight($("review-approve"), async () => {
      try {
        const result = await RiAuth.post(`api/portal/agents/proposals/${encodeURIComponent(proposal.proposal_id)}/approve`,
          { ...binding(), digest: proposal.digest }, { retry: true });
        hideReview(); $("create-form").reset(); $("advanced").open = false;
        showCredential(result);
        await load(true);
        showStatus(`${result.agent.id} is approved. Save its credential now; it is shown only once.`);
      } catch (error) {
        if (error.code === "credential_already_issued") hideReview();
        await failed(error);
      }
    });
  }
  async function rotate(button, agent, ttl) {
    if (!snapshot) return;
    if (!window.confirm(`Replace the credential of ${agent.id}? The current credential stops working at once.`)) return;
    const key = crypto.randomUUID();
    await RiAuth.inFlight(button, async () => {
      try {
        const result = await RiAuth.post(`api/portal/agents/${encodeURIComponent(agent.id)}/rotate`, { ...binding(), ttl }, { key, retry: true });
        showCredential(result);
        await load(true);
        showStatus(`${agent.id} has a new credential. Save it now; the previous one no longer works.`);
      } catch (error) { await failed(error); }
    });
  }
  async function revoke(button, agent) {
    if (!snapshot) return;
    if (!window.confirm(`Revoke ${agent.id}? It stops working at once and cannot be turned back on.`)) return;
    await RiAuth.inFlight(button, async () => {
      try {
        await RiAuth.post(`api/portal/agents/${encodeURIComponent(agent.id)}/revoke`, binding(), { retry: true });
        await load(); showStatus(`${agent.id} is revoked.`); $("agents-title").focus();
      } catch (error) { await failed(error); }
    });
  }

  // The credential lives only in this page's memory and DOM, never in storage.
  function showCredential(result) {
    issued = { text: credentialFile(result.credential), agent: result.credential.agent_id };
    $("credential-agent").textContent = result.credential.agent_id;
    $("credential-expires").textContent = date(result.credential.expires_at);
    $("credential-json").textContent = issued.text;
    $("credential-saved").checked = false;
    $("credential-status").textContent = "";
    $("credential-panel").hidden = false;
    RiAuth.arm();
    $("credential-panel").scrollIntoView({ block: "start", behavior: "smooth" });
    $("credential-title").focus({ preventScroll: true });
  }
  function clearCredential() {
    issued = null;
    for (const id of ["credential-json", "credential-agent", "credential-expires", "credential-status"]) $(id).textContent = "";
    $("credential-panel").hidden = true;
  }
  $("credential-copy").addEventListener("click", async () => {
    if (!issued) return;
    try { await navigator.clipboard.writeText(issued.text); $("credential-status").textContent = "Credential copied."; }
    catch {
      const selection = window.getSelection(), range = document.createRange();
      range.selectNodeContents($("credential-json")); selection.removeAllRanges(); selection.addRange(range);
      $("credential-status").textContent = "Select the credential and copy it with your keyboard.";
    }
  });
  $("credential-download").addEventListener("click", () => {
    if (!issued) return;
    const url = URL.createObjectURL(new Blob([issued.text], { type: "application/json" }));
    const link = node("a"); link.href = url; link.download = `riauth-agent-${issued.agent}.json`;
    document.body.append(link); link.click(); link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    $("credential-status").textContent = "Credential downloaded. Keep the file readable only by you.";
  });
  RiAuth.guard($("credential-close"), () => {
    if (!$("credential-saved").checked) {
      $("credential-status").textContent = "Save the credential, then confirm that you saved it."; $("credential-saved").focus(); return;
    }
    clearCredential(); $("agents-title").focus();
  });
  window.addEventListener("beforeunload", (event) => { if (issued) event.preventDefault(); });
  window.addEventListener("pagehide", () => { clearCredential(); pending = null; $("review-panel").hidden = true; });

  function createError(message, field) {
    $("create-error").textContent = message; $("create-error").hidden = false;
    if (field) { $(field).setAttribute("aria-invalid", "true"); $(field).setAttribute("aria-describedby", `${field}-hint create-error`); }
    $("create-error").focus();
  }
  function clearCreateError() {
    $("create-error").hidden = true;
    for (const id of ["agent-id", "agent-advanced"]) {
      $(id).removeAttribute("aria-invalid"); $(id).setAttribute("aria-describedby", `${id}-hint`);
    }
  }
  $("personal-options").replaceChildren(...PERSONAL.map(([action, text]) => {
    const label = node("label", "permission-choice"), input = node("input"), span = node("span", "", text);
    input.type = "checkbox"; input.value = action; input.name = "personal";
    span.append(node("code", "", `${action}=self`));
    label.append(input, span);
    return label;
  }));
  $("create-form").addEventListener("submit", (event) => {
    event.preventDefault();
    if (!snapshot) return;
    clearCreateError();
    const id = $("agent-id").value.trim();
    if (!validAgentId(id)) { createError("Use 1 to 64 letters, digits, dots, hyphens, underscores or @ for the name.", "agent-id"); return; }
    const ttl = Number($("create-form").querySelector('input[name="lifetime"]:checked')?.value);
    const selected = [...$("personal-options").querySelectorAll("input:checked")].map((input) => input.value);
    const parsed = parsePermissions(selected, $("agent-advanced").value);
    if (parsed.error) { $("advanced").open = true; createError(parsed.error, "agent-advanced"); return; }
    if (!parsed.permissions.length) { createError("Choose at least one thing the agent may do."); return; }
    RiAuth.inFlight($("create-submit"), async () => {
      try {
        const proposal = await RiAuth.post("api/portal/agents", { ...binding(), agent: { id, permissions: parsed.permissions, ttl } });
        await load(true);
        showReview(proposal);
      } catch (error) {
        if (error.status === 401 || ["account_changed", "session_changed"].includes(error.code)) { await failed(error); return; }
        createError(error.description || "The agent could not be prepared. Try again.");
      }
    });
  });
  $("agent-id").addEventListener("input", clearCreateError);
  $("agent-advanced").addEventListener("input", clearCreateError);
  RiAuth.guard($("review-approve"), () => approve());
  $("review-cancel").addEventListener("click", () => { hideReview(); $("agents-title").focus(); });

  $("verify-form").addEventListener("submit", (event) => {
    event.preventDefault();
    if (!snapshot) return;
    const password = $("verify-password").value;
    if (!password) { showVerifyError("Enter your password.", ["verify-password"]); return; }
    if (!RiAuth.otp("verify-otp").valid()) {
      showVerifyError("Enter a 6-digit code, or an 8-digit code if your app uses one.", ["verify-otp"]); return;
    }
    const otp = RiAuth.otp("verify-otp").value();
    clearVerifyInvalid();
    RiAuth.inFlight($("verify-password-button"), async () => {
      try {
        await RiAuth.post("api/portal/login/password", {
          username: snapshot.user.username, password, otp, reauthenticate: true
        });
        $("verify-password").value = $("verify-otp").value = "";
        await verified();
      } catch (error) {
        showVerifyError(error.description || "Could not confirm your identity. Try again.", ["verify-password", "verify-otp"]);
      }
    });
  });
  for (const id of ["verify-password", "verify-otp"]) {
    $(id).addEventListener("input", () => {
      $(id).removeAttribute("aria-invalid");
      if (id === "verify-otp") $(id).setAttribute("aria-describedby", "verify-otp-hint"); else $(id).removeAttribute("aria-describedby");
    });
  }
  $("verify-passkey").addEventListener("click", () => RiAuth.inFlight($("verify-passkey"), async () => {
    if (!snapshot) return;
    try {
      await passkey(); await verified();
    } catch (error) {
      $("verify-error").textContent = error.name === "NotAllowedError" || error.name === "AbortError"
        ? "Passkey confirmation was cancelled or timed out. Try again."
        : error.description || "Could not confirm your identity with a passkey.";
      $("verify-error").hidden = false; $("verify-error").focus();
    }
  }));
  window.addEventListener("pageshow", (event) => { if (event.persisted) load(); });
  load();
})();
