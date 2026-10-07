// Headless authenticator-app enrollment and recovery-code sign-in on a 390×844 CSS viewport.
//
// Already covered elsewhere, and not repeated here:
// - password plus an authenticator app the fixture already enrolled (signin.spec.js)
// - passkey enrollment, passwordless sign-in, and revocation (signin.spec.js, passkey-revocation.spec.js)
// - email password reset, which this fixture cannot deliver (accessibility-journeys.spec.js)
//
// Bob starts with a password only. This journey enrolls an authenticator app from the setup
// key shown on the page, then signs in with a recovery code. Enrollment ends the other
// browser session. That recovery code works once; a second unused code still works and the
// remaining count drops. The setup key is read from the page and the six-digit code is
// computed here. This is not a phone authenticator app, a physical device, or iOS or Android.
// The viewport is CSS only. Hardware authenticators, real mobile devices, and email recovery
// stay manual or unavailable.
import { test, expect } from '@playwright/test';
import { fixtureStartupMs, startFixture } from './fixture.js';
import { totp, wrongCode } from './totp.js';

const VIEWPORT = { width: 390, height: 844 };
const SIGN_IN_ERROR = 'Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes.';
const CSP = /content.security.policy|csp violation|refused to (load|execute|apply|connect|frame)/i;

test.use({ viewport: VIEWPORT, headless: true });

let fixture, stopFixture;
test.beforeAll(async () => {
  test.setTimeout(fixtureStartupMs + 5000);
  ({ fixture, stop: stopFixture } = await startFixture());
});
test.afterAll(async () => { await stopFixture?.(); });

