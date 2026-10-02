"use strict";
// JSON writes stay under the original browser's return-cookie path. Native delivery
// belongs to its original resume route; a fetch never follows a relying-party redirect.
(() => {
  const $ = (id) => document.getElementById(id);
  const page = $("source-stage"), { resume, cancel, continue: continuation } = page.dataset;
  let busy = false, ended = false, factor = false;

  function problem(text) {
    $("stage-error").textContent = text;
    $("stage-error").hidden = false;
  }
  function show(result) {
    if ((result?.status === "pending" || result?.status === "local_factor_required") &&
        result.code_issued === false && result.continue === undefined) {
      factor = result.status === "local_factor_required";
      $("stage-factor").hidden = !factor;
      $("stage-resume").hidden = !factor;
      $("stage-recheck").hidden = factor;
      $("stage-status").textContent = factor ? "Confirm your local second factor." : "Your provider has not finished yet.";
      return;
    }
    if (((result?.status === "complete" && result.code_issued === true) ||
         ((result?.status === "rejected" || result?.status === "cancelled") && result.code_issued === false)) &&
        result.continue === continuation && continuation.startsWith("/") && !continuation.startsWith("//")) {
      ended = true;
      location.assign(continuation);
      return;
    }
    ended = true;
    problem("riAuth returned an unexpected result. Reload this page to check the original sign-in.");
    $("stage-reload").hidden = false;
  }
  async function send(button, path, body) {
    if (busy || ended) return;
    busy = true;
    $("stage-error").hidden = true;
    try {
      await RiAuth.inFlight(button, async () => {
        try {
          show(await RiAuth.post(path, body));
        } catch (error) {
          if (error.status === 0 || error.status >= 500) {
            ended = true;
            problem("Could not confirm the result. Reload this page before trying again.");
            $("stage-reload").hidden = false;
          } else {
            problem(error.description || "This sign-in could not be continued.");
          }
        } finally { $("stage-otp").value = ""; }
      });
    } finally { busy = false; }
  }
  $("stage-form").addEventListener("submit", (event) => event.preventDefault());
  RiAuth.guard($("stage-resume"), () => {
    if (!factor) return;
    const otp = $("stage-otp").value.trim();
    if (!otp) { problem("Enter an authenticator or recovery code."); return; }
    void send($("stage-resume"), resume, { otp });
  });
  RiAuth.guard($("stage-recheck"), () => void send($("stage-recheck"), resume, {}));
  RiAuth.guard($("stage-cancel"), () => void send($("stage-cancel"), cancel, {}));
  // Exactly one initial, guarded JSON check. No automatic factor/cancel retry or polling.
  void send($("stage-recheck"), resume, {});
})();
