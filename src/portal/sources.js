"use strict";
// Continues a sign-in or link that this browser started with an upstream provider. The
// login's one-use credential stays in an HttpOnly cookie: this page only asks the server what
// finishing would do, shows it, and confirms or cancels. Nothing secret is in its address.
(() => {
  const $ = (id) => document.getElementById(id);
  const { base } = RiAuth;
  const security = { href: `${base}account/security`, label: "Back to account security" };
  let review = null;

  function end(title, text, link = { href: `${base}apps`, label: "Back to sign-in" }) {
    review = null;
    $("review").hidden = true;
    $("review-title").textContent = title;
    $("review-intro").textContent = "";
    $("review-end-text").textContent = text;
    $("review-end-link").href = link.href;
    $("review-end-link").textContent = link.label;
    $("review-end").hidden = false;
    $("review-title").focus();
  }
  function problem(text) {
    $("review-error").textContent = text;
    $("review-error").hidden = false;
    $("review-error").focus();
  }
  // Every server answer that ends this login has its own explanation. Before the review is
  // shown there is no form to report into, so any other failure ends the page too.
  function failed(error, linking, loading = false) {
    const back = linking ? security : undefined;
    const report = loading ? (text) => end("riAuth couldn't continue this sign-in", text, back) : problem;
    if (error.code === "source_login_expired") return end("This sign-in has ended", "It expired, was cancelled, or has already finished. Start again when you're ready.", back);
    if (error.code === "source_login_pending") return end("Your provider hasn't finished yet", "Finish at the provider, or start again.", back);
    if (error.code === "account_changed") return end("Your signed-in account changed", "The provider was not linked. Start linking again from account security.", security);
    if (error.code === "access_denied") return end("riAuth couldn't finish this sign-in", error.description || "Your provider account wasn't accepted.", back);
    if (error.status === 0) return report("Couldn't reach riAuth. Check your connection and try again.");
    if (error.status === 429) return report("Too many attempts. Wait a minute, then try again.");
    return report(error.description || "This sign-in couldn't be completed. Try again.");
  }
  const person = (user) => `${user.display_name} (@${user.username})`;

  function show(data) {
    const upstream = [data.name, data.email].filter(Boolean).join(" · ") || data.subject;
    $("review-provider").textContent = data.provider;
    $("review-upstream").textContent = data.email && data.email_verified === false ? `${upstream} (email not verified by the provider)` : upstream;
    $("review-local-label").textContent = "riAuth account";
    let confirm = "Sign in";
    if (data.linking) {
      $("review-eyebrow").textContent = "LINK A PROVIDER";
      $("review-title").textContent = `Link ${data.provider} to your account?`;
      $("review-intro").textContent = `After linking, you can sign in to riAuth with this ${data.provider} account.`;
      $("review-local").textContent = person(data.local_user);
      confirm = "Link account";
    } else if (data.local_user) {
      $("review-title").textContent = `Continue as ${data.local_user.display_name}?`;
      $("review-intro").textContent = `${data.provider} confirmed who you are. riAuth will sign you in to this account.`;
      $("review-local").textContent = person(data.local_user);
    } else if (data.auto_provision) {
      $("review-title").textContent = "Create your riAuth account?";
      $("review-intro").textContent = `No riAuth account uses this ${data.provider} account yet. Continuing creates one for it.`;
      $("review-local-label").textContent = "New riAuth account";
      $("review-local").textContent = "Created when you continue";
      confirm = "Create account and sign in";
    } else {
      // Nothing can finish: the provider account is not linked and cannot be created here.
      void RiAuth.post("api/portal/sources/finish", { approve: false }).catch(() => {});
      end(`No riAuth account uses this ${data.provider} account`,
        `Sign in with your password or passkey first, then link ${data.provider} from account security. An administrator can also link it for you.`);
      return;
    }
    $("review-otp-field").hidden = !data.local_otp_required;
    $("review-confirm").textContent = confirm;
    $("review-error").hidden = true;
    review = data;
    $("review").hidden = false;
    (data.local_otp_required ? $("review-otp") : $("review-title")).focus();
  }

  async function load() {
    try {
      const data = await RiAuth.post("api/portal/sources/review", {}, { retry: true });
      if (data.status !== "review") { failed({ code: "source_login_pending" }); return; }
      show(data);
    } catch (error) { failed(error, false, true); }
  }
  async function confirm() {
    if (!review) return;
    const otp = $("review-otp").value.trim();
    if (review.local_otp_required && !otp) { problem("Enter the code from your authenticator app, or a recovery code."); return; }
    await RiAuth.inFlight($("review-confirm"), async () => {
      const linking = review.linking;
      try {
        const result = await RiAuth.post("api/portal/sources/finish", { approve: true, otp: otp || null });
        $("review-otp").value = "";
        location.assign(result.linked ? `${base}account/security?linked=1` : `${base}apps`);
      } catch (error) {
        if (error.code === "invalid_code") { $("review-otp").value = ""; problem(error.description); $("review-otp").focus(); return; }
        failed(error, linking);
      }
    });
  }
  $("review-form").addEventListener("submit", (event) => { event.preventDefault(); void confirm(); });
  RiAuth.guard($("review-confirm"), (event) => { event.preventDefault(); void confirm(); });
  $("review-cancel").addEventListener("click", () => RiAuth.inFlight($("review-cancel"), async () => {
    const linking = review?.linking;
    try { await RiAuth.post("api/portal/sources/finish", { approve: false }); } catch { /* The login expires on its own. */ }
    location.assign(linking ? `${base}account/security` : `${base}apps`);
  }));
  load();
})();