function watch(context, problems) {
  context.on('console', (message) => {
    if (['error', 'warning'].includes(message.type()) && CSP.test(message.text())) problems.push(`${message.type()}: ${message.text()}`);
  });
  context.on('weberror', (error) => problems.push(`page error: ${error.error().message}`));
}
async function arm(context, problems) {
  await context.addInitScript(() => document.addEventListener('securitypolicyviolation',
    (event) => console.error(`CSP violation: ${event.violatedDirective} blocked ${event.blockedURI}`)));
  watch(context, problems);
}
const sso = async (context) => (await context.cookies()).find((cookie) => /riauth_sso$/.test(cookie.name));
const portalStatus = (page) => page.evaluate((url) => fetch(url, { credentials: 'same-origin' }).then((response) => response.status), `${fixture.issuer}/api/portal`);
async function fits(page) {
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1)).toBe(true);
}
// WebKit on macOS moves focus only between text fields on Tab, like Safari; Option-Tab
// reaches every control there. Other engines and WebKit on Linux use Tab.
const tabKey = (browserName, backward = false) => {
  const option = browserName === 'webkit' && process.platform === 'darwin';
  if (backward) return option ? 'Alt+Shift+Tab' : 'Shift+Tab';
  return option ? 'Alt+Tab' : 'Tab';
};
async function focusedId(page) {
  return page.evaluate(() => document.activeElement?.id ?? '');
}
async function focusLabel(page) {
  return page.evaluate(() => {
    const active = document.activeElement;
    if (!active || active === document.body) return 'BODY';
    if (active === document.documentElement) return 'HTML';
    const label = active.getAttribute('aria-label');
    const cls = typeof active.className === 'string' && active.className ? `.${active.className.trim().split(/\s+/)[0]}` : '';
    return `${active.tagName}${active.id ? `#${active.id}` : ''}${cls}${label ? `[${label}]` : ''}`;
  });
}
// After sign-in, WebKit can leave focus on the document and then ignore Option-Tab.
// Put focus on the skip link, which is in the page tab order, and continue from there.
async function releaseHiddenFocus(page) {
  await page.evaluate(() => {
    const active = document.activeElement;
    const rendered = active && active !== document.body && active !== document.documentElement && active.getClientRects().length > 0;
    if (rendered) return;
    document.querySelector('.skip-link')?.focus();
  });
}
async function moveFocus(page, browserName, id, backward, limit) {
  await releaseHiddenFocus(page);
  const key = tabKey(browserName, backward);
  const seen = [];
  for (let i = 0; i <= limit; i += 1) {
    const current = await focusedId(page);
    if (current === id) return;
    if (i === limit) throw new Error(`${key} did not reach #${id}. Focus path: ${seen.join(' -> ') || '(none)'}`);
    seen.push(await focusLabel(page));
    await page.keyboard.press(key);
    await page.waitForFunction(
      (previous) => (document.activeElement?.id ?? '') !== previous,
      current,
      { timeout: 250 },
    ).catch(() => {});
  }
}
async function tabTo(page, browserName, id, limit = 40) {
  await moveFocus(page, browserName, id, false, limit);
}
async function tabBack(page, browserName, id, limit = 15) {
  await moveFocus(page, browserName, id, true, limit);
}
async function tabToControl(page, browserName, id, limit = 40) {
  await tabTo(page, browserName, id, limit);
}
async function within(locator) {
  const box = await locator.boundingBox();
  expect(box, 'control is inside the 390×844 viewport').not.toBeNull();
  expect(box.x).toBeGreaterThanOrEqual(0);
  expect(box.y).toBeGreaterThanOrEqual(0);
  expect(box.x + box.width).toBeLessThanOrEqual(VIEWPORT.width + 1);
  expect(box.y + box.height).toBeLessThanOrEqual(VIEWPORT.height + 1);
  return box;
}
async function keyboardSignIn(page, browserName, user, otp = '') {
  await expect(page.locator('#auth')).toBeVisible();
  await tabTo(page, browserName, 'login-username', 30);
  await page.locator('#login-username').fill(user.username);
  await tabTo(page, browserName, 'login-password', 10);
  await page.locator('#login-password').fill(user.password);
  const recovery = otp.startsWith('ri_recovery_');
  if ((await page.locator('#login-otp').getAttribute('data-otp-kind') === 'recovery') !== recovery) {
    await tabTo(page, browserName, 'login-otp-mode', 10);
    await page.keyboard.press('Enter');
  } else await tabTo(page, browserName, 'login-otp', 10);
  await expect(page.locator('#login-otp')).toBeFocused();
  await expect(page.locator('#login-otp')).toHaveAttribute('inputmode', recovery ? 'text' : 'numeric');
  await page.locator('#login-otp').fill(otp);
  await expect(page.locator('#login-otp')).toHaveValue(otp);
  await tabTo(page, browserName, 'password-login', 10);
  expect((await within(page.locator('#password-login'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
}
async function expectSignedIn(page, browserName, { mfa }) {
  await expect(page.locator('#auth')).toBeHidden();
  if (new URL(page.url()).pathname === new URL(`${fixture.issuer}/apps/security`).pathname) {
    await expect(page.locator('#security-page')).toBeVisible();
    await expect(page.locator('#catalogue')).toBeHidden();
    await expect(page.locator('#security-title')).toBeFocused();
    await tabToControl(page, browserName, 'security-close', 60);
    await page.keyboard.press('Enter');
    await expect(page).toHaveURL(`${fixture.issuer}/apps`);
    await expect(page.locator('#security-page')).toBeHidden();
  }
  await expect(page.locator('#catalogue')).toBeVisible();
  await expect(page.locator('#account-name')).toHaveText('Bob Example');
  await expect(page.locator('#auth')).toBeHidden();
  if (mfa) await expect(page.locator('#mfa-notice')).toBeHidden();
  else {
    await expect(page.locator('#mfa-notice')).toBeVisible();
    await expect(page.locator('#mfa-notice-text')).toHaveText('Some applications need extra verification. Add a passkey or an authenticator app under Sign-in and security.');
  }
  expect(await portalStatus(page)).toBe(200);
  await fits(page);
}
async function expectRejected(page) {
  const error = page.locator('#auth-error');
  await expect(error).toBeFocused();
  await expect(error).toHaveText(SIGN_IN_ERROR);
  await expect(page.locator('#login-username')).toHaveValue('bob');
  for (const id of ['login-username', 'login-password', 'login-otp']) {
    await expect(page.locator(`#${id}`)).toHaveAttribute('aria-invalid', 'true');
  }
  await expect(page.locator('#login-password')).toHaveValue('');
  await expect(page.locator('#login-otp')).toHaveValue('');
  await expect(page.locator('#catalogue')).toBeHidden();
  expect(await portalStatus(page)).toBe(401);
  await within(error);
  await fits(page);
}
async function openSecurity(page, browserName) {
  // The sidebar precedes the main page in document order. Walk backwards
  // from a page heading/form instead of relying on Tab wrapping at the end.
  const fromMain = await page.evaluate(() => Boolean(document.activeElement?.closest('main')));
  if (fromMain) await tabBack(page, browserName, 'account-security', 50);
  else await tabTo(page, browserName, 'account-security', 50);
  expect((await within(page.locator('#account-security'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
  const securityPage = page.locator('#security-page');
  await expect(securityPage).toBeVisible();
  await expect(page).toHaveURL(`${fixture.issuer}/apps/security`);
  await expect(page.locator('#catalogue')).toBeHidden();
  await expect(page.locator('#security-account')).toHaveText('Bob Example (@bob)');
  await within(page.locator('#security-title'));
  await fits(page);
}
async function expectRecoveryCount(page, browserName, left) {
  await openSecurity(page, browserName);
  await expect(page.locator('#security-status')).toHaveText('You have no passkeys yet.');
  await expect(page.locator('#reauth-panel')).toBeHidden();
  await expect(page.locator('#totp-summary')).toHaveText('On. After your password, enter the 6-digit code your authenticator app shows.');
  await expect(page.locator('#recovery-summary')).toHaveText(`${left} of 10 recovery codes left. Each one signs you in once instead of an app code.`);
  await tabToControl(page, browserName, 'recovery-rotate', 30);
  expect((await within(page.locator('#recovery-rotate'))).height).toBeGreaterThanOrEqual(24);
  await within(page.locator('#recovery-summary'));
  await page.keyboard.press('Escape');
  await expect(page.locator('#security-page')).toBeVisible();
  await expect(page).toHaveURL(`${fixture.issuer}/apps/security`);
  await tabToControl(page, browserName, 'security-close', 50);
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(`${fixture.issuer}/apps`);
  await expect(page.locator('#catalogue')).toBeVisible();
}

test('enrolling an authenticator app ends every session and a recovery code signs in once', async ({ context, browser, browserName }) => {
  test.setTimeout(120000);
  test.skip(test.info().project.use.headless === false, 'This journey stays headless and does not take the shared desktop');
  const problems = [];
  await arm(context, problems);
  const page = await context.newPage();
  const user = fixture.users.bob;
  await page.goto(`${fixture.issuer}/apps`);
  await expect(page.locator('#auth')).toBeVisible();
  await fits(page);

  const other = await browser.newContext({ viewport: VIEWPORT });
  try {
    await arm(other, problems);
    const otherPage = await other.newPage();
    await keyboardSignIn(page, browserName, user);
    await expectSignedIn(page, browserName, { mfa: false });
    await otherPage.goto(`${fixture.issuer}/apps`);
    await keyboardSignIn(otherPage, browserName, user);
    await expectSignedIn(otherPage, browserName, { mfa: false });
    const otherSession = await sso(other);
    expect(otherSession?.value).toMatch(/^ri_sso_/);

    // Give browser Back two distinct workspace entries to traverse.
    await page.locator('#workspace-settings').click();
    await expect(page).toHaveURL(`${fixture.issuer}/apps/settings`);
    await expect(page.locator('#settings-page')).toBeVisible();
    await openSecurity(page, browserName);
    await expect(page.locator('#security-status')).toHaveText('You have no passkeys yet.');
    await expect(page.locator('#totp-summary')).toHaveText('Off. Use an authenticator app on your phone or computer for 6-digit sign-in codes after your password.');
    await expect(page.locator('#recovery-section')).toBeHidden();
    // Cancellation must finish before a new page becomes active. A second
    // Back while the real cancellation is held must settle on the latest
    // history entry, with neither a leaked setup key nor a mismatched page.
    await tabToControl(page, browserName, 'totp-setup', 20);
    await page.keyboard.press('Enter');
    await expect(page.locator('#totp-enroll-title')).toBeFocused();
    const discardedKey = (await page.locator('#totp-key').innerText()).replace(/\s+/g, '');
    expect(discardedKey).toMatch(/^[A-Z2-7]+=*$/);
    let releaseCancel, enteredCancel, cancelRequests = 0;
    const mayCancel = new Promise((resolve) => { releaseCancel = resolve; });
    const cancelling = new Promise((resolve) => { enteredCancel = resolve; });
    const cancelPattern = '**/api/portal/mfa/totp/cancel';
    const holdCancel = async (route) => {
      cancelRequests += 1;
      enteredCancel();
      await mayCancel;
      await route.continue();
    };
    await page.route(cancelPattern, holdCancel);
    const cancelled = page.waitForResponse((response) => response.request().method() === 'POST'
      && new URL(response.url()).pathname.endsWith('/api/portal/mfa/totp/cancel'));
    try {
      await page.goBack();
      await cancelling;
      await page.goBack();
      await expect(page).toHaveURL(`${fixture.issuer}/apps`);
      await expect(page.locator('#totp-key')).toBeEmpty();
      await expect(page.locator('#totp-enroll')).toBeHidden();
      await expect(page.locator('#security-page')).toBeVisible();
      await expect(page.locator('#catalogue')).toBeHidden();
    } finally { releaseCancel(); }
    expect((await cancelled).status()).toBe(200);
    await page.unroute(cancelPattern, holdCancel);
    await expect(page.locator('#catalogue')).toBeVisible();
    await expect(page.locator('#page-title')).toBeFocused();
    await expect(page.locator('#security-page')).toBeHidden();
    await expect(page.locator('#settings-page')).toBeHidden();
    await expect(page).toHaveURL(`${fixture.issuer}/apps`);
    expect(cancelRequests).toBe(1);
    await openSecurity(page, browserName);
    await tabToControl(page, browserName, 'totp-setup', 20);
    expect((await within(page.locator('#totp-setup'))).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    await expect(page.locator('#totp-enroll-title')).toBeFocused();
    await expect(page.locator('#totp-enroll-title')).toHaveText('Set up your authenticator app');
    await expect(page.locator('#totp-key-details')).toHaveText('Time-based, 6 digits, new code every 30 seconds (SHA1). Spaces in the key are optional.');
    const secret = (await page.locator('#totp-key').innerText()).replace(/\s+/g, '');
    expect(secret).toMatch(/^[A-Z2-7]+=*$/);
    expect(secret).not.toBe(discardedKey);
    await within(page.locator('#totp-enroll-title'));

    await tabTo(page, browserName, 'totp-code', 15);
    await page.keyboard.press('Enter');
    const enrollError = page.locator('#totp-error');
    await expect(enrollError).toBeFocused();
    await expect(enrollError).toHaveText('Enter the 6-digit code your authenticator app shows.');
    await expect(page.locator('#totp-code')).toHaveAttribute('aria-invalid', 'true');
    await within(enrollError);
    await fits(page);
    expect(await portalStatus(page)).toBe(200);
    expect(await portalStatus(otherPage)).toBe(200);

    await tabBack(page, browserName, 'totp-code');
    await page.locator('#totp-code').fill(wrongCode(secret));
    await tabTo(page, browserName, 'totp-confirm', 10);
    expect((await within(page.locator('#totp-confirm'))).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    await expect(enrollError).toBeFocused();
    await expect(enrollError).toHaveText("That code didn't match. Enter the newest code from your app.");
    await expect(page.locator('#totp-code')).toHaveValue('');
    await expect(page.locator('#totp-code')).toHaveAttribute('aria-invalid', 'true');
    await expect(page.locator('#totp-code')).toBeEnabled();
    await within(enrollError);
    expect(await portalStatus(page)).toBe(200);
    expect((await sso(other))?.value).toBe(otherSession.value);

    // The confirming step is spent by enrollment, so later sign-in uses a recovery code.
    await tabBack(page, browserName, 'totp-code');
    await page.locator('#totp-code').fill(totp(secret));
    await tabTo(page, browserName, 'totp-confirm', 10);
    await page.keyboard.press('Enter');
    await expect(page.locator('#toast')).toHaveText('Authenticator app turned on. Sign in with your password and a code from it.');
    await expect(page.locator('#auth-description')).toHaveText('Your sessions have ended. Sign in again to return to your applications.');
    await expect(page.locator('#auth')).toBeHidden();
    await expect(page.locator('#catalogue')).toBeHidden();
    await expect.poll(async () => Boolean(await sso(context))).toBe(false);
    expect(await portalStatus(page)).toBe(401);
    await expect.poll(() => portalStatus(otherPage)).toBe(401);
    expect((await sso(other))?.value).toBe(otherSession.value);

    const codesPage = page.locator('#codes-page');
    await expect(codesPage).toBeVisible();
    await expect(page.locator('#security-content')).toBeHidden();
    await expect(page.locator('#codes-title')).toBeFocused();
    await expect(page.locator('#codes-title')).toHaveText('Authenticator app turned on');
    await expect(page.locator('#codes-description')).toHaveText("You're signed out everywhere. Sign in again with your password and a code from your app. Save these recovery codes somewhere safe, like a password manager. Each one signs you in once if you can't use your authenticator app. They won't be shown again.");
    const codes = (await page.locator('#codes-list li').allTextContents()).map((code) => code.trim());
    expect(codes).toHaveLength(10);
    expect(new Set(codes).size).toBe(10);
    for (const code of codes) expect(code).toMatch(/^ri_recovery_[A-Za-z0-9_-]{40,}$/);
    await within(page.locator('#codes-title'));
    await fits(page);
    await tabTo(page, browserName, 'codes-done', 15);
    await expect(page.locator('#codes-done')).not.toHaveAttribute('aria-disabled', 'true');
    expect((await within(page.locator('#codes-done'))).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    await expect(page.locator('#codes-saved')).toBeFocused();
    await expect(page.locator('#codes-status')).toHaveText('Save the codes, then confirm that you saved them.');
    await expect(codesPage).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(codesPage).toBeVisible();
    const codesRoute = page.url();
    await page.goBack();
    await expect(page).toHaveURL(codesRoute);
    await expect(codesPage).toBeVisible();
    await expect(page.locator('#codes-saved')).toBeFocused();
    await expect(page.locator('#codes-status')).toHaveText("Save your recovery codes before leaving. They won't be shown again.");
    await within(page.locator('label.checkbox'));
    await page.keyboard.press('Space');
    await expect(page.locator('#codes-saved')).toBeChecked();
    await tabTo(page, browserName, 'codes-done', 10);
    await expect(page.locator('#codes-done')).not.toHaveAttribute('aria-disabled', 'true');
    await page.keyboard.press('Enter');
    await expect(codesPage).toBeHidden();
    await expect(page.locator('#codes-list')).toBeEmpty();
    await expect(page.locator('#auth')).toBeVisible();

    await otherPage.reload();
    await expect(otherPage.locator('#auth')).toBeVisible();
    await expect(otherPage.locator('#catalogue')).toBeHidden();
    expect((await sso(other))?.value).toBe(otherSession.value);

    await keyboardSignIn(page, browserName, user);
    await expectRejected(page);
    expect(await sso(context)).toBeUndefined();

    await keyboardSignIn(page, browserName, user, codes[0]);
    await expectSignedIn(page, browserName, { mfa: true });
    const recovered = await sso(context);
    expect(recovered?.value).toMatch(/^ri_sso_/);
    await expectRecoveryCount(page, browserName, 9);

    await keyboardSignIn(otherPage, browserName, user, codes[0]);
    await expectRejected(otherPage);
    expect((await sso(other))?.value).toBe(otherSession.value);
    await keyboardSignIn(otherPage, browserName, user, codes[1]);
    await expectSignedIn(otherPage, browserName, { mfa: true });
    expect((await sso(other))?.value).toMatch(/^ri_sso_/);
    expect((await sso(other))?.value).not.toBe(otherSession.value);
    await expectRecoveryCount(otherPage, browserName, 8);
    // A recovery-code sign-in does not end the browser that already signed in with one.
    expect((await sso(context))?.value).toBe(recovered.value);
    expect(await portalStatus(page)).toBe(200);
    await expectRecoveryCount(page, browserName, 8);
    expect(problems, 'CSP violations and page errors').toEqual([]);
  } finally {
    await other.close();
  }
});
