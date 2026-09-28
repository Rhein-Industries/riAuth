"use strict";
// Email links carry their one-time proof in a fragment. Viewing this page, including
// by a mail scanner, never spends it. Only the form's user submission sends a POST.
(() => {
  const $ = (id) => document.getElementById(id);
  const paths = { accept: `${RiAuth.base}account/accept`, verify: `${RiAuth.base}account/verify`, reset: `${RiAuth.base}account/reset` };
  const purpose = Object.keys(paths).find((key) => paths[key] === location.pathname) || null;
  const form = $("account-form");
  const button = $("account-submit");
  const password = $("account-password");
  const confirm = $("account-confirm");
  const username = $("account-username");
  const error = $("account-error");
  let token = new URLSearchParams(location.hash.slice(1)).get("token");
  // Remove the proof from the address bar and browser history before showing actions.
  if (location.hash) history.replaceState(history.state, "", `${location.pathname}${location.search}`);
  // Without a proof, the reset page asks for a new link instead.
  const mode = purpose === "reset" && !token ? "request" : purpose;
  const setsPassword = mode === "accept" || mode === "reset";
  const feature = { accept: "identity.invitations", verify: "identity.email_verification", reset: "identity.email_password_reset", request: "identity.email_password_reset" }[mode];
  const FACTORS_KEPT = "Resetting your password never removes your passkeys or authenticator app. If you lost one of them, contact your administrator.";

  function clearSecrets() { password.value = ""; confirm.value = ""; }
  function fail(message, terminal = false) {
    error.textContent = message;
    error.hidden = false;
    if (terminal) {
      token = null;
      form.hidden = true;
      $("account-fallback").hidden = false;
    }
    error.focus();
  }
  function proofIssue(error) {
    const accept = mode === "accept", reset = mode === "reset";
    switch (error.code) {
      case "account_code_expired":
        return [accept ? "This invitation has expired. Ask your administrator for a new invitation." : reset ? "This reset link has expired. Request a new one." : "This verification link has expired. Request a new verification email.", true];
      case "account_code_revoked":
        return [accept ? "This invitation was revoked. Ask your administrator if you still need access." : reset ? "This reset link is no longer valid. Request a new one." : "This verification link was revoked. Request a new verification email.", true];
      case "account_code_used":
        return [accept ? "This invitation has already been accepted. Continue to sign in." : reset ? "This reset link was already used. If that wasn't you, request a new link and tell your administrator." : "This verification link has already been used. Your email may already be verified.", true];
      case "account_code_replaced":
        return ["A newer email link replaced this one. Open the most recent email and try again.", true];
      case "account_code_invalid":
        return ["This email link is invalid. Open the original email link or request a new one.", true];
      case "account_changed":
        return [reset ? "Your account changed after this link was sent, for example its password or email address. Request a new link." : "The account changed after this link was sent. Request a new link.", true];
      case "access_denied":
        return [accept ? "This invitation can no longer be accepted. Ask your administrator for a new invitation." : reset ? "This account can't reset its password with this link. Contact your administrator." : "This email can no longer be verified with this link. Request a new one.", true];
      case "invalid_request":
        return [error.description || "Check your password and try again.", false];
      default:
        if (error.status === 0) return ["Could not reach riAuth. Check your connection and try again. If the action succeeded, this link will report that it was already used.", false];
        if (error.status === 429) return ["Too many attempts. Wait a minute before trying again.", false];
        return ["Could not complete this request. Try again. If it keeps failing, ask your administrator for a new link.", false];
    }
  }
  function requestIssue(error) {
    if (error.code === "delivery_unavailable") return "Password reset by email isn't available on this server. Contact your administrator.";
    if (error.status === 429) return "Too many requests. Wait a minute before trying again.";
    if (error.code === "invalid_request") return "Enter the username you sign in with.";
    if (error.status === 0) return "Could not reach riAuth. Check your connection and try again.";
    return "Could not request a reset link. Try again later.";
  }

  const fallback = $("account-fallback").querySelector("a");
  if (mode === "accept") {
    document.title = "Accept invitation · riAuth";
    $("account-title").textContent = "Accept your invitation";
    $("account-description").textContent = "Set a password to activate your account. You will sign in after accepting the invitation.";
    button.textContent = "Accept invitation";
  } else if (mode === "verify") {
    document.title = "Verify email · riAuth";
    $("account-title").textContent = "Verify your email";
    $("account-description").textContent = "Confirm that this email address belongs to you. This link works once.";
    fallback.textContent = "Sign in to request a new link";
    button.textContent = "Verify email";
  } else if (mode === "reset") {
    document.title = "Reset password · riAuth";
    $("account-title").textContent = "Choose a new password";
    $("account-description").textContent = "Set a new password for your account. This link works once, and saving signs you out everywhere.";
    $("account-note").textContent = FACTORS_KEPT;
    fallback.textContent = "Request a new reset link";
    fallback.href = paths.reset;
    fallback.dataset.capability = "identity.email_password_reset";
    button.textContent = "Reset password";
  } else if (mode === "request") {
    document.title = "Reset password · riAuth";
    $("account-title").textContent = "Reset your password";
    $("account-description").textContent = "Enter your username. If the account can reset its password, we'll email a one-use link to its verified address. The link expires after 30 minutes.";
    $("account-note").textContent = FACTORS_KEPT;
    $("account-username-field").hidden = false;
    username.required = true;
    button.textContent = "Email me a reset link";
  }
  $("account-note").hidden = !$("account-note").textContent;
  $("account-password-fields").hidden = !setsPassword;
  password.required = confirm.required = setsPassword;
  if (mode === "request") {
    // A new email needs configured delivery; an existing proof does not.
    form.hidden = true;
  } else if (!mode || !token || !token.startsWith("ri_mail_") || token.length > 128) {
    fail("This email link is missing or invalid. Open the original email link or request a new one.", true);
  } else {
    form.hidden = false;
    $("account-title").focus();
  }

  async function refreshCapabilities() {
    let state = null;
    try { state = await RiAuthCapabilities.refresh(); } catch { /* Proof redemption can still work if this read fails. */ }
    RiAuthCapabilities.apply();
    if (mode === "request" && $("account-complete").hidden && error.hidden) {
      if (!state) { fail("Could not check whether email recovery is available. Reload this page and try again.", true); return; }
      if (!RiAuthCapabilities.usable(feature)) { fail("Password reset by email isn't available on this server. Contact your administrator.", true); return; }
      form.hidden = false;
      username.focus();
    } else if (mode !== "request" && state && feature && !RiAuthCapabilities.compiled(feature) && !form.hidden) {
      fail("This account action isn't available on this server. Contact your administrator.", true);
    }
  }
  void refreshCapabilities();

  window.addEventListener("pagehide", () => { token = null; clearSecrets(); });
  // A newer email link opened in this tab changes only the fragment; start over with it.
  window.addEventListener("hashchange", () => {
    if (new URLSearchParams(location.hash.slice(1)).has("token")) location.reload();
  });
  window.addEventListener("pageshow", (event) => {
    if (event.persisted && mode !== "request") fail("For your security, reopen the original email link to continue.", true);
    if (event.persisted && mode === "request" && $("account-complete").hidden) {
      form.hidden = true;
      error.hidden = true;
      void refreshCapabilities();
    }
  });

  function complete(title, heading, text, link) {
    form.hidden = true;
    error.hidden = true;
    $("account-description").hidden = true;
    $("account-title").textContent = title;
    $("account-complete-title").textContent = heading;
    $("account-complete-text").textContent = text;
    $("account-complete-link").textContent = link;
    $("account-complete").hidden = false;
    $("account-complete").focus();
  }
  function requestLink() {
    if (!RiAuthCapabilities.usable("identity.email_password_reset")) return;
    const name = username.value.trim();
    if (!name) { username.setAttribute("aria-invalid", "true"); fail("Enter the username you sign in with."); return; }
    username.removeAttribute("aria-invalid");
    RiAuth.inFlight(button, async () => {
      error.hidden = true;
      try {
        // Every account gets the same answer, so it never says whether one can be reset.
        await RiAuth.post("api/portal/account/reset-request", { username: name });
        complete("Check your email", "Reset link requested",
          `If ${name} has a verified email address and a password here, we emailed a one-use reset link. It expires in 30 minutes. Accounts that sign in through your organization's directory or only with passkeys have no password to reset here.`,
          "Back to sign in");
      } catch (failure) { fail(requestIssue(failure)); }
    });
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (mode === "request") { requestLink(); return; }
    if (!token || !form.reportValidity()) return;
    if (setsPassword && password.value !== confirm.value) {
      confirm.setAttribute("aria-invalid", "true");
      fail("Your passwords do not match.");
      return;
    }
    confirm.removeAttribute("aria-invalid");
    RiAuth.inFlight(button, async () => {
      error.hidden = true;
      const input = { token, ...(setsPassword ? { password: password.value } : {}) };
      clearSecrets();
      try {
        // A one-time proof POST is never retried automatically.
        const result = await RiAuth.post(`api/portal/account/${mode}`, input);
        token = null;
        if (mode === "accept") complete("Invitation accepted", "Your account is ready", "Sign in with your new password to open your applications. An application may also require a passkey or authenticator code.", "Continue to sign in");
        else if (mode === "reset") complete("Password reset", "Sign in with your new password", result?.factors_reset
          ? "Every session on your account was signed out. Your previous passkeys and authenticator app were removed. Enroll new factors after signing in."
          : "Every session on your account was signed out. Your passkeys and authenticator app are unchanged; if you use an authenticator app, signing in still asks for its code.", "Continue to sign in");
        else complete("Email verified", "You're all set", "Your email address has been verified.", "Open applications");
      } catch (failure) {
        const [message, terminal] = proofIssue(failure);
        fail(message, terminal);
      } finally {
        input.token = "";
        if (input.password) input.password = "";
      }
    });
  });
})();
