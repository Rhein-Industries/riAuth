// Headless two-authenticator journey on a 390×844 CSS viewport.
//
// Already covered elsewhere, and not repeated here:
// - removing one passkey and signing back in with the password (passkey-revocation.spec.js)
// - authenticator-app enrollment and recovery-code sign-in (authenticator-recovery.spec.js)
// - renaming a passkey (passkey-rename.spec.js)
// - password-reset denial and replay through the fixture's loopback mail (password-reset-replay.spec.js)
//
// Discoverable sign-in on one Playwright WebAuthn shim returns that shim's first
// resident credential and stops there. riAuth also sends the account's existing
// credential ids as excludeCredentials, so a second create on the same shim fails
// with InvalidStateError. This journey therefore uses two browser contexts. Each
// installs its own shim before its page exists, and each shim holds one resident
// credential. The second context enrolls only after it signs in with the password
// and the next authenticator code. That code is computed from the setup key shown
// on the page. The contexts do not copy the SSO cookie. Removing the first passkey
// ends every session, rejects the credential that shim still holds, and leaves the
// second passkey able to sign in. The authenticator app stays on. A shim is not a
// physical security key, a synced passkey, a phone authenticator, or iOS or Android.
// The viewport is CSS only. This journey does not drive a screen reader. An engine
// whose page has no WebAuthn is skipped. Mail stays off. Hardware authenticators,
// synced passkeys, phone hybrid, real mobile devices, and external email stay open.
import { test, expect } from '@playwright/test';
import { fixtureStartupMs, startFixture } from './fixture.js';
import { hotp, step, wrongCode } from './totp.js';

