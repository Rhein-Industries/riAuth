"use strict";
// Sign-in buttons for upstream providers on the applications page. Starting one sets its
// one-use credential as an HttpOnly cookie; this script only follows the provider's URL.
(() => {
  const container = document.getElementById("source-login");
  if (!container) return;
  const error = document.createElement("p");
  error.className = "form-error"; error.setAttribute("role", "alert"); error.tabIndex = -1; error.hidden = true;

  async function start(source) {
    error.hidden = true;
    try {
      const started = await RiAuth.post(`api/portal/sources/${encodeURIComponent(source.id)}/start`, {});
      const target = new URL(started.authorization_url);
      if (target.protocol !== "https:" && target.protocol !== "http:") throw new Error("Unexpected provider address");
      location.assign(target.href);
    } catch (failure) {
      error.textContent = failure.status === 429 ? "Too many sign-in attempts. Wait a minute, then try again."
        : failure.status === 0 ? "Couldn't reach riAuth. Check your connection and try again."
          : failure.description || `Couldn't start signing in with ${source.name}. Try again.`;
      error.hidden = false; error.focus();
    }
  }
  (async () => {
    let data;
    try { data = await RiAuth.get("api/portal/sources"); } catch { return; }
    if (!Array.isArray(data?.sources) || !data.sources.length) return;
    const buttons = data.sources.map((source) => {
      const button = document.createElement("button");
      button.type = "button"; button.className = "button secondary";
      button.textContent = `Continue with ${source.name}`;
      RiAuth.guard(button, () => RiAuth.inFlight(button, () => start(source)));
      return button;
    });
    container.replaceChildren(...buttons, error);
    container.hidden = false;
  })();
})();
