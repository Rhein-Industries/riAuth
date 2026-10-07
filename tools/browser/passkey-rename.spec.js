// Headless passkey rename on a 390×844 CSS viewport.
//
// Already covered elsewhere, and not repeated here:
// - password, TOTP, consent, logout, and passkey enrollment sign-in (signin.spec.js)
// - removing a passkey, which ends every session (passkey-revocation.spec.js)
// - authenticator-app enrollment and recovery-code sign-in (authenticator-recovery.spec.js)
// - email password reset, which this fixture cannot deliver (accessibility-journeys.spec.js)
//
// The fixture's passkey account starts with a password and no passkey. This journey enrolls
// one passkey through Playwright's WebAuthn shim, signs in with that credential, and renames
// it from the keyboard. An empty name and a whitespace-only name leave the stored passkey
// unchanged and return focus to the name field. The saved name is trimmed. The passkey id
// and the virtual authenticator credential stay the same, the new name survives a reload,
// and that same credential signs in again. Rename does not end the account's other browser
// session. The shim is not a physical security key, a synced platform passkey, a phone, or
// iOS or Android. The viewport is CSS only. An engine whose page has no WebAuthn is skipped.
// Hardware authenticators, real mobile devices, and email recovery stay open.
import { test, expect } from '@playwright/test';
import { fixtureStartupMs, startFixture } from './fixture.js';

