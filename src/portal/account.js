"use strict";
// Email links carry their one-time proof in a fragment. Viewing this page, including
// by a mail scanner, never spends it. Only the form's user submission sends a POST.
(() => {
  const $ = (id) => document.getElementById(id);
  const acceptPath = `${RiAuth.base}account/accept`;
  const verifyPath = `${RiAuth.base}account/verify`;
  const mode = location.pathname === acceptPath ? "accept" : location.pathname === verifyPath ? "verify" : null;
  const form = $("account-form");
  const button = $("account-submit");
  const password = $("account-password");
  const confirm = $("account-confirm");
  const error = $("account-error");
  let token = new URLSearchParams(location.hash.slice(1)).get("token");
  // Remove the proof from the address bar and browser history before showing actions.
  if (location.hash) history.replaceState(history.state, "", `${location.pathname}${location.search}`);

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
  function issue(error) {
    switch (error.code) {
      case "account_code_expired":
        return [mode === "accept" ? "This invitation has expired. Ask your administrator for a new invitation." : "This verification link has expired. Request a new verification email.", true];
      case "account_code_revoked":
        return [mode === "accept" ? "This invitation was revoked. Ask your administrator if you still need access." : "This verification link was revoked. Request a new verification email.", true];
      case "account_code_used":
        return [mode === "accept" ? "This invitation has already been accepted. Continue to sign in." : "This verification link has already been used. Your email may already be verified.", true];
      case "account_code_replaced":
        return ["A newer email link replaced this one. Open the most recent email and try again.", true];
      case "account_code_invalid":
        return ["This email link is invalid. Open the original email link or request a new one.", true];
      case "account_changed":
        return ["The account changed after this link was sent. Request a new link.", true];
      case "access_denied":
        return [mode === "accept" ? "This invitation can no longer be accepted. Ask your administrator for a new invitation." : "This email can no longer be verified with this link. Request a new one.", true];
      case "invalid_request":
        return [error.description || "Check your password and try again.", false];
      default:
        if (error.status === 0) return ["Could not reach riAuth. Check your connection and try again. If the action succeeded, this link will report that it was already used.", false];
        if (error.status === 429) return ["Too many attempts. Wait a minute before trying again.", false];
        return ["Could not complete this request. Try again. If it keeps failing, ask your administrator for a new link.", false];
    }
  }

  if (mode === "accept") {
    document.title = "Accept invitation · riAuth";
    $("account-title").textContent = "Accept your invitation";
    $("account-description").textContent = "Set a password to activate your account. You will sign in after accepting the invitation.";
    $("account-password-fields").hidden = false;
    password.required = true;
    confirm.required = true;
    button.textContent = "Accept invitation";
  } else if (mode === "verify") {
    document.title = "Verify email · riAuth";
    $("account-title").textContent = "Verify your email";
    $("account-description").textContent = "Confirm that this email address belongs to you. This link works once.";
    $("account-fallback").querySelector("a").textContent = "Sign in to request a new link";
    button.textContent = "Verify email";
  }
  if (!mode || !token || !token.startsWith("ri_mail_") || token.length > 128) {
    fail("This email link is missing or invalid. Open the original email link or request a new one.", true);
  } else {
    form.hidden = false;
    $("account-title").focus();
  }

  window.addEventListener("pagehide", () => { token = null; clearSecrets(); });
  window.addEventListener("pageshow", (event) => {
    if (event.persisted) fail("For your security, reopen the original email link to continue.", true);
  });

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!token || !form.reportValidity()) return;
    if (mode === "accept" && password.value !== confirm.value) {
      confirm.setAttribute("aria-invalid", "true");
      fail("Your passwords do not match.");
      return;
    }
    confirm.removeAttribute("aria-invalid");
    RiAuth.inFlight(button, async () => {
      error.hidden = true;
      const input = { token, ...(mode === "accept" ? { password: password.value } : {}) };
      clearSecrets();
      try {
        // A one-time proof POST is never retried automatically.
        await RiAuth.post(`api/portal/account/${mode}`, input);
        token = null;
        form.hidden = true;
        $("account-description").hidden = true;
        $("account-title").textContent = mode === "accept" ? "Invitation accepted" : "Email verified";
        $("account-complete-title").textContent = mode === "accept" ? "Your account is ready" : "You're all set";
        $("account-complete-text").textContent = mode === "accept" ? "Sign in with your new password to open your applications. An application may also require a passkey or authenticator code." : "Your email address has been verified.";
        $("account-complete-link").textContent = mode === "accept" ? "Continue to sign in" : "Open applications";
        $("account-complete").hidden = false;
        $("account-complete").focus();
      } catch (failure) {
        const [message, terminal] = issue(failure);
        fail(message, terminal);
      } finally {
        input.token = "";
        if (input.password) input.password = "";
      }
    });
  });
})();
