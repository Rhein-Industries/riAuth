import { test, expect } from '@playwright/test';
import { fixtureStartupMs, startFixture } from './fixture.js';
let stopFixture, fixture;
test.beforeAll(async () => {
  test.setTimeout(fixtureStartupMs + 5000);
  ({ fixture, stop: stopFixture } = await startFixture());
});
test.afterAll(async () => { await stopFixture?.(); });

const bearer = () => ({ authorization: `Bearer ${fixture.token}` });
async function portalSignIn(page, user) {
  await page.goto(`${fixture.issuer}/apps`);
  await page.locator('#login-username').fill(user.username);
  await page.locator('#login-password').fill(user.password);
  await page.locator('#password-login').click();
  await expect(page.locator('#catalogue')).toBeVisible();
}
async function portalSignOut(page) {
  await page.goto(`${fixture.issuer}/apps`);
  await page.getByRole('button', { name: 'Sign out' }).click();
  await expect(page.locator('#login-username')).toBeVisible();
}
const stepTitle = (page) => page.locator('#wizard-step-title');
const next = (page) => page.getByRole('button', { name: 'Continue' }).click();
async function openWizard(page) {
  await page.goto(`${fixture.issuer}/admin#/applications/new`);
  await expect(stepTitle(page)).toHaveText('Step 1 of 6: Application');
}
async function storedClient(page, id) {
  const response = await page.request.get(`${fixture.issuer}/api/clients`, { headers: bearer() });
  expect(response.ok()).toBe(true);
  return (await response.json()).find((client) => client.client_id === id);
}
// Creates a web application and returns the one-time secret the wizard shows.
async function createWebApp(page, name, callback) {
  await openWizard(page);
  await page.getByLabel('Name', { exact: true }).fill(name);
  await next(page);
  await expect(stepTitle(page)).toHaveText('Step 2 of 6: Sign-in URLs');
  await page.getByLabel('Redirect URIs').fill(callback);
  await next(page);
  for (const title of ['Step 3 of 6: Access', 'Step 4 of 6: Claims', 'Step 5 of 6: Credentials']) {
    await expect(stepTitle(page)).toHaveText(title);
    await next(page);
  }
  await expect(stepTitle(page)).toHaveText('Step 6 of 6: Review');
  await page.getByRole('button', { name: 'Create application' }).click();
  await expect(page.getByRole('heading', { level: 1, name: `Connect ${name}` })).toBeVisible();
  const secret = (await page.locator('.connection-facts code', { hasText: /^ri_client_/ }).innerText()).trim();
  expect(secret).toMatch(/^ri_client_/);
  return secret;
}
const secretGone = async (page, secret) => {
  expect(await page.content()).not.toContain(secret);
  await expect(page.locator('#secret-value')).toHaveValue('');
};

