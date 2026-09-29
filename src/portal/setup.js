"use strict";
(() => {
  const form = document.getElementById("setup-form");
  const error = document.getElementById("setup-error");
  const proof = document.getElementById("setup-proof");
  const password = document.getElementById("setup-password");
  const confirm = document.getElementById("setup-confirm");
  const method = document.getElementById("setup-method");
  const methodValue = () => method.querySelector("input:checked").value;
  const button = document.getElementById("setup-submit");
  const cancel = document.getElementById("setup-cancel");
  const progress = document.getElementById("setup-progress");
  let heldProof = "", firstFlow = null, backupFlow = null, backupStart = null, expiry, generation = 0;
  function clearSecrets() { proof.value = ""; password.value = ""; confirm.value = ""; }
  function fail(message) { error.textContent = message; error.hidden = false; error.focus(); }
  function announce(message) { progress.textContent = message; progress.hidden = false; progress.focus(); }
  function mode() {
    const passkey = methodValue() === "passkey";
    document.getElementById("setup-password-fields").hidden = passkey;
    document.getElementById("setup-passkey-fields").hidden = !passkey;
    password.required = confirm.required = !passkey;
    password.disabled = confirm.disabled = passkey;
    for (const id of ["setup-primary", "setup-backup"]) {
      const field = document.getElementById(id); field.required = passkey; field.disabled = !passkey;
    }
    button.textContent = passkey ? "Enroll primary passkey" : "Create administrator";
  }
  function lock(locked) {
    for (const field of form.querySelectorAll("input,select")) field.disabled = locked;
    if (!locked) mode();
    cancel.hidden = !locked;
  }
  function reset() {
    generation += 1; clearTimeout(expiry); clearSecrets(); heldProof = "";
    firstFlow = backupFlow = backupStart = null; lock(false); progress.hidden = true;
  }
  function complete() {
    reset(); form.hidden = true;
    const done = document.getElementById("setup-complete"); done.hidden = false; done.focus();
  }
  const post = (action, body) => RiAuth.post(`api/setup/passkey/${action}`, { proof: heldProof, ...body });
  async function cancelFlow() {
    const flow = backupFlow || firstFlow;
    if (flow && !await flow.cancel()) return false;
    if (backupStart) {
      try { await post("cancel", { ceremony: backupStart.ceremony }); }
      catch { /* A lost cancellation reply cannot extend the server's expiry. */ }
    }
    return true;
  }
  function deadline(started) {
    clearTimeout(expiry);
    expiry = setTimeout(async () => {
      if (await cancelFlow()) {
        reset(); fail("Passkey setup expired. Enter your proof and start again. Your administrator has not been created by this enrollment.");
      }
    }, started.expires_in * 1000);
    return started;
  }
  method.addEventListener("change", () => { password.value = confirm.value = ""; error.hidden = true; mode(); });
  cancel.addEventListener("click", () => RiAuth.inFlight(cancel, async () => {
    if (!await cancelFlow()) { announce("Verification is already submitted. Wait for its result before starting again."); return; }
    reset(); announce("Passkey setup stopped. Enter your proof to start again or choose a password. Unfinished server enrollment expires automatically.");
  }));
  window.addEventListener("pagehide", () => {
    // No proof or ceremony is put into URLs, persistent storage, or navigation requests.
    void cancelFlow(); reset();
  });
  window.addEventListener("pageshow", (event) => { if (event.persisted) reset(); });
  mode();
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!form.reportValidity()) return;
    if (methodValue() === "password" && password.value !== confirm.value) { fail("Your passwords do not match."); return; }
    RiAuth.inFlight(button, async () => {
      error.hidden = true;
      const thisRun = generation;
      if (methodValue() === "passkey") {
        if (!RiAuth.passkeysAvailable()) { fail("Use a browser that supports passkeys on a secure connection."); return; }
        if (!firstFlow) {
          const account = {
            username: document.getElementById("setup-username").value,
            display_name: document.getElementById("setup-name").value,
            email: document.getElementById("setup-email").value || null,
            primary_name: document.getElementById("setup-primary").value.trim(),
            backup_name: document.getElementById("setup-backup").value.trim()
          };
          if (!account.primary_name || !account.backup_name || account.primary_name === account.backup_name) {
            fail("Name the primary and backup passkeys differently."); return;
          }
          heldProof = proof.value.trim(); clearSecrets(); lock(true);
          firstFlow = RiAuth.passkeyFlow(
            async () => deadline(await post("start", { account })),
            (credential, started) => post("first", { ceremony: started.ceremony, credential }),
            true, (started) => post("cancel", { ceremony: started.ceremony }));
        }
        try {
          if (!backupStart) {
            const next = await firstFlow();
            if (thisRun !== generation) return;
            backupStart = deadline(next);
            button.textContent = "Enroll backup passkey and create administrator";
            announce("Primary passkey verified. Your account is still pending. Enroll your backup on a separate device or security key.");
          } else {
            backupFlow ||= RiAuth.passkeyFlow(() => Promise.resolve(backupStart),
              (credential, started) => post("finish", { ceremony: started.ceremony, credential }),
              true, (started) => post("cancel", { ceremony: started.ceremony }));
            await backupFlow();
            if (thisRun === generation) complete();
          }
        } catch (e) {
          if (thisRun !== generation) return;
          if (e.name === "NotAllowedError") { fail("Passkey prompt closed. Select the enrollment button again to retry, or cancel setup."); return; }
          // Submitted responses are never retried. The proof survives an unsuccessful
          // ceremony, but it must be entered again to explicitly start a new one.
          await cancelFlow(); reset();
          if (e.status === 409 && /Instance already initialized/i.test(e.description || "")) complete();
          else if (e.status === 401) fail(e.description || "Setup expired. Enter your proof and start again.");
          else fail(e.description || "We could not confirm setup. Try signing in first. If setup is still needed, enter your proof and start again.");
        }
        return;
      }
      const input = {
        proof: proof.value.trim(), username: document.getElementById("setup-username").value,
        display_name: document.getElementById("setup-name").value,
        email: document.getElementById("setup-email").value || null, password: password.value
      };
      clearSecrets();
      try {
        await RiAuth.post("api/setup", input);
        complete();
      } catch (e) {
        if (e.status === 409) {
          complete(); fail("Setup has already completed. Continue to sign in with the administrator account.");
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