const VIEWPORT = { width: 390, height: 844 };
const LAPTOP = 'Laptop passkey';
const BACKUP = 'Backup passkey';
const SETUP_NOTICE = 'Some applications need extra verification. Add a passkey or an authenticator app under Sign-in and security.';
const FACTOR_NOTICE = 'Some applications need your passkey or authenticator code.';
const SIGN_IN_ERROR = 'Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes.';
const UNKNOWN = "This passkey isn't registered with riAuth. Use another passkey or sign in with your password.";
const SESSIONS_ENDED = 'Your sessions have ended. Sign in again to return to your applications.';
const TOTP_OFF = 'Off. Use an authenticator app on your phone or computer for 6-digit sign-in codes after your password.';
const TOTP_ON = 'On. After your password, enter the 6-digit code your authenticator app shows.';
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
// Firefox can leave the dialog's last button focused and then ignore a forward Tab.
// Shift+Tab still walks back through the controls that loaded after the dialog opened.
async function tabToControl(page, browserName, id, limit = 40) {
  if (await focusedId(page) === 'security-close') await tabBack(page, browserName, id, limit);
  else await tabTo(page, browserName, id, limit);
}
// Chromium's forward Tab from the document skips #passkey-name and stops on Add a passkey.
// One Shift+Tab from that button reaches the name field. Other engines tab forward to it.
async function tabToPasskeyName(page, browserName) {
  await releaseHiddenFocus(page);
  const forward = tabKey(browserName, false);
  const seen = [];
  for (let i = 0; i <= 15; i += 1) {
    const id = await focusedId(page);
    if (id === 'passkey-name') return;
    if (id === 'add-passkey') {
      await page.keyboard.press(tabKey(browserName, true));
      await page.waitForFunction(() => document.activeElement?.id === 'passkey-name', null, { timeout: 500 }).catch(() => {});
      if (await focusedId(page) === 'passkey-name') return;
      throw new Error(`Shift+Tab from #add-passkey did not reach #passkey-name. Focus: ${await focusLabel(page)}`);
    }
    if (i === 15) throw new Error(`${forward} did not reach #passkey-name. Focus path: ${seen.join(' -> ') || '(none)'}`);
    seen.push(await focusLabel(page));
    await page.keyboard.press(forward);
    await page.waitForFunction(
      (previous) => (document.activeElement?.id ?? '') !== previous,
      id,
      { timeout: 250 },
    ).catch(() => {});
  }
}
// The passkey row buttons have an accessible name and no id. They are above the name
// field. Firefox can leave focus outside the dialog, send the first Tab to #passkey-name,
// and then ignore a further Tab once Close is focused. Shift+Tab walks back through the rows.
async function tabToLabel(page, browserName, label, limit = 50) {
  await releaseHiddenFocus(page);
  let backward = (await focusedId(page)) === 'security-close';
  const seen = [];
  for (let i = 0; i <= limit; i += 1) {
    const current = await focusLabel(page);
    if (current.includes(`[${label}]`)) return;
    if (!backward && (await focusedId(page)) === 'security-close') backward = true;
    const key = tabKey(browserName, backward);
    if (i === limit) throw new Error(`${key} did not reach ${label}. Focus path: ${seen.join(' -> ') || '(none)'}`);
    seen.push(current);
    await page.keyboard.press(key);
    await page.waitForFunction((previous) => {
      const active = document.activeElement;
      if (!active || active === document.body) return 'BODY' !== previous;
      if (active === document.documentElement) return 'HTML' !== previous;
      const aria = active.getAttribute('aria-label');
      const cls = typeof active.className === 'string' && active.className ? `.${active.className.trim().split(/\s+/)[0]}` : '';
      const next = `${active.tagName}${active.id ? `#${active.id}` : ''}${cls}${aria ? `[${aria}]` : ''}`;
      return next !== previous;
    }, current, { timeout: 250 }).catch(() => {});
  }
}
// A security dialog can be taller than 844 CSS pixels. Scroll the control into view,
// then require its box to sit inside the viewport without horizontal overflow.
async function within(locator) {
  await locator.scrollIntoViewIfNeeded();
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
  await tabTo(page, browserName, 'login-otp', 10);
  const code = typeof otp === 'function' ? otp() : otp;
  await page.locator('#login-otp').fill(code);
  await tabTo(page, browserName, 'password-login', 10);
  expect((await within(page.locator('#password-login'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
}
async function expectSignedIn(page, { mfa, notice }) {
  await expect(page.locator('#catalogue')).toBeVisible();
  await expect(page.locator('#account-name')).toHaveText('Pat Passkey');
  await expect(page.locator('#auth')).toBeHidden();
  if (mfa) await expect(page.locator('#mfa-notice')).toBeHidden();
  else {
    await expect(page.locator('#mfa-notice')).toBeVisible();
    await expect(page.locator('#mfa-notice-text')).toHaveText(notice);
  }
  expect(await portalStatus(page)).toBe(200);
  await fits(page);
}
async function expectActorSignedOut(page, context) {
  await expect(page.locator('#auth')).toBeVisible();
  await expect(page.locator('#catalogue')).toBeHidden();
  await expect(page.locator('#auth-description')).toHaveText(SESSIONS_ENDED);
  expect(await portalStatus(page)).toBe(401);
  await expect.poll(async () => Boolean(await sso(context))).toBe(false);
}
async function expectOtherEnded(otherPage, other, cookie) {
  await expect.poll(() => portalStatus(otherPage)).toBe(401);
  expect((await sso(other))?.value).toBe(cookie);
}
async function openSecurity(page, browserName) {
  await tabTo(page, browserName, 'account-security', 50);
  expect((await within(page.locator('#account-security'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
  const dialog = page.locator('#security-dialog');
  await expect(dialog).toBeVisible();
  await expect(page.locator('#security-account')).toHaveText('Pat Passkey (@passkey1)');
  await within(dialog);
  await fits(page);
  return dialog;
}
async function replaceFocusedText(page, value) {
  await page.keyboard.press('ControlOrMeta+A');
  if (value) await page.keyboard.type(value);
  else await page.keyboard.press('Backspace');
}
async function authenticatorIdentity(context) {
  const rows = await context.credentials.get({ rpId: 'localhost' });
  expect(rows).toHaveLength(1);
  const [row] = rows;
  return { id: row.id, rpId: row.rpId, userHandle: row.userHandle, publicKey: row.publicKey };
}
async function storedPasskeys(page) {
  return page.evaluate(async (url) => {
    const response = await fetch(url, { credentials: 'same-origin' });
    if (!response.ok) throw new Error(`passkeys ${response.status}`);
    const body = await response.json();
    const passkeys = Array.isArray(body.passkeys) ? body.passkeys : [];
    return passkeys.map((passkey) => ({ id: String(passkey.id), name: String(passkey.name) }));
  }, `${fixture.issuer}/api/portal/passkeys`);
}
function namedPasskey(passkeys, name) {
  const found = passkeys.filter((passkey) => passkey.name === name);
  if (found.length !== 1) throw new Error(`expected one ${name}, saw ${passkeys.map((passkey) => passkey.name).join(', ') || '(none)'}`);
  return found[0];
}
async function signInWithPasskey(page, browserName) {
  const button = page.locator('#passkey-login');
  await expect(button).toBeVisible();
  if (await focusedId(page) !== 'passkey-login') await tabTo(page, browserName, 'passkey-login', 25);
  expect((await within(button)).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
  await expectSignedIn(page, { mfa: true });
}
async function enrollNamedPasskey(page, browserName, name) {
  await tabToPasskeyName(page, browserName);
  await replaceFocusedText(page, name);
  await expect(page.locator('#passkey-name')).toHaveValue(name);
  await tabToControl(page, browserName, 'add-passkey', 12);
  const add = page.locator('#add-passkey');
  await expect(add).toBeFocused();
  await expect(add).toHaveText(name === LAPTOP ? 'Add a passkey' : 'Add another passkey');
  expect((await within(add)).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
  await expect(page.locator('#toast')).toHaveText('Passkey added. Sign in with it to continue.');
  await expectActorSignedOut(page, page.context());
}
// Enrollment spends the step it accepts. The next sign-in uses a later step that is
// still inside the server's one-step window, including when the wall clock has moved on.
function laterCode(secret, confirmedStep) {
  const nowStep = step();
  return hotp(secret, nowStep > confirmedStep ? nowStep : confirmedStep + 1);
}
async function acknowledgeCodes(page, browserName) {
  const codesDialog = page.locator('#codes-dialog');
  await expect(codesDialog).toBeVisible();
  await expect(page.locator('#codes-title')).toBeFocused();
  await expect(page.locator('#codes-title')).toHaveText('Authenticator app turned on');
  const codes = (await page.locator('#codes-list li').allTextContents()).map((code) => code.trim());
  expect(codes).toHaveLength(10);
  expect(new Set(codes).size).toBe(10);
  for (const code of codes) expect(code).toMatch(/^ri_recovery_[A-Za-z0-9_-]{40,}$/);
  await within(codesDialog);
  await fits(page);
  await tabTo(page, browserName, 'codes-done', 15);
  await expect(page.locator('#codes-done')).not.toHaveAttribute('aria-disabled', 'true');
  expect((await within(page.locator('#codes-done'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
  await expect(page.locator('#codes-saved')).toBeFocused();
  await expect(page.locator('#codes-status')).toHaveText('Save the codes, then confirm that you saved them.');
  await expect(codesDialog).toBeVisible();
  await within(page.locator('label.checkbox'));
  await page.keyboard.press('Space');
  await expect(page.locator('#codes-saved')).toBeChecked();
  await tabTo(page, browserName, 'codes-done', 10);
  await expect(page.locator('#codes-done')).not.toHaveAttribute('aria-disabled', 'true');
  await page.keyboard.press('Enter');
  await expect(codesDialog).toBeHidden();
  await expect(page.locator('#codes-list')).toBeEmpty();
}

test('removing one of two passkeys leaves the other and the authenticator app', async ({ context, browser, browserName }) => {
  test.setTimeout(180000);
  test.skip(test.info().project.use.headless === false, 'This journey stays headless and does not take the shared desktop');
  const problems = [];
  await arm(context, problems);
  // The shim has to be installed before the page that calls WebAuthn exists.
  await context.credentials.install();
  const page = await context.newPage();
  const user = fixture.users.passkey;
  await page.goto(`${fixture.issuer}/apps`);
  test.skip(!(await page.evaluate(() => 'PublicKeyCredential' in window && isSecureContext)),
    'This engine build has no WebAuthn, so the pages hide their passkey buttons');

  const other = await browser.newContext({ viewport: VIEWPORT });
  try {
    await arm(other, problems);
    await other.credentials.install();
    const otherPage = await other.newPage();
    await keyboardSignIn(page, browserName, user);
    await expectSignedIn(page, { mfa: false, notice: SETUP_NOTICE });
    await otherPage.goto(`${fixture.issuer}/apps`);
    await keyboardSignIn(otherPage, browserName, user);
    await expectSignedIn(otherPage, { mfa: false, notice: SETUP_NOTICE });
    const firstOther = await sso(other);
    expect(firstOther?.value).toMatch(/^ri_sso_/);
    expect(await portalStatus(otherPage)).toBe(200);

    await openSecurity(page, browserName);
    await expect(page.locator('#security-status')).toHaveText('You have no passkeys yet.');
    await expect(page.locator('#passkey-list')).toBeEmpty();
    await expect(page.locator('#reauth-panel')).toBeHidden();
    await expect(page.locator('#totp-summary')).toHaveText(TOTP_OFF);
    await expect(page.locator('#recovery-section')).toBeHidden();
    await enrollNamedPasskey(page, browserName, LAPTOP);
    await expectOtherEnded(otherPage, other, firstOther.value);
    const laptop = await authenticatorIdentity(context);
    expect(laptop.rpId).toBe('localhost');
    expect(laptop.id).toMatch(/\S/);
    expect(laptop.userHandle).toMatch(/\S/);

    // The passkey session is fresh MFA, which the authenticator enrollment requires.
    await signInWithPasskey(page, browserName);
    expect(await authenticatorIdentity(context)).toEqual(laptop);
    await otherPage.reload();
    await expect(otherPage.locator('#auth')).toBeVisible();
    expect((await sso(other))?.value).toBe(firstOther.value);
    await keyboardSignIn(otherPage, browserName, user);
    await expectSignedIn(otherPage, { mfa: false, notice: FACTOR_NOTICE });
    const passwordSession = await sso(other);
    expect(passwordSession?.value).toMatch(/^ri_sso_/);
    expect(passwordSession.value).not.toBe(firstOther.value);

    await openSecurity(page, browserName);
    await expect(page.locator('#security-status')).toHaveText(/^1 of \d+ passkeys\./);
    await expect(page.locator('#passkey-list strong')).toHaveText(LAPTOP);
    await expect(page.locator('#reauth-panel')).toBeHidden();
    await expect(page.locator('#totp-summary')).toHaveText(TOTP_OFF);
    await expect(page.locator('#recovery-section')).toBeHidden();
    await tabToControl(page, browserName, 'totp-setup', 40);
    expect((await within(page.locator('#totp-setup'))).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    await expect(page.locator('#totp-enroll-title')).toBeFocused();
    await expect(page.locator('#totp-enroll-title')).toHaveText('Set up your authenticator app');
    await expect(page.locator('#totp-key-details')).toHaveText('Time-based, 6 digits, new code every 30 seconds (SHA1). Spaces in the key are optional.');
    const secret = (await page.locator('#totp-key').innerText()).replace(/\s+/g, '');
    expect(secret).toMatch(/^[A-Z2-7]+=*$/);
    await within(page.locator('#security-dialog'));

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
    expect((await sso(other))?.value).toBe(passwordSession.value);

    await tabBack(page, browserName, 'totp-code');
    await page.locator('#totp-code').fill(wrongCode(secret));
    await tabTo(page, browserName, 'totp-confirm', 10);
    expect((await within(page.locator('#totp-confirm'))).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    await expect(enrollError).toBeFocused();
    await expect(enrollError).toHaveText("That code didn't match. Enter the newest code from your app.");
    await expect(page.locator('#totp-code')).toHaveValue('');
    await expect(page.locator('#totp-code')).toHaveAttribute('aria-invalid', 'true');
    await within(enrollError);
    expect(await portalStatus(page)).toBe(200);
    expect((await sso(other))?.value).toBe(passwordSession.value);

    await tabBack(page, browserName, 'totp-code');
    const confirmedStep = step();
    await page.locator('#totp-code').fill(hotp(secret, confirmedStep));
    await tabTo(page, browserName, 'totp-confirm', 10);
    await page.keyboard.press('Enter');
    await expect(page.locator('#toast')).toHaveText('Authenticator app turned on. Sign in with your password and a code from it.');
    await expectActorSignedOut(page, context);
    await expectOtherEnded(otherPage, other, passwordSession.value);
    await acknowledgeCodes(page, browserName);

    await otherPage.reload();
    await expect(otherPage.locator('#auth')).toBeVisible();
    await expect(otherPage.locator('#catalogue')).toBeHidden();
    expect((await sso(other))?.value).toBe(passwordSession.value);
    await tabTo(otherPage, browserName, 'login-username', 30);
    await otherPage.locator('#login-username').fill(user.username);
    await tabTo(otherPage, browserName, 'login-password', 10);
    await otherPage.locator('#login-password').fill(user.password);
    await tabTo(otherPage, browserName, 'login-otp', 10);
    await otherPage.locator('#login-otp').fill('');
    await tabTo(otherPage, browserName, 'password-login', 10);
    expect((await within(otherPage.locator('#password-login'))).height).toBeGreaterThanOrEqual(24);
    const rejectedAt = Date.now();
    await otherPage.keyboard.press('Enter');
    const rejected = otherPage.locator('#auth-error');
    await expect(rejected).toBeFocused();
    // The failed password check waits out the one-second credential floor before this error.
    expect(Date.now() - rejectedAt).toBeGreaterThanOrEqual(900);
    await expect(rejected).toHaveText(SIGN_IN_ERROR);
    await expect(otherPage.locator('#login-username')).toHaveValue(user.username);
    for (const id of ['login-username', 'login-password', 'login-otp']) {
      await expect(otherPage.locator(`#${id}`)).toHaveAttribute('aria-invalid', 'true');
    }
    await expect(otherPage.locator('#login-password')).toHaveValue('');
    await expect(otherPage.locator('#login-otp')).toHaveValue('');
    await expect(otherPage.locator('#catalogue')).toBeHidden();
    expect(await portalStatus(otherPage)).toBe(401);
    expect((await sso(other))?.value).toBe(passwordSession.value);
    await within(rejected);
    await fits(otherPage);

    await keyboardSignIn(otherPage, browserName, user, () => laterCode(secret, confirmedStep));
    await expectSignedIn(otherPage, { mfa: true });
    const codeSession = await sso(other);
    expect(codeSession?.value).toMatch(/^ri_sso_/);
    expect(codeSession.value).not.toBe(passwordSession.value);

    // Signing in with the first passkey does not end the authenticator session.
    await signInWithPasskey(page, browserName);
    expect(await authenticatorIdentity(context)).toEqual(laptop);
    const laptopSession = await sso(context);
    expect(laptopSession?.value).toMatch(/^ri_sso_/);
    expect((await sso(other))?.value).toBe(codeSession.value);
    expect(await portalStatus(otherPage)).toBe(200);

    await openSecurity(otherPage, browserName);
    await expect(otherPage.locator('#security-status')).toHaveText(/^1 of \d+ passkeys\./);
    await expect(otherPage.locator('#passkey-list strong')).toHaveText(LAPTOP);
    await expect(otherPage.locator('#reauth-panel')).toBeHidden();
    await expect(otherPage.locator('#totp-summary')).toHaveText(TOTP_ON);
    await enrollNamedPasskey(otherPage, browserName, BACKUP);
    await expectOtherEnded(page, context, laptopSession.value);
    const backup = await authenticatorIdentity(other);
    expect(backup.rpId).toBe('localhost');
    expect(backup.id).not.toBe(laptop.id);
    expect(backup.publicKey).not.toBe(laptop.publicKey);
    expect(backup.userHandle).toBe(laptop.userHandle);

    await page.reload();
    await expect(page.locator('#auth')).toBeVisible();
    expect((await sso(context))?.value).toBe(laptopSession.value);
    await signInWithPasskey(page, browserName);
    const renewedLaptop = await sso(context);
    expect(renewedLaptop?.value).toMatch(/^ri_sso_/);
    expect(renewedLaptop.value).not.toBe(laptopSession.value);
    expect(await authenticatorIdentity(context)).toEqual(laptop);
    // Signing in with the second passkey does not end the first passkey's new session.
    await signInWithPasskey(otherPage, browserName);
    expect(await authenticatorIdentity(other)).toEqual(backup);
    const backupSession = await sso(other);
    expect(backupSession?.value).toMatch(/^ri_sso_/);
    expect((await sso(context))?.value).toBe(renewedLaptop.value);
    expect(await portalStatus(page)).toBe(200);

    await openSecurity(page, browserName);
    await expect(page.locator('#security-status')).toHaveText(/^2 of \d+ passkeys\./);
    await expect(page.locator('#reauth-panel')).toBeHidden();
    await expect(page.locator('#totp-summary')).toHaveText(TOTP_ON);
    const names = await page.locator('#passkey-list strong').allTextContents();
    expect(names).toHaveLength(2);
    expect(names).toEqual(expect.arrayContaining([LAPTOP, BACKUP]));
    await expect(page.getByRole('button', { name: `Remove ${LAPTOP}` })).toBeVisible();
    await expect(page.getByRole('button', { name: `Remove ${BACKUP}` })).toBeVisible();
    const listed = await storedPasskeys(page);
    const storedLaptop = namedPasskey(listed, LAPTOP);
    const storedBackup = namedPasskey(listed, BACKUP);
    expect(storedLaptop.id).not.toBe(storedBackup.id);
    // The portal id is a digest of the credential id, not the shim's raw id.
    expect(storedLaptop.id).not.toBe(laptop.id);
    expect(storedBackup.id).not.toBe(backup.id);

    await tabToLabel(page, browserName, `Remove ${LAPTOP}`);
    const remove = page.getByRole('button', { name: `Remove ${LAPTOP}` });
    await expect(remove).toBeFocused();
    expect((await within(remove)).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    await expect(page.locator('#passkey-action-title')).toBeFocused();
    await expect(page.locator('#passkey-action-title')).toHaveText(`Remove ${LAPTOP}?`);
    await expect(page.locator('#passkey-action-description')).toHaveText('This passkey will stop working for this account and all your sessions will end. Make sure you have another way to sign in.');
    await tabToControl(page, browserName, 'passkey-action-confirm', 10);
    const confirm = page.locator('#passkey-action-confirm');
    await expect(confirm).toBeFocused();
    await expect(confirm).toHaveText('Remove passkey');
    await expect(confirm).not.toHaveAttribute('aria-disabled', 'true');
    expect((await within(confirm)).height).toBeGreaterThanOrEqual(24);
    await fits(page);
    await page.keyboard.press('Enter');
    await expect(page.locator('#toast')).toHaveText('Passkey removed. Sign in again.');
    await expectActorSignedOut(page, context);
    await expectOtherEnded(otherPage, other, backupSession.value);
    expect(await authenticatorIdentity(context)).toEqual(laptop);

    const rejectedPasskey = page.locator('#passkey-login');
    await expect(rejectedPasskey).toBeVisible();
    if (await focusedId(page) !== 'passkey-login') await tabTo(page, browserName, 'passkey-login', 25);
    expect((await within(rejectedPasskey)).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    const unknown = page.locator('#auth-error');
    await expect(unknown).toHaveText(UNKNOWN);
    await expect(unknown).toBeFocused();
    await expect(page.locator('#auth')).toBeVisible();
    await expect(page.locator('#catalogue')).toBeHidden();
    expect(await portalStatus(page)).toBe(401);
    await within(unknown);
    await fits(page);
    expect(await authenticatorIdentity(context)).toEqual(laptop);

    await otherPage.reload();
    await expect(otherPage.locator('#auth')).toBeVisible();
    await expect(otherPage.locator('#catalogue')).toBeHidden();
    expect((await sso(other))?.value).toBe(backupSession.value);
    await signInWithPasskey(otherPage, browserName);
    expect(await authenticatorIdentity(other)).toEqual(backup);
    expect(await portalStatus(otherPage)).toBe(200);
    await openSecurity(otherPage, browserName);
    await expect(otherPage.locator('#security-status')).toHaveText(/^1 of \d+ passkeys\./);
    await expect(otherPage.locator('#passkey-list strong')).toHaveText(BACKUP);
    await expect(otherPage.getByRole('button', { name: `Remove ${LAPTOP}` })).toHaveCount(0);
    await expect(otherPage.getByRole('button', { name: `Remove ${BACKUP}` })).toBeVisible();
    await expect(otherPage.locator('#reauth-panel')).toBeHidden();
    await expect(otherPage.locator('#totp-summary')).toHaveText(TOTP_ON);
    await expect(otherPage.locator('#totp-remove')).toBeVisible();
    await expect(otherPage.locator('#recovery-summary')).toHaveText('10 of 10 recovery codes left. Each one signs you in once instead of an app code.');
    const remaining = await storedPasskeys(otherPage);
    expect(remaining).toEqual([storedBackup]);
    await within(otherPage.locator('#totp-summary'));
    await fits(otherPage);
    // Closing the other context makes Playwright snapshot this page. WebKit's snapshot
    // inserts an inline style, which the portal CSP rejects, so the assertion has to run first.
    expect(problems, 'CSP violations and page errors').toEqual([]);
  } finally {
    await other.close();
  }
});
