// Local fixture journeys for keyboard, axe (WCAG 2.1 A/AA), 320/768/1440 reflow and 200% text.
// Chromium's virtual internal authenticator is the only authenticator this file drives.
// It does not claim a physical security key, a synced platform passkey, a phone hybrid
// transport, iOS or Android, or a spoken screen reader (VoiceOver, TalkBack, NVDA).
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { fixtureStartupMs, startFixture } from './fixture.js';

let stopFixture, fixture;
test.beforeAll(async () => {
  test.setTimeout(fixtureStartupMs + 5000);
  ({ fixture, stop: stopFixture } = await startFixture());
});
test.afterAll(async () => { await stopFixture?.(); });

async function axe(page) {
  const report = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
  expect(report.violations.map(({ id, nodes }) => ({ id, nodes: nodes.map(({ target, failureSummary }) => ({ target, failureSummary })) }))).toEqual([]);
}
async function fits(page) {
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1)).toBe(true);
}
async function reflow(page) {
  const original = page.viewportSize();
  for (const width of [1440, 768, 320]) {
    await page.setViewportSize({ width, height: 800 });
    await fits(page);
  }
  await page.setViewportSize(original);
  await page.evaluate(() => {
    const nodes = [...document.body.querySelectorAll('h1,h2,h3,p,a,button,input,label,li,summary')];
    const sizes = nodes.map((node) => parseFloat(getComputedStyle(node).fontSize));
    nodes.forEach((node, index) => { node.style.fontSize = `${sizes[index] * 2}px`; });
  });
  await fits(page);
}
const tabKey = (browserName) => (browserName === 'webkit' && process.platform === 'darwin' ? 'Alt+Tab' : 'Tab');
async function focusedId(page) {
  return page.evaluate(() => document.activeElement?.id ?? '');
}
async function tabTo(page, browserName, id, limit = 40) {
  for (let i = 0; i < limit; i += 1) {
    if (await focusedId(page) === id) return;
    await page.keyboard.press(tabKey(browserName));
  }
  throw new Error(`Tab did not reach #${id}`);
}
async function onScreen(locator) {
  return locator.evaluate((el) => {
    const box = el.getBoundingClientRect();
    return box.width > 0 && box.height > 0 && box.top >= 0 && box.left >= 0 && box.bottom <= window.innerHeight + 1 && box.right <= window.innerWidth + 1;
  });
}

test('device approval stays keyboard reachable on a small screen', async ({ page, browserName }) => {
  test.setTimeout(90000);
  await page.goto(`${fixture.issuer}/device`);
  await expect(page.locator('#device-code')).toBeFocused();
  await axe(page);
  await reflow(page);
  await page.reload();
  await page.setViewportSize({ width: 320, height: 700 });
  await expect(page.locator('#device-code')).toBeFocused();
  await page.keyboard.type('short');
  await page.keyboard.press('Enter');
  await expect(page.locator('#device-enter-error')).toBeFocused();
  await expect(page.locator('#device-enter-error')).toHaveText('Enter the ten-character code shown on your device.');
  expect(await onScreen(page.locator('#device-enter-error'))).toBe(true);
  await tabTo(page, browserName, 'device-code');
  await fits(page);
});

test('password reset focuses the unavailable-recovery alert', async ({ page, browserName }) => {
  test.setTimeout(90000);
  // This fixture has no mail delivery, so the request form never opens.
  await page.goto(`${fixture.issuer}/account/reset`);
  const error = page.locator('#account-error');
  await expect(error).toBeFocused();
  await expect(error).toHaveText("Password reset by email isn't available on this server. Contact your administrator.");
  await axe(page);
  await reflow(page);
  await page.reload();
  await page.setViewportSize({ width: 320, height: 640 });
  await expect(error).toBeFocused();
  expect(await onScreen(error)).toBe(true);
  const link = page.locator('#account-fallback a');
  for (let i = 0; i < 20; i += 1) {
    if (await link.evaluate((el) => el === document.activeElement)) break;
    await page.keyboard.press(tabKey(browserName));
  }
  await expect(link).toBeFocused();
  await fits(page);
});