test('a service is set up step by step with the API scopes it was given', async ({ page }) => {
  await portalSignIn(page, fixture.admin);
  await openWizard(page);
  await page.getByLabel('Name', { exact: true }).fill('Billing sync');
  await page.getByRole('radio', { name: /^Service/ }).check();
  // The Application step is checked before any API scope has been collected.
  await next(page);
  await expect(stepTitle(page)).toHaveText('Step 2 of 4: API access');
  await next(page);
  await expect(page.getByRole('alert')).toContainText('Enter at least one API scope');
  await page.getByLabel('API scopes').fill('billing.read billing.write');
  await next(page);
  await expect(stepTitle(page)).toHaveText('Step 3 of 4: Credentials');
  await next(page);
  await expect(stepTitle(page)).toHaveText('Step 4 of 4: Review');
  const review = page.locator('.review-facts');
  await expect(review).toContainText('billing.read');
  await expect(page.locator('#view')).not.toContainText('setup.pending');
  await page.getByRole('button', { name: 'Create application' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Connect Billing sync' })).toBeVisible();
  const client = await storedClient(page, 'billing-sync');
  expect(client.service).toBe(true);
  expect([...client.scopes].sort()).toEqual(['billing.read', 'billing.write']);
});

test('the one-time secret is dropped when the session ends or the account changes', async ({ page, context }) => {
  // Session loss in this tab: the next refresh shows the sign-in gate without the secret.
  await portalSignIn(page, fixture.admin);
  let secret = await createWebApp(page, 'Reports one', 'https://reports-one.example.com/callback');
  await context.clearCookies();
  await page.getByRole('button', { name: 'Refresh' }).click();
  await expect(page.locator('#gate-title')).toHaveText('Sign in to administer riAuth');
  await secretGone(page, secret);

  // Another person signs in from a second tab; this tab then sees a non-administrator.
  await portalSignIn(page, fixture.admin);
  secret = await createWebApp(page, 'Reports two', 'https://reports-two.example.com/callback');
  const other = await context.newPage();
  await portalSignOut(other);
  await portalSignIn(other, fixture.users.bob);
  await page.getByRole('button', { name: 'Refresh' }).click();
  await expect(page.locator('#gate-title')).toHaveText('Administrator access required');
  await secretGone(page, secret);
  await page.evaluate(() => { location.hash = '#/applications'; location.hash = '#/applications/new'; });
  await secretGone(page, secret);

  // Another administrator: the new session never renders the previous one's secret.
  const ops = { username: 'ops', password: 'another fixture administrator 2026' };
  const created = await page.request.post(`${fixture.issuer}/api/users`, { headers: bearer(), data: { ...ops, admin: true } });
  expect(created.ok()).toBe(true);
  await portalSignOut(other);
  await portalSignIn(other, fixture.admin);
  secret = await createWebApp(other, 'Reports three', 'https://reports-three.example.com/callback');
  await portalSignOut(page);
  await portalSignIn(page, ops);
  await other.getByRole('button', { name: 'Refresh' }).click();
  await expect(stepTitle(other)).toHaveText('Step 1 of 6: Application');
  await secretGone(other, secret);

  // Explicit sign-out from the administration page.
  await portalSignOut(other);
  await portalSignIn(other, fixture.admin);
  secret = await createWebApp(other, 'Reports four', 'https://reports-four.example.com/callback');
  await other.locator('#sign-out').click();
  await expect(other.locator('#gate-title')).toHaveText('Sign in to administer riAuth');
  await secretGone(other, secret);
});

test('a refresh keeps the fields typed on the open step', async ({ page }) => {
  await portalSignIn(page, fixture.admin);
  await openWizard(page);
  await page.getByLabel('Name', { exact: true }).fill('Unsaved name');
  // Mark the rendered form so the assertion runs on the re-rendered one.
  await page.locator('.wizard-form').evaluate((form) => { form.dataset.before = 'refresh'; });
  await page.getByRole('button', { name: 'Refresh' }).click();
  await expect(page.locator('.wizard-form[data-before]')).toHaveCount(0);
  await expect(page.getByLabel('Name', { exact: true })).toHaveValue('Unsaved name');
});

test('an idle tab checks the account on focus before showing a one-time secret', async ({ page, context }) => {
  const focus = () => page.evaluate(() => window.dispatchEvent(new Event('focus')));
  const secretField = () => page.locator('.connection-facts code', { hasText: /^ri_client_/ });
  await portalSignIn(page, fixture.admin);
  const secret = await createWebApp(page, 'Idle one', 'https://idle-one.example.com/callback');
  // The same administrator returning sees the secret again.
  await focus();
  await expect(secretField()).toBeVisible();
  await expect(secretField()).toHaveText(secret);

  // Another administrator signs in from a second tab; this tab is idle, never refreshed.
  const idle = { username: 'idle-admin', password: 'idle tab fixture administrator 2026' };
  const created = await page.request.post(`${fixture.issuer}/api/users`, { headers: bearer(), data: { ...idle, admin: true } });
  expect(created.ok()).toBe(true);
  const other = await context.newPage();
  await portalSignOut(other);
  await portalSignIn(other, idle);
  // Hold the session check: the secret is covered before riAuth answers.
  let release;
  const held = new Promise((done) => { release = done; });
  await page.route('**/api/admin/session', async (route) => { await held; await route.continue(); });
  await focus();
  await expect(page.locator('#view')).toBeHidden();
  await expect(secretField()).toBeHidden();
  release();
  await expect(stepTitle(page)).toHaveText('Step 1 of 6: Application');
  await expect(page.locator('#account-detail')).toContainText('idle-admin');
  await page.unroute('**/api/admin/session');
  await secretGone(page, secret);
});
