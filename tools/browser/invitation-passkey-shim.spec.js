// Headless invitation passkey acceptance with Playwright's simulated credential.
//
// The shim replaces navigator.credentials before the page loads. It generates a
// P-256 key in the test process, sets the user-present and user-verified bits
// itself, and keeps the credential discoverable when creation asks for
// residentKey "required" or "preferred". It does not prompt, it cannot refuse
// verification, and it is not the browser's authenticator. webauthn-rs still
// requires those bits for passkey registration and checks the challenge,
// origin, and relying party; attestation format none is accepted. The server
// does not prove discoverability. Signing in afterwards sends no allow list,
// and this shim answers that only for a credential it stored as discoverable.
// Its credential list is not evidence of the user-verified bit, so this file
// does not read one. Chromium's CDP virtual authenticator stays in
// invitation-passkey.spec.js and is not used here. The shim is not a physical
// key, a synced passkey, a phone, or a mobile operating system. Tokens come
// from the fixture's loopback SMTP capture, not an external mailbox. The
// viewport is 390×844 CSS pixels. This is not a screen reader, and Playwright
// is not a CI job.
import { readFile } from 'node:fs/promises';
import { test, expect } from '@playwright/test';
import { fixtureStartupMs, startFixture } from './fixture.js';

const VIEWPORT = { width: 390, height: 844 };
const EXPIRED = 'This invitation has expired. Ask your administrator for a new invitation.';
const USED = 'This invitation has already been accepted. Continue to sign in.';
const BLANK = 'Passkey name must not be blank.';
const PASSKEY_READY = 'Sign in with your new passkey to open your applications. This page did not sign you in.';
const DESCRIPTION = 'Set a password, or add a passkey, to activate your account. You will sign in after accepting the invitation.';
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
async function simulated(context) {
  const stored = await context.credentials.get({ rpId: 'localhost' });
  return stored.map(({ id, rpId, userHandle }) => ({ id, rpId, userHandle }));
}
function clientData(value) {
  return JSON.parse(Buffer.from(value, 'base64url').toString('utf8'));
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
  return { passkey, expired };
}
async function openInvite(page) {
  await expect(page.locator('#account-title')).toHaveText('Accept your invitation');
  await expect(page.locator('#account-title')).toBeFocused({ timeout: 15000 });
  await expect(page.locator('#account-form')).toBeVisible();
  await expect(page.locator('#account-passkey-form')).toBeVisible();
  await expect(page.locator('#account-description')).toHaveText(DESCRIPTION);
  await expect(page.getByRole('button', { name: 'Accept with a passkey' })).toBeVisible();
  expect(page.url()).not.toContain('ri_mail_');
  expect((await within(page.locator('#account-passkey-submit'))).height).toBeGreaterThanOrEqual(24);
  await within(page.locator('#account-passkey-name'));
  await fits(page);
}
async function expectSignedOut(page, context) {
  expect(await sso(context), 'accept page does not mint a session cookie').toBeUndefined();
  expect(await portalStatus(page)).toBe(401);
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

test('simulated credential accepts an invitation passkey without a session', async ({ context, browserName }) => {
  test.setTimeout(180000);
  test.skip(test.info().project.use.headless === false, 'This journey stays headless and does not take the shared desktop');
  const problems = [];
  await arm(context, problems);
  // Installed before the page exists. This is not a CDP virtual authenticator.
  await context.credentials.install();
  const page = await context.newPage();
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

  await page.goto(links.expired);
  test.skip(!(await page.evaluate(() => 'PublicKeyCredential' in window && isSecureContext)),
    'This engine build has no WebAuthn, so the invitation page hides its passkey choice');
  await openInvite(page);
  await tabTo(page, browserName, 'account-passkey-name', 12);
  await page.locator('#account-passkey-name').fill('Expired key');
  await tabTo(page, browserName, 'account-passkey-submit', 6);
  const expiredStart = page.waitForResponse((result) => result.url().endsWith('/api/account/accept/passkey/start') && result.request().method() === 'POST');
  await page.keyboard.press('Enter');
  const expiredResponse = await expiredStart;
  expect(expiredResponse.status()).toBe(410);
  expect(await expiredResponse.json()).toMatchObject({ error: 'account_code_expired' });
  await expect(page.locator('#account-error')).toBeFocused();
  await expect(page.locator('#account-error')).toHaveText(EXPIRED);
  await expect(page.locator('#account-form')).toBeHidden();
  await expect(page.locator('#account-passkey-form')).toBeHidden();
  await expect(page.locator('#account-fallback')).toBeVisible();
  await within(page.locator('#account-error'));
  expect(posts).toEqual({ start: 1, finish: 0, cancel: 0, password: 0 });
  expect(await simulated(context)).toEqual([]);
  await expectSignedOut(page, context);

  await page.goto(links.passkey);
  await openInvite(page);
  await tabTo(page, browserName, 'account-passkey-name', 12);
  await page.locator('#account-passkey-name').fill('   ');
  await tabTo(page, browserName, 'account-passkey-submit', 6);
  await page.keyboard.press('Enter');
  await expect(page.locator('#account-error')).toBeFocused();
  await expect(page.locator('#account-error')).toHaveText(BLANK);
  await expect(page.locator('#account-passkey-name')).toHaveAttribute('aria-invalid', 'true');
  await expect(page.locator('#account-form')).toBeVisible();
  await expect(page.locator('#account-passkey-form')).toBeVisible();
  expect(posts.start).toBe(1);
  expect(await simulated(context)).toEqual([]);

  await page.locator('#account-passkey-name').fill('Invitation key');
  await tabTo(page, browserName, 'account-passkey-submit', 6);
  const started = page.waitForResponse((result) => result.url().endsWith('/api/account/accept/passkey/start') && result.request().method() === 'POST');
  const finished = page.waitForResponse((result) => result.url().endsWith('/api/account/accept/passkey/finish') && result.request().method() === 'POST');
  await page.keyboard.press('Enter');
  const start = await started;
  expect(start.status()).toBe(200);
  expect(new URL(start.url()).origin).toBe(new URL(fixture.issuer).origin);
  const challenge = await start.json();
  expect(challenge.public_key.publicKey.rp.id).toBe('localhost');
  expect(challenge.public_key.publicKey.authenticatorSelection.userVerification).toBe('required');
  expect(challenge.public_key.publicKey.authenticatorSelection.residentKey).toBe('required');
  expect(challenge.public_key.publicKey.authenticatorSelection.requireResidentKey).toBe(true);
  const posted = start.request().postDataJSON();
  expect(Object.keys(posted).sort()).toEqual(['name', 'token']);
  expect(posted.name).toBe('Invitation key');
  expect(posted.token).toMatch(/^ri_mail_/);
  const finish = await finished;
  expect(finish.status()).toBe(200);
  expect(await sessionCookies(finish)).toEqual([]);
  expect(await finish.json()).toEqual({ completed: true, login_required: true });
  const finishedBody = finish.request().postDataJSON();
  expect(Object.keys(finishedBody).sort()).toEqual(['ceremony', 'response', 'token']);
  expect(finishedBody.ceremony).toMatch(/^ri_invite_enroll_/);
  expect(finishedBody.token).toMatch(/^ri_mail_/);
  expect(Object.keys(finishedBody.response).sort()).toEqual(['clientExtensionResults', 'id', 'rawId', 'response', 'type']);
  expect(finishedBody.response.type).toBe('public-key');
  const created = clientData(finishedBody.response.response.clientDataJSON);
  expect(created.type).toBe('webauthn.create');
  expect(created.origin).toBe(new URL(fixture.issuer).origin);
  expect(created.crossOrigin).toBe(false);
  expect(created.challenge).toBe(challenge.public_key.publicKey.challenge);
  await expect(page.locator('#account-complete')).toBeFocused();
  await expect(page.locator('#account-title')).toHaveText('Invitation accepted');
  await expect(page.locator('#account-complete-title')).toHaveText('Your account is ready');
  await expect(page.locator('#account-complete-text')).toHaveText(PASSKEY_READY);
  await expect(page.locator('#account-form')).toBeHidden();
  await expect(page.locator('#account-passkey-form')).toBeHidden();
  await within(page.locator('#account-complete-title'));
  await fits(page);
  await expectSignedOut(page, context);
  const enrolled = await simulated(context);
  expect(enrolled).toHaveLength(1);
  expect(enrolled[0].rpId).toBe('localhost');
  expect(enrolled[0].id).toMatch(/^[A-Za-z0-9_-]+$/);
  expect(enrolled[0].userHandle).toMatch(/^[A-Za-z0-9_-]+$/);
  expect(posts).toEqual({ start: 2, finish: 1, cancel: 0, password: 0 });

  await page.goto(links.passkey);
  await openInvite(page);
  await page.locator('#account-passkey-name').fill('Another key');
  const replay = page.waitForResponse((result) => result.url().endsWith('/api/account/accept/passkey/start') && result.request().method() === 'POST');
  await page.locator('#account-passkey-submit').click();
  const replayResponse = await replay;
  expect(replayResponse.status()).toBe(410);
  expect(await replayResponse.json()).toMatchObject({ error: 'account_code_used' });
  await expect(page.locator('#account-error')).toBeFocused();
  await expect(page.locator('#account-error')).toHaveText(USED);
  await expect(page.locator('#account-form')).toBeHidden();
  await expect(page.locator('#account-passkey-form')).toBeHidden();
  await expect(page.locator('#account-fallback')).toBeVisible();
  expect(posts).toEqual({ start: 3, finish: 1, cancel: 0, password: 0 });
  expect(await simulated(context)).toEqual(enrolled);
  await expectSignedOut(page, context);

  await page.goto(`${fixture.issuer}/apps`);
  await expect(page.locator('#passkey-login')).toBeVisible({ timeout: 15000 });
  await tabTo(page, browserName, 'passkey-login', 30);
  expect((await within(page.locator('#passkey-login'))).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
  await expect(page.locator('#catalogue')).toBeVisible({ timeout: 20000 });
  await expect(page.locator('#account-name')).toHaveText('Ivy Invite');
  expect(await portalStatus(page)).toBe(200);
  await fits(page);
  await signOut(page, context, browserName);
  expect(await simulated(context)).toEqual(enrolled);
  expect(posts).toEqual({ start: 3, finish: 1, cancel: 0, password: 0 });
  test.info().annotations.push({
    type: 'authenticator',
    description: 'Playwright simulated credential; not CDP, not the browser authenticator, and not hardware',
  });
  expect(problems, 'console CSP or page errors').toEqual([]);
});
