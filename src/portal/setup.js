"use strict";
(() => {
  const form = document.getElementById("setup-form");
  const error = document.getElementById("setup-error");
  const proof = document.getElementById("setup-proof");
  const password = document.getElementById("setup-password");
  const confirm = document.getElementById("setup-confirm");
  const button = document.getElementById("setup-submit");
  function clearSecrets() { proof.value = ""; password.value = ""; confirm.value = ""; }
  function fail(message) { error.textContent = message; error.hidden = false; error.focus(); }
  window.addEventListener("pagehide", clearSecrets);
  window.addEventListener("pageshow", (event) => { if (event.persisted) clearSecrets(); });
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!form.reportValidity()) return;
    if (password.value !== confirm.value) { fail("Your passwords do not match."); return; }
    RiAuth.inFlight(button, async () => {
      error.hidden = true;
      const input = {
        proof: proof.value.trim(),
        username: document.getElementById("setup-username").value,
        display_name: document.getElementById("setup-name").value,
        email: document.getElementById("setup-email").value || null,
        password: password.value
      };
      clearSecrets();
      try {
        // Single-use mutation: no automatic retry and no proof in URLs or storage.
        await RiAuth.post("api/setup", input);
        form.hidden = true;
        const complete = document.getElementById("setup-complete");
        complete.hidden = false; complete.focus();
      } catch (e) {
        if (e.status === 409) {
          form.hidden = true;
          fail("Setup has already completed. Continue to sign in with the administrator account.");
          document.getElementById("setup-complete").hidden = false;
        } else if (e.status === 401) {
          fail("The setup proof is invalid or expired. Ask your operator for a new proof, then enter your password again.");
        } else if (e.status === 400) {
          fail(e.description || "Check your account details and enter the proof and password again.");
        } else {
          fail("We could not confirm setup. Try signing in first. If setup is still needed, enter the proof and password again.");
        }
      } finally { input.proof = ""; input.password = ""; }
    });
  });
})();