const VIEWPORT = { width: 390, height: 844 };
const ENROLLED_NAME = 'Desk key';
const RENAMED_NAME = 'Pocket key';
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
// The passkey row buttons have an accessible name and no id.
async function tabToLabel(page, browserName, label, limit = 40) {
  await releaseHiddenFocus(page);
  const key = tabKey(browserName, false);
  const seen = [];
  for (let i = 0; i <= limit; i += 1) {
    const current = await focusLabel(page);
    if (current.includes(`[${label}]`)) return;
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
async function within(locator) {
  const box = await locator.boundingBox();
  expect(box, 'control is inside the 390×844 viewport').not.toBeNull();
  expect(box.x).toBeGreaterThanOrEqual(0);
  expect(box.y).toBeGreaterThanOrEqual(0);
  expect(box.x + box.width).toBeLessThanOrEqual(VIEWPORT.width + 1);
  expect(box.y + box.height).toBeLessThanOrEqual(VIEWPORT.height + 1);
  return box;
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
async function expectSignedIn(page, browserName) {
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
  await expect(page.locator('#account-name')).toHaveText('Pat Passkey');
  await expect(page.locator('#auth')).toBeHidden();
  expect(await portalStatus(page)).toBe(200);
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
  await expect(page.locator('#security-account')).toHaveText('Pat Passkey (@passkey1)');
  await within(page.locator('#security-title'));
  await fits(page);
  return securityPage;
}
async function replaceFocusedText(page, value) {
  await page.keyboard.press('ControlOrMeta+A');
  if (value) await page.keyboard.type(value);
  else await page.keyboard.press('Backspace');
}
async function storedPasskey(page) {
  return page.evaluate(async (url) => {
    const response = await fetch(url, { credentials: 'same-origin' });
    if (!response.ok) throw new Error(`passkeys ${response.status}`);
    const body = await response.json();
    const passkeys = Array.isArray(body.passkeys) ? body.passkeys : [];
    if (passkeys.length !== 1) throw new Error(`expected one passkey, saw ${passkeys.length}`);
    return { id: passkeys[0].id, name: passkeys[0].name };
  }, `${fixture.issuer}/api/portal/passkeys`);
}
async function authenticatorIdentity(context) {
  const rows = await context.credentials.get({ rpId: 'localhost' });
  expect(rows).toHaveLength(1);
  const [row] = rows;
  return { id: row.id, rpId: row.rpId, userHandle: row.userHandle, publicKey: row.publicKey };
}
async function signInWithPasskey(page, browserName) {
  const button = page.locator('#passkey-login');
  await expect(button).toBeVisible();
  if (await focusedId(page) !== 'passkey-login') await tabTo(page, browserName, 'passkey-login', 25);
  expect((await within(button)).height).toBeGreaterThanOrEqual(24);
  await page.keyboard.press('Enter');
  await expectSignedIn(page, browserName);
  await expect(page.locator('#mfa-notice')).toBeHidden();
}

test('renaming a passkey keeps the credential and the other session', async ({ context, browser, browserName }) => {
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
    await keyboardSignIn(page, browserName, user);
    await expectSignedIn(page, browserName);
    await expect(page.locator('#mfa-notice')).toBeVisible();
    await expect(page.locator('#mfa-notice-text')).toHaveText('Some applications need extra verification. Add a passkey or an authenticator app under Sign-in and security.');

    await openSecurity(page, browserName);
    await expect(page.locator('#security-status')).toHaveText('You have no passkeys yet.');
    await expect(page.locator('#passkey-list')).toBeEmpty();
    await expect(page.locator('#reauth-panel')).toBeHidden();
    await tabToPasskeyName(page, browserName);
    await replaceFocusedText(page, ENROLLED_NAME);
    await expect(page.locator('#passkey-name')).toHaveValue(ENROLLED_NAME);
    await tabToControl(page, browserName, 'add-passkey', 10);
    expect((await within(page.locator('#add-passkey'))).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    await expect(page.locator('#toast')).toHaveText('Passkey added. Sign in with it to continue.');
    await expect(page.locator('#auth')).toBeVisible();
    await expect(page.locator('#catalogue')).toBeHidden();
    expect(await portalStatus(page)).toBe(401);
    await expect.poll(async () => Boolean(await sso(context))).toBe(false);

    // Adding the passkey ended this session. Signing in with it makes the session fresh and MFA,
    // which rename requires. The other browser signs in only after that, so its session is live.
    await signInWithPasskey(page, browserName);
    const credential = await authenticatorIdentity(context);
    await otherPage.goto(`${fixture.issuer}/apps`);
    await keyboardSignIn(otherPage, browserName, user);
    await expectSignedIn(otherPage, browserName);
    const otherSession = await sso(other);
    expect(otherSession?.value).toMatch(/^ri_sso_/);
    expect(await portalStatus(otherPage)).toBe(200);

    await openSecurity(page, browserName);
    await expect(page.locator('#security-status')).toHaveText(/^1 of \d+ passkeys\./);
    await expect(page.locator('#reauth-panel')).toBeHidden();
    await expect(page.locator('#passkey-list strong')).toHaveText(ENROLLED_NAME);
    const stored = await storedPasskey(page);
    expect(stored.name).toBe(ENROLLED_NAME);
    expect(stored.id).toMatch(/\S/);
    let renamePosts = 0;
    page.on('request', (request) => {
      if (request.method() === 'POST' && new URL(request.url()).pathname.endsWith('/rename')) renamePosts += 1;
    });

    await tabToLabel(page, browserName, `Rename ${ENROLLED_NAME}`);
    const rename = page.getByRole('button', { name: `Rename ${ENROLLED_NAME}` });
    await expect(rename).toBeFocused();
    expect((await within(rename)).height).toBeGreaterThanOrEqual(24);
    await page.keyboard.press('Enter');
    const field = page.locator('#passkey-rename');
    await expect(page.locator('#passkey-action-title')).toHaveText(`Rename ${ENROLLED_NAME}`);
    await expect(page.locator('#passkey-action-description')).toHaveText('Choose a name that helps you recognize this device or security key.');
    await expect(page.locator('#passkey-action-confirm')).toHaveText('Save name');
    await expect(field).toBeFocused();
    await expect(field).toHaveValue(ENROLLED_NAME);
    await expect(field).toHaveAttribute('maxlength', '200');
    await within(field);

    await replaceFocusedText(page, '');
    await expect(field).toHaveValue('');
    await page.keyboard.press('Enter');
    await expect(page.locator('#security-status')).toHaveText('Enter a name for this passkey.');
    await expect(field).toBeFocused();
    await expect(page.locator('#passkey-list strong')).toHaveText(ENROLLED_NAME);
    await expect(page.locator('#reauth-panel')).toBeHidden();
    expect(renamePosts).toBe(0);
    await fits(page);

    await replaceFocusedText(page, '   ');
    await expect(field).toHaveValue('   ');
    await page.keyboard.press('Enter');
    await expect(page.locator('#security-status')).toHaveText('Enter a name for this passkey.');
    await expect(field).toBeFocused();
    await expect(page.locator('#passkey-list strong')).toHaveText(ENROLLED_NAME);
    expect(await storedPasskey(page)).toEqual(stored);
    expect(await authenticatorIdentity(context)).toEqual(credential);
    expect(renamePosts).toBe(0);

    await replaceFocusedText(page, `  ${RENAMED_NAME}  `);
    await expect(field).toHaveValue(`  ${RENAMED_NAME}  `);
    await tabToControl(page, browserName, 'passkey-action-confirm', 10);
    const confirm = page.locator('#passkey-action-confirm');
    await expect(confirm).toBeFocused();
    await expect(confirm).toHaveText('Save name');
    expect((await within(confirm)).height).toBeGreaterThanOrEqual(24);
    const posted = page.waitForResponse((response) => response.request().method() === 'POST' && new URL(response.url()).pathname.endsWith('/rename'));
    await page.keyboard.press('Enter');
    const response = await posted;
    expect(response.status()).toBe(200);
    const body = await response.json();
    expect(body.renamed).toBe(true);
    expect(body.sessions_revoked).toBe(false);
    expect(body.passkey.id).toBe(stored.id);
    expect(body.passkey.name).toBe(RENAMED_NAME);
    expect(renamePosts).toBe(1);

    await expect(page.locator('#security-status')).toHaveText('Passkey renamed.');
    await expect(page.locator('#passkey-action')).toBeHidden();
    await expect(page.locator('#catalogue')).toBeHidden();
    await expect(page.locator('#security-page')).toBeVisible();
    await expect(page.locator('#passkey-list strong')).toHaveText(RENAMED_NAME);
    await expect(page.getByRole('button', { name: `Rename ${RENAMED_NAME}` })).toBeVisible();
    await expect(page.getByRole('button', { name: `Rename ${ENROLLED_NAME}` })).toHaveCount(0);
    await expect(page.getByRole('button', { name: `Remove ${RENAMED_NAME}` })).toBeVisible();
    const renamed = await storedPasskey(page);
    expect(renamed).toEqual({ id: stored.id, name: RENAMED_NAME });
    expect(await authenticatorIdentity(context)).toEqual(credential);
    expect((await sso(other))?.value).toBe(otherSession.value);
    expect(await portalStatus(otherPage)).toBe(200);
    expect(await portalStatus(page)).toBe(200);
    await fits(page);

    await page.reload();
    await expect(page).toHaveURL(`${fixture.issuer}/apps/security`);
    await expect(page.locator('#security-page')).toBeVisible();
    await expect(page.locator('#security-title')).toBeFocused();
    await expect(page.locator('#catalogue')).toBeHidden();
    expect(await portalStatus(page)).toBe(200);
    await expect(page.locator('#security-status')).toHaveText(/^1 of \d+ passkeys\./);
    await expect(page.locator('#passkey-list strong')).toHaveText(RENAMED_NAME);
    await expect(page.locator('#reauth-panel')).toBeHidden();
    expect(await storedPasskey(page)).toEqual(renamed);
    expect(await authenticatorIdentity(context)).toEqual(credential);
    await otherPage.reload();
    await expectSignedIn(otherPage, browserName);
    expect((await sso(other))?.value).toBe(otherSession.value);

    await page.keyboard.press('Escape');
    await expect(page.locator('#security-page')).toBeVisible();
    await expect(page).toHaveURL(`${fixture.issuer}/apps/security`);
    await tabToControl(page, browserName, 'security-close', 50);
    await page.keyboard.press('Enter');
    await expect(page.locator('#catalogue')).toBeVisible();
    await tabBack(page, browserName, 'sign-out', 20);
    const signOut = page.locator('#sign-out');
    expect((await within(signOut)).height).toBeGreaterThanOrEqual(24);
    await expect(signOut).not.toHaveAttribute('aria-disabled', 'true');
    await page.keyboard.press('Enter');
    await expect(page.locator('#toast')).toHaveText('You’re signed out.');
    await expect(page.locator('#auth')).toBeVisible();
    await expect(page.locator('#catalogue')).toBeHidden();
    await expect.poll(async () => Boolean(await sso(context))).toBe(false);
    expect(await portalStatus(page)).toBe(401);
    expect((await sso(other))?.value).toBe(otherSession.value);
    expect(await portalStatus(otherPage)).toBe(200);

    await signInWithPasskey(page, browserName);
    expect(await authenticatorIdentity(context)).toEqual(credential);
    await openSecurity(page, browserName);
    await expect(page.locator('#passkey-list strong')).toHaveText(RENAMED_NAME);
    expect(await storedPasskey(page)).toEqual(renamed);
    await expect(page.getByRole('button', { name: `Remove ${RENAMED_NAME}` })).toBeVisible();
    await fits(page);
    expect(problems, 'CSP violations and page errors').toEqual([]);
  } finally {
    await other.close();
  }
});
