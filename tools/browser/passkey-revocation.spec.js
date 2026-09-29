// Headless passkey revocation on a 390×844 CSS viewport.
//
// Already covered elsewhere, and not repeated here:
// - password, TOTP, consent, logout, terminal approval, and phone reflow (signin.spec.js)
// - passkey enrollment and passwordless sign-in through this same WebAuthn shim (signin.spec.js)
// - Chromium's CDP virtual authenticator, including a failed user verification (signin.spec.js)
// - keyboard cancellation of that Chromium authenticator (accessibility-journeys.spec.js)
// - email password reset, which this fixture cannot deliver (accessibility-journeys.spec.js)
//
// This journey enrolls one passkey, proves it can sign in, then removes it. Enrollment and
// removal each end the account's other browser session. The removed credential is then
// rejected, and the password still signs in. Playwright's WebAuthn shim supplies the
// credential in Chromium, Firefox, and WebKit. That shim is not a physical security key,
// a synced platform passkey, a phone, or iOS or Android. The viewport is CSS only.
// An engine whose page has no WebAuthn is skipped. Hardware and real mobile devices stay
// manual gates.
import { test, expect } from '@playwright/test';
import { fixtureStartupMs, startFixture } from './fixture.js';

const VIEWPORT = { width: 390, height: 844 };
const PASSKEY_NAME = 'Viewport passkey';
const UNKNOWN = "This passkey isn't registered with riAuth. Use another passkey or sign in with your password.";
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
async function passwordSignIn(page, user) {
  await page.goto(`${fixture.issuer}/apps`);
  await expect(page.locator('#auth')).toBeVisible();
  await fits(page);
  await page.locator('#login-username').fill(user.username);
  await page.locator('#login-password').fill(user.password);
  await page.locator('#password-login').click();
  await expect(page.locator('#catalogue')).toBeVisible();
  await expect(page.locator('#account-name')).toHaveText('Pat Passkey');
}

test('removing a passkey ends every session and rejects that credential', async ({ context, browser }) => {
  test.setTimeout(120000);
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
    const otherPage = await other.newPage();
    await passwordSignIn(page, user);
    await passwordSignIn(otherPage, user);
    const otherSession = await sso(other);
    expect(otherSession?.value).toMatch(/^ri_sso_/);
    expect(await portalStatus(otherPage)).toBe(200);

    await page.locator('#account-security').click();
    const dialog = page.getByRole('dialog', { name: 'Sign-in and security' });
    await expect(dialog).toBeVisible();
    await expect(page.locator('#security-status')).toHaveText('You have no passkeys yet.');
    await fits(page);
    await page.locator('#passkey-name').fill(PASSKEY_NAME);
    await page.getByRole('button', { name: 'Add a passkey' }).click();
    await expect(page.locator('#toast')).toHaveText('Passkey added. Sign in with it to continue.');
    await expect(page.locator('#auth')).toBeVisible();
    await expect(page.locator('#catalogue')).toBeHidden();
    expect(await portalStatus(page)).toBe(401);
    expect(await sso(context)).toBeUndefined();
    // The other browser keeps its cookie. The account epoch makes that session useless.
    await expect.poll(() => portalStatus(otherPage)).toBe(401);
    expect((await sso(other))?.value).toBe(otherSession.value);
    expect(await context.credentials.get({ rpId: 'localhost' })).toHaveLength(1);

    await page.getByRole('button', { name: 'Sign in with a passkey' }).click();
    await expect(page.locator('#catalogue')).toBeVisible();
    await expect(page.locator('#account-name')).toHaveText('Pat Passkey');
    await expect(page.locator('#mfa-notice')).toBeHidden();
    expect(await portalStatus(page)).toBe(200);
    await passwordSignIn(otherPage, user);
    const renewed = await sso(other);
    expect(renewed?.value).toMatch(/^ri_sso_/);
    expect(renewed.value).not.toBe(otherSession.value);

    await page.locator('#account-security').click();
    await expect(dialog).toBeVisible();
    await expect(page.locator('#security-status')).toHaveText(/^1 of \d+ passkeys\./);
    await page.getByRole('button', { name: `Remove ${PASSKEY_NAME}` }).click();
    await expect(page.locator('#passkey-action-title')).toHaveText(`Remove ${PASSKEY_NAME}?`);
    await expect(page.locator('#passkey-action-description')).toHaveText('This passkey will stop working for this account and all your sessions will end. Make sure you have another way to sign in.');
    const confirm = page.locator('#passkey-action-confirm');
    await expect(confirm).toHaveText('Remove passkey');
    await confirm.scrollIntoViewIfNeeded();
    const box = await confirm.boundingBox();
    expect(box, 'remove confirmation is visible in the mobile viewport').not.toBeNull();
    expect(box.height).toBeGreaterThanOrEqual(24);
    expect(box.x).toBeGreaterThanOrEqual(0);
    expect(box.y).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(VIEWPORT.width + 1);
    expect(box.y + box.height).toBeLessThanOrEqual(VIEWPORT.height + 1);
    await fits(page);
    await confirm.click();

    await expect(page.locator('#toast')).toHaveText('Passkey removed. Sign in again.');
    await expect(page.locator('#auth')).toBeVisible();
    await expect(page.locator('#catalogue')).toBeHidden();
    expect(await portalStatus(page)).toBe(401);
    expect(await sso(context)).toBeUndefined();
    await expect.poll(() => portalStatus(otherPage)).toBe(401);
    expect((await sso(other))?.value).toBe(renewed.value);
    await otherPage.reload();
    await expect(otherPage.locator('#auth')).toBeVisible();
    await expect(otherPage.locator('#catalogue')).toBeHidden();

    // The shim still holds the credential. riAuth no longer does, so sign-in is refused.
    expect(await context.credentials.get({ rpId: 'localhost' })).toHaveLength(1);
    await page.getByRole('button', { name: 'Sign in with a passkey' }).click();
    const rejected = page.locator('#auth-error');
    await expect(rejected).toHaveText(UNKNOWN);
    await expect(rejected).toBeFocused();
    await expect(page.locator('#auth')).toBeVisible();
    await expect(page.locator('#catalogue')).toBeHidden();
    expect(await portalStatus(page)).toBe(401);
    const visible = await rejected.evaluate((el) => {
      const rect = el.getBoundingClientRect();
      return rect.width > 0 && rect.height > 0 && rect.top >= 0 && rect.left >= 0
        && rect.bottom <= window.innerHeight + 1 && rect.right <= window.innerWidth + 1;
    });
    expect(visible).toBe(true);
    await fits(page);

    await passwordSignIn(page, user);
    expect(await portalStatus(page)).toBe(200);
    await page.locator('#account-security').click();
    await expect(dialog).toBeVisible();
    await expect(page.locator('#security-status')).toHaveText('You have no passkeys yet.');
    await expect(page.locator('#passkey-list')).toBeEmpty();
    await fits(page);
    expect(problems, 'CSP violations and page errors').toEqual([]);
  } finally {
    await other.close();
  }
});
