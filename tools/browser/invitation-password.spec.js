// Headless password invitation acceptance on a 390×844 CSS viewport.
//
// Chromium, Firefox, and WebKit each accept one invitation with a password.
// An expired invitation and a replay of the accepted link are rejected. Success
// does not sign the browser in. Signing in with the new password and then
// signing out removes that session. This journey never installs a virtual
// authenticator and never posts to the invitation passkey endpoints. A visible
// passkey choice stays unused. Chromium's CDP invitation ceremony stays in
// invitation-passkey.spec.js. Tokens come from the fixture's
// loopback SMTP capture, not an external mailbox. The viewport is CSS only.
// This is not a screen reader, a physical key, a synced passkey, a phone, or
// a mobile operating system. The integration job is written to run this file headless in
// Chromium, Firefox, and WebKit and does not install an authenticator.
import { readFile } from 'node:fs/promises';
import { test, expect } from '@playwright/test';
import { fixtureStartupMs, startFixture } from './fixture.js';

const VIEWPORT = { width: 390, height: 844 };
const PASSWORD = 'invite-password-accept-123';
const EXPIRED = 'This invitation has expired. Ask your administrator for a new invitation.';
const USED = 'This invitation has already been accepted. Continue to sign in.';
const PASSWORD_READY = 'Sign in with your new password to open your applications. An application may also require a passkey or authenticator code.';
const DESCRIPTION_PASSWORD = 'Set a password to activate your account. You will sign in after accepting the invitation.';
const DESCRIPTION_BOTH = 'Set a password, or add a passkey, to activate your account. You will sign in after accepting the invitation.';
const CSP = /content.security.policy|csp violation|refused to (load|execute|apply|connect|frame)/i;

test.use({ viewport: VIEWPORT, headless: true });

