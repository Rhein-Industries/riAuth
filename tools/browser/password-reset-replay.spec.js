// Headless password-reset denial and replay on a 390×844 CSS viewport.
//
// Already covered elsewhere, and not repeated here:
// - password, TOTP, consent, logout, and passkey sign-in (signin.spec.js)
// - passkey removal, which ends every session (passkey-revocation.spec.js)
// - authenticator-app enrollment and recovery-code sign-in (authenticator-recovery.spec.js)
// - passkey rename (passkey-rename.spec.js)
// - the reset page when this fixture has no mail (accessibility-journeys.spec.js)
//
// This journey opts the fixture into its loopback SMTP capture. An empty username
// never requests a link. An unknown username and the unverified bob account get the
// same Check your email screen, and the capture file has no reset message for them.
// The verified recovery account gets that same screen. The one captured message is
// the fixture's local sink, not an external mailbox. Opening its browser link does
// not spend the proof. Mismatched passwords and the current password leave the link
// unused. A new password signs every session out, rejects the old password, and does
// not sign the reset page in. Submitting that same link again reports that it was
// already used, and the replacement password still signs in. The recovery account
// has no passkey or authenticator app, so the kept-factor sentence is the page copy.
// The viewport is CSS only. Hardware authenticators, real mobile devices, and
// external email stay open.
import { readFile } from 'node:fs/promises';
import { test, expect } from '@playwright/test';
import { fixtureStartupMs, startFixture } from './fixture.js';

const VIEWPORT = { width: 390, height: 844 };
const REPLACEMENT = 'recovery-reset-password-456';
const REPLAY = 'recovery-replay-password-789';
const MISMATCH = 'recovery-mismatch-password-1';
const FACTORS_KEPT = 'Resetting your password never removes your passkeys or authenticator app. If you lost one of them, contact your administrator.';
const USED = "This reset link was already used. If that wasn't you, request a new link and tell your administrator.";
const CSP = /content.security.policy|csp violation|refused to (load|execute|apply|connect|frame)/i;

test.use({ viewport: VIEWPORT, headless: true });