test('sessions reauthentication keeps an empty-password error in view', async ({ page, browserName }) => {
  test.setTimeout(90000);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${fixture.issuer}/apps`);
  const start = page.getByRole('button', { name: /sign in with your terminal/i });
  await expect(start).toBeVisible();
  await start.click();
  await expect(page.locator('#user-code')).not.toBeEmpty();
  const code = (await page.locator('#user-code').innerText()).trim();
  const approved = await page.request.post(`${fixture.issuer}/api/portal/requests/${encodeURIComponent(code)}`, {
    headers: { authorization: `Bearer ${fixture.token}` }, data: { approve: true }
  });
  expect(approved.ok(), await approved.text()).toBe(true);
  await expect(page.getByRole('link', { name: 'Open Fixture application (opens in a new tab)', exact: true })).toBeVisible({ timeout: 15000 });
  await page.goto(`${fixture.issuer}/account/security`);
  await expect(page.locator('#verify-password')).toBeVisible();
  await expect(page.locator('#verify-hint')).toContainText('terminal');
  await axe(page);
  await reflow(page);
  await page.reload();
  await page.setViewportSize({ width: 320, height: 480 });
  await expect(page.locator('#verify-password')).toBeVisible();
  await page.locator('#verify-password').focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('#verify-error')).toBeFocused();
  await expect(page.locator('#verify-error')).toHaveText('Enter your password.');
  await expect(page.locator('#verify-password')).toHaveAttribute('aria-invalid', 'true');
  await expect(page.locator('#verify-password')).toHaveAttribute('aria-describedby', 'verify-error');
  const outline = await page.locator('#verify-error').evaluate((el) => getComputedStyle(el).outlineStyle);
  expect(outline).not.toBe('none');
  expect(await onScreen(page.locator('#verify-error'))).toBe(true);
  await tabTo(page, browserName, 'verify-password');
  await page.keyboard.type('x');
  await expect(page.locator('#verify-password')).not.toHaveAttribute('aria-invalid', 'true');
  await fits(page);
  for (const width of [768, 1440]) {
    await page.setViewportSize({ width, height: 800 });
    await fits(page);
  }
});

test('sign-in and security dialog reaches its close control at 320 pixels', async ({ page, browserName }) => {
  test.setTimeout(90000);
  const user = fixture.users.bob;
  await page.goto(`${fixture.issuer}/apps`);
  await expect(page.locator('#login-username')).toBeVisible();
  await page.locator('#login-username').fill(user.username);
  await page.locator('#login-password').fill(user.password);
  await page.locator('#password-login').click();
  await expect(page.locator('#catalogue')).toBeVisible();
  await page.locator('#account-security').click();
  const dialog = page.getByRole('dialog', { name: 'Sign-in and security' });
  await expect(dialog).toBeVisible();
  await axe(page);
  await page.setViewportSize({ width: 320, height: 640 });
  await expect(dialog).toBeVisible();
  await tabTo(page, browserName, 'security-close', 50);
  const inside = await page.locator('#security-close').evaluate((el) => {
    const box = el.getBoundingClientRect();
    const frame = el.closest('dialog').getBoundingClientRect();
    return box.top >= frame.top - 1 && box.bottom <= frame.bottom + 1 && box.left >= frame.left - 1 && box.right <= frame.right + 1;
  });
  expect(inside).toBe(true);
  await fits(page);
  for (const width of [768, 1440]) {
    await page.setViewportSize({ width, height: 800 });
    await fits(page);
    await expect(page.locator('#security-close')).toBeVisible();
  }
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.evaluate(() => {
    const dialog = document.getElementById('security-dialog');
    for (const node of dialog.querySelectorAll('h2,h3,h4,p,a,button,input,label,li')) {
      node.style.fontSize = `${parseFloat(getComputedStyle(node).fontSize) * 2}px`;
    }
  });
  await fits(page);
  const dialogFits = await page.locator('#security-dialog').evaluate((el) => el.scrollWidth <= el.clientWidth + 1);
  expect(dialogFits).toBe(true);
});

test('signed-in account controls stay inside a 320 pixel viewport', async ({ page, browserName }) => {
  test.setTimeout(90000);
  const user = fixture.users.bob;
  await page.setViewportSize({ width: 320, height: 700 });
  await page.goto(`${fixture.issuer}/apps`);
  await page.locator('#login-username').fill(user.username);
  await page.locator('#login-password').fill(user.password);
  await page.locator('#password-login').click();
  await expect(page.locator('#catalogue')).toBeVisible();
  await axe(page);
  for (const id of ['nav-all', 'nav-favorites', 'account-security', 'sign-out']) {
    await tabTo(page, browserName, id);
    expect(await onScreen(page.locator(`#${id}`))).toBe(true);
  }
  const sidebarFits = await page.locator('.sidebar').evaluate((el) => el.scrollWidth <= el.clientWidth + 1);
  expect(sidebarFits).toBe(true);
  await fits(page);
});

test('keyboard cancellation of a Chromium virtual authenticator is announced', async ({ page, context, browserName }) => {
  test.skip(browserName !== 'chromium', 'CDP virtual authenticators exist only in Chromium');
  test.setTimeout(90000);
  const user = fixture.users.native;
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  const { authenticatorId } = await cdp.send('WebAuthn.addVirtualAuthenticator', {
    options: { protocol: 'ctap2', transport: 'internal', hasResidentKey: true, hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true }
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${fixture.issuer}/apps`);
  await page.locator('#login-username').fill(user.username);
  await page.locator('#login-password').fill(user.password);
  await page.locator('#password-login').click();
  await expect(page.locator('#catalogue')).toBeVisible();
  await page.locator('#account-security').click();
  await page.getByRole('button', { name: 'Add a passkey' }).click();
  await expect(page.locator('#toast')).toHaveText('Passkey added. Sign in with it to continue.');
  await expect(page.locator('#passkey-login')).toBeFocused();
  const { credentials } = await cdp.send('WebAuthn.getCredentials', { authenticatorId });
  expect(credentials).toHaveLength(1);
  await cdp.send('WebAuthn.setUserVerified', { authenticatorId, isUserVerified: false });
  await page.keyboard.press('Enter');
  await expect(page.locator('#auth-error')).toBeFocused();
  await expect(page.locator('#auth-error')).toHaveText('Passkey sign-in was cancelled or timed out. Select the button to try again.');
  expect(await onScreen(page.locator('#auth-error'))).toBe(true);
  await axe(page);
  await tabTo(page, browserName, 'passkey-login-cancel');
  await page.keyboard.press('Enter');
  await expect(page.locator('#passkey-login')).toBeFocused();
  await expect(page.locator('#passkey-login-cancel')).toBeHidden();
  await fits(page);
});
