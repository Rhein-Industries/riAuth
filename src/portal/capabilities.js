"use strict";
// Browser pages use the running instance's capability state. This only controls
// presentation; API routes still authorize every read and write independently.
(() => {
  const base = document.querySelector('meta[name="riauth-base"]').content;
  let current = null;
  const usable = (name) => current?.feature_states?.[name]?.usable === true;
  const compiled = (name) => current?.feature_states?.[name]?.compiled === true;
  async function refresh() {
    current = null;
    apply();
    const controller = new AbortController(), timeout = setTimeout(() => controller.abort(), 15000);
    try {
      const response = await fetch(`${base}api/capabilities`, {
        method: "GET", credentials: "same-origin", mode: "same-origin", cache: "no-store",
        headers: { Accept: "application/json" }, signal: controller.signal
      });
      if (!response.ok) throw new Error("Capability state unavailable");
      const state = await response.json();
      if (state.scope !== "instance" || state.schema_version !== "riauth.capabilities/v2" || !state.feature_states) {
        throw new Error("Invalid instance capability state");
      }
      current = state;
      apply();
      return state;
    } finally {
      clearTimeout(timeout);
    }
  }
  function apply(root = document) {
    for (const node of root.querySelectorAll("[data-capability]")) {
      node.setAttribute("data-capability-ready", "");
      node.toggleAttribute("data-capability-disabled", !usable(node.dataset.capability));
    }
  }
  window.RiAuthCapabilities = { refresh, usable, compiled, apply };
})();
