"use strict";
// Shared by the portal and the sign-in page. Credentials stay in HttpOnly cookies: this
// file never reads cookies, stores tokens or builds absolute URLs.
(() => {
  const base = document.querySelector('meta[name="riauth-base"]').content;
  const busy = new WeakSet();
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  function failure(status, code, description) {
    const error = new Error(description || "Request failed");
    error.status = status; error.code = code; error.description = description; return error;
  }
  async function send(path, init) {
    const controller = new AbortController(), timeout = setTimeout(() => controller.abort(), 15000);
    try {
      let response;
      try {
        response = await fetch(`${base}${path}`, {
          ...init, credentials: "same-origin", mode: "same-origin", cache: "no-store",
          // Keep a verifiable Origin on same-origin POSTs in Firefox/WebKit even
          // though pages suppress referrers for external navigation.
          referrerPolicy: "same-origin", redirect: "error", signal: controller.signal
        });
      } catch { throw failure(0, "network_error", ""); }
      let data = null;
      try { data = await response.json(); } catch { /* Proxies and admission errors may not return JSON. */ }
      if (!response.ok) {
        throw failure(response.status, typeof data?.error === "string" ? data.error : "",
          typeof data?.error_description === "string" ? data.error_description : "");
      }
      return data;
    } finally { clearTimeout(timeout); }
  }
  // Only idempotent calls retry a busy server; a replayed credential POST would spend a code.
  async function retrying(request, retry) {
    for (const delay of retry ? [1000, 3000] : []) {
      try { return await request(); } catch (error) {
        if (error.status !== 503) throw error;
        await sleep(delay * (0.8 + Math.random() * 0.4));
      }
    }
    return request();
  }
  function get(path) {
    return retrying(() => send(path, { method: "GET", headers: { "Accept": "application/json" } }), true);
  }
  function post(path, body, options = {}) {
    return retrying(() => send(path, {
      method: "POST", body: JSON.stringify(body ?? {}),
      headers: { "X-Riauth-Portal": "1", "Content-Type": "application/json", "Accept": "application/json" }
    }), options.retry === true);
  }
  // Buttons that act on a decision ignore activation for a moment after their screen
  // appears and after this window is shown or regains focus. A click aimed at a window that
  // just went away (double-click-jacking: frame-ancestors cannot help, the page is not
  // framed) cannot land on them. aria-disabled keeps keyboard focus where it is.
  const ARM_MS = 500, guarded = new Set();
  let armedUntil = 0, armTimer;
  function arm() {
    armedUntil = performance.now() + ARM_MS;
    for (const button of guarded) button.setAttribute("aria-disabled", "true");
    clearTimeout(armTimer);
    armTimer = setTimeout(() => { for (const button of guarded) button.removeAttribute("aria-disabled"); }, ARM_MS);
  }
  function guard(button, action) {
    guarded.add(button);
    if (performance.now() < armedUntil) button.setAttribute("aria-disabled", "true");
    button.addEventListener("click", (event) => {
      if (performance.now() < armedUntil) { event.preventDefault(); return; }
      action(event);
    });
  }
  window.addEventListener("focus", arm);
  window.addEventListener("pageshow", arm);
  document.addEventListener("visibilitychange", () => { if (document.visibilityState === "visible") arm(); });
  arm();

  async function inFlight(button, fn) {
    if (busy.has(button)) return undefined;
    busy.add(button); button.disabled = true; button.setAttribute("aria-busy", "true");
    try { return await fn(); }
    finally { busy.delete(button); button.disabled = false; button.removeAttribute("aria-busy"); }
  }

  function bytes(value) {
    const text = atob(String(value).replaceAll("-", "+").replaceAll("_", "/").padEnd(Math.ceil(String(value).length / 4) * 4, "="));
    return Uint8Array.from(text, (c) => c.charCodeAt(0));
  }
  function text(buffer) {
    let binary = "";
    for (const byte of new Uint8Array(buffer)) binary += String.fromCharCode(byte);
    return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
  }
  // Accepts the server's `public_key` object or its inner `publicKey` options. Extensions
  // and null members are dropped.
  function options(publicKey) {
    const { extensions, ...rest } = publicKey?.publicKey ?? publicKey;
    return Object.fromEntries(Object.entries(rest).filter(([, value]) => value !== null && value !== undefined));
  }
  const descriptors = (list) => (list || []).map((c) => ({ type: c.type, id: bytes(c.id) }));
  async function passkeyGet(publicKey) {
    const o = options(publicKey);
    const credential = await navigator.credentials.get({ publicKey: { ...o, challenge: bytes(o.challenge), allowCredentials: descriptors(o.allowCredentials) } });
    const r = credential.response;
    return {
      id: credential.id, rawId: text(credential.rawId), type: credential.type,
      response: { authenticatorData: text(r.authenticatorData), clientDataJSON: text(r.clientDataJSON), signature: text(r.signature), userHandle: r.userHandle ? text(r.userHandle) : null },
      clientExtensionResults: {}
    };
  }
  async function passkeyCreate(publicKey) {
    const o = options(publicKey);
    const credential = await navigator.credentials.create({ publicKey: { ...o, challenge: bytes(o.challenge), user: { ...o.user, id: bytes(o.user.id) }, excludeCredentials: descriptors(o.excludeCredentials) } });
    const r = credential.response;
    return {
      id: credential.id, rawId: text(credential.rawId), type: credential.type,
      response: { attestationObject: text(r.attestationObject), clientDataJSON: text(r.clientDataJSON), transports: typeof r.getTransports === "function" ? r.getTransports() : [] },
      clientExtensionResults: {}
    };
  }
  // WebKit only calls WebAuthn from a fresh user gesture. After a cancelled prompt the
  // unused options are kept, so the next click reaches WebAuthn without a fetch first.
  // start() resolves to {ceremony, public_key, expires_in}; finish(credential, started).
  function passkeyFlow(start, finish, create = false) {
    let kept = null;
    return async () => {
      let started = kept && Date.now() - kept.at < 240000 ? kept.started : null;
      const at = started ? kept.at : Date.now();
      kept = null;
      if (!started) started = await start();
      let credential;
      try { credential = await (create ? passkeyCreate : passkeyGet)(started.public_key); }
      catch (error) { if (error?.name === "NotAllowedError") kept = { started, at }; throw error; }
      return finish(credential, started);
    };
  }
  const passkeysAvailable = () => "PublicKeyCredential" in window && isSecureContext;
  const shellQuote = (s) => `'${s.replaceAll("'", "'\\''")}'`;

  window.RiAuth = Object.freeze({ base, get, post, inFlight, arm, guard, passkeyGet, passkeyCreate, passkeyFlow, passkeysAvailable, shellQuote });
})();