let fixture, stopFixture;
test.beforeAll(async () => {
  test.setTimeout(fixtureStartupMs + 5000);
  ({ fixture, stop: stopFixture } = await startFixture({ mailCapture: true }));
  expect(fixture.mail_capture, 'fixture mail capture path').toEqual(expect.any(String));
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
async function sessionCookies(response) {
  const headers = await response.headersArray();
  return headers.filter((header) => header.name.toLowerCase() === 'set-cookie');
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
function messageText(raw) {
  const normalized = raw.replace(/\r\n/g, '\n');
  const split = normalized.indexOf('\n\n');
  const headers = split === -1 ? '' : normalized.slice(0, split);
  const body = split === -1 ? normalized : normalized.slice(split + 2);
  if (!/content-transfer-encoding:\s*quoted-printable/i.test(headers)) return body;
  return body.replace(/=\n/g, '').replace(/=([0-9A-Fa-f]{2})/g, (_, hex) => String.fromCharCode(Number.parseInt(hex, 16)));
}
function forAccount(bodies, username) {
  return bodies.filter((body) => messageText(body).split('\n').some((line) => line.trim() === `Account: ${username}`));
}
function browserLink(body) {
  const match = messageText(body).match(/Open in your browser: (\S+)/);
  if (!match) throw new Error('invitation mail for an account has no browser link');
  return match[1];
}
async function invitationLinks() {
  await expect.poll(async () => {
    const bodies = await capturedBodies();
    return ['invite-passkey', 'invite-password', 'invite-expired'].every((username) => forAccount(bodies, username).length === 1);
  }, { timeout: 20000 }).toBe(true);
  const bodies = await capturedBodies();
  const link = (username) => {
    const found = browserLink(forAccount(bodies, username)[0]);
    expect(found.startsWith(`${fixture.issuer}/account/accept#token=ri_mail_`)).toBe(true);
    return found;
  };
  const passkey = link('invite-passkey');
  const password = link('invite-password');
  const expired = link('invite-expired');
  expect(new Set([passkey, password, expired]).size).toBe(3);
  return { password, expired };
}
async function openInvite(page, url) {
  const ready = page.waitForResponse((result) => result.url().endsWith('/api/capabilities') && result.ok(), { timeout: 15000 });
  await page.goto(url);
  await ready;
  await expect(page.locator('#account-title')).toHaveText('Accept your invitation');
  await expect(page.locator('#account-title')).toBeFocused({ timeout: 15000 });
  await expect(page.locator('#account-form')).toBeVisible();
  await expect(page.locator('#account-password')).toBeVisible();
  await expect(page.locator('#account-confirm')).toBeVisible();
  await expect(page.locator('#account-submit')).toHaveText('Accept invitation');
  await expect.poll(async () => {
    const offered = await page.locator('#account-passkey-form').isVisible();
    const description = await page.locator('#account-description').textContent();
    if (offered && description === DESCRIPTION_BOTH) return 'shown';
    if (!offered && description === DESCRIPTION_PASSWORD) return 'hidden';
    return 'pending';
  }).not.toBe('pending');
  const choice = await page.locator('#account-passkey-form').isVisible();
  expect(page.url()).not.toContain('ri_mail_');
  expect((await within(page.locator('#account-password'))).height).toBeGreaterThanOrEqual(24);
  expect((await within(page.locator('#account-confirm'))).height).toBeGreaterThanOrEqual(24);
  expect((await within(page.locator('#account-submit'))).height).toBeGreaterThanOrEqual(24);
  if (choice) {
    expect((await within(page.locator('#account-passkey-submit'))).height).toBeGreaterThanOrEqual(24);
    await within(page.locator('#account-passkey-name'));
  }
  await fits(page);
  return choice;
}
async function typePassword(page, browserName) {
  await tabTo(page, browserName, 'account-password', 12);
  await page.locator('#account-password').fill('');
  await page.keyboard.type(PASSWORD);
  await tabTo(page, browserName, 'account-confirm', 6);
  await page.locator('#account-confirm').fill('');
  await page.keyboard.type(PASSWORD);
  await tabTo(page, browserName, 'account-submit', 6);
}
async function expectSignedOut(page, context) {
  expect(await sso(context), 'accept page does not mint a session cookie').toBeUndefined();
  expect(await portalStatus(page)).toBe(401);
}
async function keyboardSignIn(page, browserName) {
  await expect(page.locator('#auth')).toBeVisible();
  await tabTo(page, browserName, 'login-username', 30);
  await page.locator('#login-username').fill('invite-password');
  await tabTo(page, browserName, 'login-password', 10);
  await page.locator('#login-password').fill(PASSWORD);
  await tabTo(page, browserName, 'login-otp', 10);
  await page.locator('#login-otp').fill('');
  await tabTo(page, browserName, 'password-login', 10);
  expect((await within(page.locator('#password-login'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
}
async function signOut(page, context, browserName) {
  await tabTo(page, browserName, 'sign-out', 25);
  const button = page.locator('#sign-out');
  expect((await within(button)).height).toBeGreaterThanOrEqual(24);
  await expect(button).not.toHaveAttribute('aria-disabled', 'true');
  await page.keyboard.press('Enter');
  await expect(page.locator('#toast')).toHaveText('You’re signed out.');
  await expect(page.locator('#auth')).toBeVisible();
  await expect.poll(async () => Boolean(await sso(context))).toBe(false);
  expect(await portalStatus(page)).toBe(401);
}

test('password invitation acceptance rejects expiry and replay without a session', async ({ page, context, browserName }) => {
  test.setTimeout(180000);
  test.skip(test.info().project.use.headless === false, 'This journey stays headless and does not take the shared desktop');
  const problems = [];
  await arm(context, problems);
  const posts = { start: 0, finish: 0, cancel: 0, password: 0 };
  page.on('request', (request) => {
    if (request.method() !== 'POST') return;
    const url = request.url();
    if (url.endsWith('/api/account/accept/passkey/start')) posts.start += 1;
    else if (url.endsWith('/api/account/accept/passkey/finish')) posts.finish += 1;
    else if (url.endsWith('/api/account/accept/passkey/cancel')) posts.cancel += 1;
    else if (url.endsWith('/api/portal/account/accept')) posts.password += 1;
  });
  const links = await invitationLinks();

  await openInvite(page, links.expired);
  await typePassword(page, browserName);
  const expiredWait = page.waitForResponse((result) => result.url().endsWith('/api/portal/account/accept') && result.request().method() === 'POST');
  await page.keyboard.press('Enter');
  const expiredResponse = await expiredWait;
  expect(expiredResponse.status()).toBe(410);
  expect(await expiredResponse.json()).toMatchObject({ error: 'account_code_expired' });
  expect(await sessionCookies(expiredResponse)).toEqual([]);
  await expect(page.locator('#account-error')).toBeFocused();
  await expect(page.locator('#account-error')).toHaveText(EXPIRED);
  await expect(page.locator('#account-form')).toBeHidden();
  await expect(page.locator('#account-passkey-form')).toBeHidden();
  await expect(page.locator('#account-fallback')).toBeVisible();
  await within(page.locator('#account-error'));
  await fits(page);
  await expectSignedOut(page, context);
  expect(posts).toEqual({ start: 0, finish: 0, cancel: 0, password: 1 });

  const passkeyChoice = await openInvite(page, links.password);
  await typePassword(page, browserName);
  const accepted = page.waitForResponse((result) => result.url().endsWith('/api/portal/account/accept') && result.request().method() === 'POST');
  await page.keyboard.press('Enter');
  const passwordResponse = await accepted;
  expect(passwordResponse.status()).toBe(200);
  expect(await sessionCookies(passwordResponse)).toEqual([]);
  expect(await passwordResponse.json()).toEqual({ completed: true, login_required: true });
  const passwordPosted = passwordResponse.request().postDataJSON();
  expect(Object.keys(passwordPosted).sort()).toEqual(['password', 'token']);
  expect(passwordPosted.password).toBe(PASSWORD);
  expect(passwordPosted.token).toMatch(/^ri_mail_[A-Za-z0-9_-]+$/);
  await expect(page.locator('#account-complete')).toBeFocused();
  await expect(page.locator('#account-title')).toHaveText('Invitation accepted');
  await expect(page.locator('#account-complete-title')).toHaveText('Your account is ready');
  await expect(page.locator('#account-complete-text')).toHaveText(PASSWORD_READY);
  await expect(page.locator('#account-form')).toBeHidden();
  await expect(page.locator('#account-passkey-form')).toBeHidden();
  await expect(page.locator('#account-fallback')).toBeHidden();
  await within(page.locator('#account-complete-title'));
  await fits(page);
  await expectSignedOut(page, context);
  expect(posts).toEqual({ start: 0, finish: 0, cancel: 0, password: 2 });

  await openInvite(page, links.password);
  await typePassword(page, browserName);
  const replay = page.waitForResponse((result) => result.url().endsWith('/api/portal/account/accept') && result.request().method() === 'POST');
  await page.keyboard.press('Enter');
  const replayResponse = await replay;
  expect(replayResponse.status()).toBe(410);
  expect(await replayResponse.json()).toMatchObject({ error: 'account_code_used' });
  expect(await sessionCookies(replayResponse)).toEqual([]);
  await expect(page.locator('#account-error')).toBeFocused();
  await expect(page.locator('#account-error')).toHaveText(USED);
  await expect(page.locator('#account-form')).toBeHidden();
  await expect(page.locator('#account-passkey-form')).toBeHidden();
  await expect(page.locator('#account-fallback')).toBeVisible();
  await within(page.locator('#account-error'));
  await fits(page);
  await expectSignedOut(page, context);
  expect(posts).toEqual({ start: 0, finish: 0, cancel: 0, password: 3 });

  await page.goto(`${fixture.issuer}/apps`);
  await keyboardSignIn(page, browserName);
  await expect(page.locator('#catalogue')).toBeVisible({ timeout: 20000 });
  await expect(page.locator('#account-name')).toHaveText('Ida Password');
  expect((await sso(context))?.value).toMatch(/^ri_sso_/);
  expect(await portalStatus(page)).toBe(200);
  await fits(page);
  await signOut(page, context, browserName);
  await fits(page);
  expect(posts).toEqual({ start: 0, finish: 0, cancel: 0, password: 3 });
  test.info().annotations.push({
    type: 'passkey-choice',
    description: passkeyChoice
      ? 'shown on the password invitation and left unused; no passkey request was sent'
      : 'not shown; no passkey request was sent',
  });
  expect(problems, 'console CSP or page errors').toEqual([]);
});
