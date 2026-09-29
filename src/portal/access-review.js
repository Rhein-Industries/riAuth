"use strict";
(() => {
  const base = document.querySelector('meta[name="riauth-base"]').content;
  const $ = (id) => document.getElementById(id);
  let review = null;
  let pending = null;
  let busy = false;

  function key() {
    const bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
  }
  function date(seconds) {
    return new Date(seconds * 1000).toLocaleString();
  }
  function node(tag, text, className) {
    const element = document.createElement(tag);
    if (text !== undefined) element.textContent = text;
    if (className) element.className = className;
    return element;
  }
  function notice(message) {
    $("status").textContent = message;
    $("status").hidden = !message;
    if (message) $("status").focus();
  }
  function controls() {
    $("refresh").disabled = busy || !!pending;
    $("retry").hidden = !pending;
    $("retry-action").disabled = busy;
    $("inspect").disabled = busy;
    document.querySelectorAll(".review-actions button").forEach((button) => {
      button.disabled = busy || !!pending;
    });
  }
  async function call(path, options = {}) {
    const headers = { "X-Riauth-Portal": "1", "Accept": "application/json" };
    if (options.key) headers["Idempotency-Key"] = options.key;
    if (options.revision !== undefined) headers["If-Match"] = `"${options.revision}"`;
    const response = await fetch(`${base}api/portal/access/${path}`, {
      method: options.method || "GET",
      headers,
      credentials: "same-origin",
      mode: "same-origin",
      cache: "no-store",
      // Under no-referrer, Safari can send Origin: null for same-origin POST.
      referrerPolicy: "same-origin",
    });
    let body = null;
    try { body = await response.json(); } catch { /* An error can have no body. */ }
    if (!response.ok) {
      const error = new Error(body?.error_description || body?.description || body?.error || "riAuth rejected the request.");
      error.status = response.status;
      throw error;
    }
    return body;
  }
  function empty(message) {
    return node("p", message, "review-empty");
  }
  function action(path, label, subject, danger = false) {
    const button = node("button", label, danger ? "button secondary" : "button primary");
    button.type = "button";
    button.addEventListener("click", () => {
      if (busy || pending || !review) return;
      if (!window.confirm(`${label} for ${subject}? Review the details on this page before continuing.`)) return;
      pending = { path, key: key(), revision: review.revision };
      void commit();
    });
    return button;
  }
  function render() {
    if (!review) return;
    $("identity").textContent = `Signed in as ${review.user.username}. Review revision ${review.revision}.`;
    const requests = $("requests");
    requests.replaceChildren();
    if (!review.requests.length) requests.append(empty("No requests are waiting for your decision."));
    for (const request of review.requests) {
      const card = node("article", undefined, "review-card");
      card.append(node("h3", `${request.username} asks for ${request.group}`));
      card.append(node("p", `Reason: ${request.reason}`));
      card.append(node("p", `Duration: ${Math.ceil(request.ttl / 60)} minutes · Requested ${date(request.created_at)}`, "review-meta"));
      const actions = node("div", undefined, "review-actions");
      const subject = `${request.username}'s request for ${request.group}`;
      actions.append(
        action(`requests/${encodeURIComponent(request.id)}/approve`, "Approve", subject),
        action(`requests/${encodeURIComponent(request.id)}/deny`, "Deny", subject, true),
      );
      card.append(actions);
      requests.append(card);
    }
    const grants = $("grants");
    grants.replaceChildren();
    if (!review.grants.length) grants.append(empty("No active temporary access is available to revoke."));
    for (const grant of review.grants) {
      const card = node("article", undefined, "review-card");
      card.append(node("h3", `${grant.username} · ${grant.group}`));
      card.append(node("p", `Ends ${date(grant.expires_at)}`, "review-meta"));
      const actions = node("div", undefined, "review-actions");
      actions.append(action(`grants/${encodeURIComponent(grant.id)}/revoke`, "Revoke", `${grant.username}'s temporary access to ${grant.group}`, true));
      card.append(actions);
      grants.append(card);
    }
    controls();
  }
  async function refresh() {
    if (busy || pending) return;
    busy = true; controls();
    try {
      review = await call("review");
      notice("");
      render();
    } catch (error) {
      review = null;
      $("requests").replaceChildren();
      $("grants").replaceChildren();
      notice(error.status === 401 ? "Your session ended. Sign in from Your applications, then return to review." :
        error.status === 403 ? "This account is not configured to review temporary access." :
        "Access review could not be loaded. Refresh to try again.");
    } finally {
      busy = false; controls();
    }
  }
  async function commit() {
    if (!pending || busy) return;
    busy = true; controls();
    try {
      await call(pending.path, { method: "POST", key: pending.key, revision: pending.revision });
      pending = null;
      busy = false;
      controls();
      await refresh();
      if (review) notice("The access change was recorded. The current review is shown.");
    } catch (error) {
      busy = false;
      if (!error.status || error.status >= 500) {
        notice("The response was lost or riAuth reported an error. Retry the same action with its saved request key, or inspect current state before starting a new action.");
      } else {
        pending = null;
        notice(error.status === 409 ? "The review changed or this item was already decided. Refresh and inspect the current state before another action." :
          error.status === 401 ? "Your session ended. Sign in from Your applications, then return to review." :
          error.status === 403 ? "Your authority to make this change is no longer active." :
          error.status === 404 ? "This item is no longer available. Refresh to inspect the current state." :
          error.message);
      }
      controls();
    }
  }
  $("refresh").addEventListener("click", () => { void refresh(); });
  $("retry-action").addEventListener("click", () => { void commit(); });
  $("inspect").addEventListener("click", () => {
    if (busy) return;
    pending = null;
    controls();
    void refresh();
  });
  void refresh();
})();