let fixture, stopFixture;
test.beforeAll(async () => {
  test.setTimeout(fixtureStartupMs + 5000);
  ({ fixture, stop: stopFixture } = await startFixture({ mailCapture: true }));
  expect(fixture.mail_capture, 'fixture mail capture path').toEqual(expect.any(String));
  expect(fixture.users.recovery?.username).toBe('recovery');
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
async function releaseHiddenFocus(page) {
  await page.evaluate(() => {
    const active = document.activeElement;
    const rendered = active && active !== document.body && active !== document.documentElement && active.getClientRects().length > 0;
    if (rendered) return;
    document.querySelector('.skip-link')?.focus();
  });
}
async function tabTo(page, browserName, id, limit = 40) {
  await releaseHiddenFocus(page);
  const key = tabKey(browserName, false);
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
async function within(locator) {
  const box = await locator.boundingBox();
  expect(box, 'control is inside the 390×844 viewport').not.toBeNull();
  expect(box.x).toBeGreaterThanOrEqual(0);
  expect(box.y).toBeGreaterThanOrEqual(0);
  expect(box.x + box.width).toBeLessThanOrEqual(VIEWPORT.width + 1);
  expect(box.y + box.height).toBeLessThanOrEqual(VIEWPORT.height + 1);
  return box;
}
function countPosts(page, posts) {
  page.on('request', (request) => {
    if (request.method() !== 'POST') return;
    const url = request.url();
    if (url.endsWith('/api/portal/account/reset-request')) posts.request += 1;
    else if (url.endsWith('/api/portal/account/reset')) posts.reset += 1;
  });
}
async function capturedBodies() {
  const text = await readFile(fixture.mail_capture, 'utf8');
  const end = text.endsWith('\n') ? text.length : text.lastIndexOf('\n') + 1;
  const complete = end > 0 ? text.slice(0, end).trim() : '';
  if (!complete) return [];
  return complete.split('\n').map((line) => {
    const record = JSON.parse(line);
    if (typeof record.body !== 'string') throw new Error('mail capture record has no body');
    return record.body;
  });
}
// The fixture stores the SMTP DATA bytes. Lettre sends this message as
// quoted-printable, so a long reset link is wrapped with a soft line break.
function messageText(raw) {
  const normalized = raw.replace(/\r\n/g, '\n');
  const split = normalized.indexOf('\n\n');
  const headers = split === -1 ? '' : normalized.slice(0, split);
  const body = split === -1 ? normalized : normalized.slice(split + 2);
  if (!/content-transfer-encoding:\s*quoted-printable/i.test(headers)) return body;
  return body.replace(/=\n/g, '').replace(/=([0-9A-Fa-f]{2})/g, (_, hex) => String.fromCharCode(Number.parseInt(hex, 16)));
}
function forAccount(bodies, username) {
  return bodies.filter((body) => messageText(body).split('\n').includes(`Account: ${username}`));
}
function resetLink(body) {
  const match = messageText(body).match(/Open in your browser: (\S+)/);
  return match ? match[1] : null;
}
async function keyboardSignIn(page, browserName, user) {
  await expect(page.locator('#auth')).toBeVisible();
  await tabTo(page, browserName, 'login-username', 30);
  await page.locator('#login-username').fill(user.username);
  await tabTo(page, browserName, 'login-password', 10);
  await page.locator('#login-password').fill(user.password);
  await tabTo(page, browserName, 'login-otp', 10);
  await page.locator('#login-otp').fill('');
  await tabTo(page, browserName, 'password-login', 10);
  expect((await within(page.locator('#password-login'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
}
async function expectSignedIn(page) {
  await expect(page.locator('#catalogue')).toBeVisible({ timeout: 20000 });
  await expect(page.locator('#account-name')).toHaveText('Rae Recovery');
}
async function openReset(page) {
  await page.goto(`${fixture.issuer}/account/reset`);
  await expect(page.locator('#account-form')).toBeVisible();
  await expect(page.locator('#account-username')).toBeFocused({ timeout: 15000 });
  await expect(page.locator('#account-note')).toHaveText(FACTORS_KEPT);
  await fits(page);
}
async function requestReset(page, browserName, username) {
  await tabTo(page, browserName, 'account-username', 12);
  await page.locator('#account-username').fill('');
  await page.keyboard.type(username);
  await tabTo(page, browserName, 'account-submit', 8);
  expect((await within(page.locator('#account-submit'))).height).toBeGreaterThanOrEqual(24);
  const response = page.waitForResponse((result) => result.url().endsWith('/api/portal/account/reset-request') && result.request().method() === 'POST');
  await page.keyboard.press('Enter');
  expect((await response).ok()).toBe(true);
  await expect(page.locator('#account-title')).toHaveText('Check your email');
  await expect(page.locator('#account-complete-title')).toHaveText('Reset link requested');
  await expect(page.locator('#account-complete-text')).toHaveText(`If ${username} has a verified email address and a password here, we emailed a one-use reset link. It expires in 30 minutes. Accounts that sign in through your organization's directory or only with passkeys have no password to reset here.`);
  await expect(page.locator('#account-form')).toBeHidden();
  await within(page.locator('#account-complete-title'));
  await fits(page);
}
async function typePasswords(page, browserName, password, confirm) {
  await tabTo(page, browserName, 'account-password', 15);
  await within(page.locator('#account-password'));
  await page.locator('#account-password').fill('');
  await page.keyboard.type(password);
  await tabTo(page, browserName, 'account-confirm', 8);
  await within(page.locator('#account-confirm'));
  await page.locator('#account-confirm').fill('');
  await page.keyboard.type(confirm);
  await tabTo(page, browserName, 'account-submit', 8);
  expect((await within(page.locator('#account-submit'))).height).toBeGreaterThanOrEqual(24);
}

test('password reset denies ineligible accounts and rejects a replayed link', async ({ context, browser, browserName }) => {
  test.setTimeout(180000);
  test.skip(test.info().project.use.headless === false, 'This journey stays headless and does not take the shared desktop');
  const problems = [];
  await arm(context, problems);
  const page = await context.newPage();
  const posts = { request: 0, reset: 0 };
  countPosts(page, posts);
  const user = fixture.users.recovery;

  await page.goto(`${fixture.issuer}/apps`);
  await expect(page.locator('#auth')).toBeVisible();
  await expect(page.locator('#forgot-password')).toBeVisible();
  await tabTo(page, browserName, 'forgot-password', 45);
  expect((await within(page.locator('#forgot-password'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/account\/reset$/);
  await expect(page.locator('#account-form')).toBeVisible();
  await expect(page.locator('#account-username')).toBeFocused({ timeout: 15000 });
  await expect(page.locator('#account-note')).toHaveText(FACTORS_KEPT);
  await within(page.locator('#account-note'));
  await fits(page);

  await page.keyboard.press('Enter');
  const empty = page.locator('#account-error');
  await expect(empty).toBeFocused();
  await expect(empty).toHaveText('Enter the username you sign in with.');
  await expect(page.locator('#account-username')).toHaveAttribute('aria-invalid', 'true');
  await within(empty);
  expect(posts.request).toBe(0);
  await fits(page);

  await requestReset(page, browserName, 'missing-user');
  expect(posts.request).toBe(1);
  await openReset(page);
  await requestReset(page, browserName, fixture.users.bob.username);
  expect(posts.request).toBe(2);

  const other = await browser.newContext({ viewport: VIEWPORT });
  try {
    await arm(other, problems);
    const otherPage = await other.newPage();
    await otherPage.goto(`${fixture.issuer}/apps`);
    await keyboardSignIn(otherPage, browserName, user);
    await expectSignedIn(otherPage);
    const otherSession = await sso(other);
    expect(otherSession?.value).toMatch(/^ri_sso_/);
    expect(await portalStatus(otherPage)).toBe(200);

    await openReset(page);
    await requestReset(page, browserName, user.username);
    expect(posts.request).toBe(3);
    await expect.poll(async () => forAccount(await capturedBodies(), user.username).length, { timeout: 20000 }).toBe(1);
    const bodies = await capturedBodies();
    expect(bodies).toHaveLength(1);
    expect(forAccount(bodies, 'missing-user')).toHaveLength(0);
    expect(forAccount(bodies, fixture.users.bob.username)).toHaveLength(0);
    expect(messageText(bodies[0]).split('\n')).toContain('Reset your riAuth password');
    const link = resetLink(bodies[0]);
    const prefix = `${fixture.issuer}/account/reset#token=ri_mail_`;
    expect(typeof link, 'loopback capture included a reset link').toBe('string');
    expect(link.startsWith(prefix)).toBe(true);
    expect(link.slice(prefix.length)).toMatch(/^[A-Za-z0-9_-]+$/);

    await page.goto(link);
    await expect(page.locator('#account-title')).toHaveText('Choose a new password');
    await expect(page.locator('#account-form')).toBeVisible();
    await expect(page.locator('#account-error')).toBeHidden();
    await expect(page.locator('#account-note')).toHaveText(FACTORS_KEPT);
    await expect(page.locator('#account-description')).toHaveText('Set a new password for your account. This link works once, and saving signs you out everywhere.');
    await fits(page);

    await typePasswords(page, browserName, REPLACEMENT, MISMATCH);
    const denied = posts.reset;
    await page.keyboard.press('Enter');
    const mismatch = page.locator('#account-error');
    await expect(mismatch).toBeFocused();
    await expect(mismatch).toHaveText('Your passwords do not match.');
    await expect(page.locator('#account-confirm')).toHaveAttribute('aria-invalid', 'true');
    await expect(page.locator('#account-form')).toBeVisible();
    await expect(page.locator('#account-complete')).toBeHidden();
    await within(mismatch);
    expect(posts.reset).toBe(denied);

    await typePasswords(page, browserName, user.password, user.password);
    const reusedResponse = page.waitForResponse((result) => result.url().endsWith('/api/portal/account/reset') && result.request().method() === 'POST');
    await page.keyboard.press('Enter');
    expect((await reusedResponse).status()).toBe(400);
    const reused = page.locator('#account-error');
    await expect(reused).toBeFocused({ timeout: 20000 });
    await expect(reused).toHaveText('Password was used recently');
    await expect(page.locator('#account-form')).toBeVisible();
    await expect(page.locator('#account-complete')).toBeHidden();
    expect(posts.reset).toBe(denied + 1);

    await typePasswords(page, browserName, REPLACEMENT, REPLACEMENT);
    const completed = page.waitForResponse((result) => result.url().endsWith('/api/portal/account/reset') && result.request().method() === 'POST');
    await page.keyboard.press('Enter');
    expect((await completed).ok()).toBe(true);
    await expect(page.locator('#account-title')).toHaveText('Password reset', { timeout: 20000 });
    await expect(page.locator('#account-complete-title')).toHaveText('Sign in with your new password');
    await expect(page.locator('#account-complete-text')).toHaveText('Every session on your account was signed out. Your passkeys and authenticator app are unchanged; if you use an authenticator app, signing in still asks for its code.');
    await expect(page.locator('#account-form')).toBeHidden();
    await within(page.locator('#account-complete-title'));
    expect(posts.reset).toBe(denied + 2);
    // Visiting Apps plants a placeholder SSO cookie. Reset must not turn it into a session.
    expect(await portalStatus(page)).toBe(401);
    await expect.poll(() => portalStatus(otherPage)).toBe(401);
    expect((await sso(other))?.value).toBe(otherSession.value);

    await page.goto(link);
    await expect(page.locator('#account-form')).toBeVisible();
    await expect(page.locator('#account-error')).toBeHidden();
    await typePasswords(page, browserName, REPLAY, REPLAY);
    const replayed = page.waitForResponse((result) => result.url().endsWith('/api/portal/account/reset') && result.request().method() === 'POST');
    await page.keyboard.press('Enter');
    expect((await replayed).status()).toBe(410);
    const used = page.locator('#account-error');
    await expect(used).toBeFocused();
    await expect(used).toHaveText(USED);
    await expect(page.locator('#account-form')).toBeHidden();
    await expect(page.locator('#account-fallback')).toBeVisible();
    await within(used);
    expect(posts.reset).toBe(denied + 3);

    await otherPage.goto(`${fixture.issuer}/apps`);
    await expect(otherPage.locator('#auth')).toBeVisible();
    await keyboardSignIn(otherPage, browserName, user);
    const rejected = otherPage.locator('#auth-error');
    await expect(rejected).toBeFocused({ timeout: 20000 });
    await expect(rejected).toHaveText('Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes.');
    await expect(otherPage.locator('#catalogue')).toBeHidden();
    await within(rejected);
    await keyboardSignIn(otherPage, browserName, { ...user, password: REPLACEMENT });
    await expectSignedIn(otherPage);
    expect(await portalStatus(otherPage)).toBe(200);
    await fits(otherPage);
    // Closing the other context makes Playwright snapshot that page. WebKit's snapshot
    // inserts an inline style, and this page's style-src rejects it. Check first.
    expect(problems, 'CSP violations and page errors').toEqual([]);
  } finally {
    await other.close();
  }
});
