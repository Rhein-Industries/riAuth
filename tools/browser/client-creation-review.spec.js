import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { fixtureStartupMs, startFixture } from './fixture.js';

let fixture, stopFixture;
const reviewer = { username: 'creation-reviewer', password: 'reviewer fixture password 2026' };
const executor = { username: 'creation-executor', password: 'executor fixture password 2026' };
const bearer = () => ({ authorization: `Bearer ${fixture.token}` });
test.beforeAll(async ({ request }) => {
  test.setTimeout(fixtureStartupMs + 20000);
  ({ fixture, stop: stopFixture } = await startFixture({ reviewedClientCreation: true }));
  for (const user of [reviewer, executor]) {
    const response = await request.post(`${fixture.issuer}/api/users`, { headers: bearer(), data: { admin: true, ...user } });
    expect(response.ok()).toBe(true);
  }
  expect((await request.post(`${fixture.issuer}/api/groups`, { headers: bearer(), data: { name: 'creation-staff' } })).ok()).toBe(true);
});
test.afterAll(async () => { await stopFixture?.(); });

async function signIn(page, user) {
  await page.goto(`${fixture.issuer}/apps`);
  await page.locator('#login-username').fill(user.username);
  await page.locator('#login-password').fill(user.password);
  await page.locator('#password-login').click();
  await expect(page.locator('#catalogue')).toBeVisible();
}
async function open(page, id) {
  await page.goto(`${fixture.issuer}/admin#/client-creation-review/${id}`);
  await expect(page.locator('#creation-digest')).toBeVisible();
}
async function draft(page, client, type = 'web') {
  // The ordinary application entry point must reach review when configured.
  await page.goto(`${fixture.issuer}/admin#/applications`);
  await page.getByRole('link', { name: 'New application', exact: true }).click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Reviewed applications');
  await page.getByLabel('Application name', { exact: true }).fill('Reviewed reports');
  await page.getByLabel('Client ID', { exact: true }).fill(client);
  await page.getByLabel('Application type', { exact: true }).selectOption(type);
  if (type !== 'service') {
    await page.getByLabel('Redirect URIs', { exact: true }).fill('https://reports.example.test/callback');
    await page.getByLabel('Allowed groups', { exact: true }).fill('creation-staff');
    await page.getByLabel('Require MFA', { exact: true }).check();
  }
  await page.getByLabel('I checked the complete application content.').check();
}
async function stage(page, client, type = 'web') {
  await draft(page, client, type);
  await page.getByRole('button', { name: 'Stage exact application', exact: true }).click();
  await expect(page.locator('#creation-status')).toHaveText('Awaiting review');
  return page.locator('#creation-id').innerText();
}
async function acknowledge(page) {
  await page.getByLabel('I checked the exact application, digest and dependencies.').check();
}

test('wizard stages exact creation, separates actors, recovers one execution and erases its secret', async ({ page, browser }) => {
  test.setTimeout(90000);
  await signIn(page, fixture.admin);
  await draft(page, 'browser-reviewed');
  await page.getByRole('button', { name: 'Refresh', exact: true }).click();
  await expect(page.getByLabel('Client ID', { exact: true })).toHaveValue('browser-reviewed');
  await expect(page.getByLabel('I checked the complete application content.')).not.toBeChecked();
  await page.getByLabel('I checked the complete application content.').check();
  const attempts = [];
  let change;
  await page.route('**/api/admin/client-creation-changes', async (route) => {
    attempts.push({ body: route.request().postDataJSON(), headers: route.request().headers() });
    if (attempts.length === 1) {
      const response = await route.fetch();
      expect(response.ok()).toBe(true);
      change = await response.json();
      await route.abort('failed');
    } else await route.continue();
  });
  await page.getByRole('button', { name: 'Stage exact application', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('response was lost or incomplete');
  await expect(page.getByLabel('Client ID', { exact: true })).toBeDisabled();
  await page.getByRole('button', { name: 'Refresh', exact: true }).click();
  await page.getByRole('button', { name: 'Retry same staging request' }).click();
  await expect(page.locator('#creation-status')).toHaveText('Awaiting review');
  expect(attempts).toHaveLength(2);
  expect(attempts[1].body).toEqual(attempts[0].body);
  expect(attempts[1].headers['idempotency-key']).toBe(attempts[0].headers['idempotency-key']);
  expect(attempts[1].headers['if-match']).toBe(attempts[0].headers['if-match']);
  expect(JSON.stringify(change)).not.toContain('ri_client_');
  expect(JSON.stringify(change)).not.toContain('"client_secret"');
  const id = await page.locator('#creation-id').innerText();
  const digest = await page.locator('#creation-digest').innerText();
  expect(id).toBe(change.proposal.id); expect(digest).toBe(change.digest);
  expect(JSON.parse(await page.locator('#creation-content').textContent())).toEqual(change.proposal.after);
  await expect(page.getByText(change.proposal.resource_revision, { exact: true })).toBeVisible();
  await expect(page.getByText(change.proposal.policy_revision, { exact: true })).toBeVisible();
  await expect(page.getByLabel('Review link')).toHaveValue(`${fixture.issuer}/admin#/client-creation-review/${id}`);
  await acknowledge(page);
  await expect(page.getByRole('button', { name: 'Approve exact application' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Create application once' })).toBeDisabled();
  await page.reload();
  await expect(page.getByLabel('I checked the exact application, digest and dependencies.')).not.toBeChecked();
  const accessibility = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  expect(accessibility.violations.map(({ id }) => id)).toEqual([]);
  await page.screenshot({ path: '/tmp/riauth-m05-creation-review-desktop.png', fullPage: true });
  await page.setViewportSize({ width: 320, height: 900 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: '/tmp/riauth-m05-creation-review-mobile.png', fullPage: true });

  const reviewContext = await browser.newContext(), executeContext = await browser.newContext();
  try {
    const review = await reviewContext.newPage(), execute = await executeContext.newPage();
    await signIn(review, reviewer); await signIn(execute, executor);
    await open(review, id); await acknowledge(review);
    await expect(review.getByRole('button', { name: 'Create application once' })).toBeDisabled();
    const approval = review.waitForRequest((r) => r.url().endsWith(`/${id}/approve`) && r.method() === 'POST');
    await review.getByRole('button', { name: 'Approve exact application' }).click();
    expect((await approval).postDataJSON()).toEqual({ digest });
    await expect(review.locator('#creation-status')).toHaveText('Approved');
    await acknowledge(review);
    await expect(review.getByRole('button', { name: 'Create application once' })).toBeDisabled();
    await open(execute, id); await acknowledge(execute); await execute.bringToFront();
    const executions = [];
    let execution;
    await execute.route(`**/api/admin/client-creation-changes/${id}/execute`, async (route) => {
      executions.push({ body: route.request().postDataJSON(), headers: route.request().headers() });
      if (executions.length === 1) {
        const response = await route.fetch();
        expect(response.ok()).toBe(true);
        execution = await response.json();
        await route.abort('failed');
      } else await route.continue();
    });
    await execute.getByRole('button', { name: 'Create application once' }).click();
    await expect(execute.locator('#creation-status')).toHaveText('Outcome unknown');
    await expect(execute.getByRole('button', { name: 'Create application once' })).toBeDisabled();
    await execute.getByRole('button', { name: 'Refresh change' }).click();
    await expect(execute.locator('#creation-status')).toHaveText('Outcome unknown');
    await execute.getByRole('button', { name: 'Recover execution result' }).click();
    await expect(execute.locator('#creation-status')).toHaveText('Executed');
    const secret = execution.client_secret;
    await expect(execute.getByLabel('One-time client secret')).toHaveValue(secret);
    expect(executions).toHaveLength(2);
    expect(executions[0].body).toEqual({ digest });
    expect(executions[1]).toEqual(executions[0]);
    expect(await execute.evaluate((secret) => JSON.stringify([localStorage, sessionStorage]).includes(secret), secret)).toBe(false);
    await execute.evaluate(() => window.dispatchEvent(new Event('blur')));
    await expect(execute.getByLabel('One-time client secret')).toHaveCount(0);
    await expect(execute.getByText(/one-time secret was erased/)).toBeVisible();
    await execute.getByRole('button', { name: 'Refresh', exact: true }).click();
    await expect(execute.locator('#creation-status')).toHaveText('Executed');
    await expect(execute.getByLabel('One-time client secret')).toHaveCount(0);
    await execute.reload();
    await expect(execute.locator('#creation-status')).toHaveText('Executed');
    expect(await execute.content()).not.toContain(secret);
    await expect(execute.getByRole('button', { name: 'Recover execution result' })).toBeHidden();
    const replay = await execute.request.post(`${fixture.issuer}/api/admin/client-creation-changes/${id}/execute`, {
      headers: { 'x-riauth-portal': '1', origin: new URL(fixture.issuer).origin }, data: { digest },
    });
    expect(replay.status()).toBe(409);
    const events = await (await page.request.get(`${fixture.issuer}/api/audit?limit=100`, { headers: bearer() })).json();
    expect(events.filter((e) => e.action === 'reviewed_client_creations.execute' && e.details.change_id === id)).toHaveLength(1);
  } finally { await reviewContext.close(); await executeContext.close(); }
});

test('stale, cancelled, expired and rejected reviews stay closed across session changes', async ({ page, browser }) => {
  await signIn(page, fixture.admin);
  await draft(page, 'stale-browser');
  const originalRevision = await page.getByText(/^Based on management revision/).innerText();
  expect((await page.request.patch(`${fixture.issuer}/api/users/bob`, { headers: bearer(), data: { display_name: 'Changed while drafting' } })).ok()).toBe(true);
  await page.getByRole('button', { name: 'Refresh', exact: true }).click();
  await expect(page.getByLabel('I checked the complete application content.')).not.toBeChecked();
  await expect(page.getByText(/^Based on management revision/)).toHaveText(originalRevision);
  await expect(page.getByLabel('Client ID', { exact: true })).toHaveValue('stale-browser');
  await page.getByLabel('I checked the complete application content.').check();
  await page.getByRole('button', { name: 'Stage exact application', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('server refused');
  await expect(page.getByRole('button', { name: 'Stage exact application', exact: true })).toBeDisabled();
  await page.getByRole('button', { name: 'Use current revision' }).click();
  await expect(page.getByLabel('I checked the complete application content.')).not.toBeChecked();
  await page.getByLabel('I checked the complete application content.').check();
  await page.getByRole('button', { name: 'Stage exact application', exact: true }).click();
  await expect(page.locator('#creation-status')).toHaveText('Awaiting review');
  const stale = await page.locator('#creation-id').innerText();
  expect((await page.request.patch(`${fixture.issuer}/api/users/bob`, { headers: bearer(), data: { display_name: 'Changed after staging' } })).ok()).toBe(true);
  await page.getByRole('button', { name: 'Refresh change' }).click();
  await expect(page.locator('#creation-status')).toHaveText('Stale');
  await expect(page.getByRole('button', { name: 'Approve exact application' })).toBeDisabled();
  await page.getByRole('button', { name: 'Cancel change' }).click();
  await expect(page.locator('#creation-status')).toHaveText('Cancelled');
  const denied = await stage(page, 'denied-browser');
  const context = await browser.newContext();
  try {
    const review = await context.newPage();
    await signIn(review, reviewer); await open(review, denied); await acknowledge(review);
    const privateDetail = 'private-database-detail <img src=x onerror=alert(1)>';
    await review.route(`**/api/admin/client-creation-changes/${denied}/approve`, (route) => route.fulfill({
      status: 403, contentType: 'application/json', body: JSON.stringify({ error: 'access_denied', error_description: privateDetail }),
    }));
    await review.getByRole('button', { name: 'Approve exact application' }).click();
    await expect(review.getByRole('alert')).toContainText('currently authorized administrators');
    await expect(review.locator('#creation-status')).toHaveText('Stale');
    expect(await review.content()).not.toContain(privateDetail);
    await review.unrouteAll({ behavior: 'wait' });
    await open(review, stale);
    await expect(review.locator('#creation-status')).toHaveText('Cancelled');
    await open(review, denied); await acknowledge(review);
    let approvals = 0;
    review.on('request', (r) => { if (r.method() === 'POST' && r.url().endsWith(`/${denied}/approve`)) approvals += 1; });
    await context.clearCookies();
    const other = await context.newPage(); await signIn(other, executor);
    await review.evaluate(() => [...document.querySelectorAll('button')].find((b) => b.textContent === 'Approve exact application').click());
    await expect(review.locator('#account-detail')).toContainText('creation-executor');
    await expect(review.locator('#creation-status')).toHaveText('Awaiting review');
    expect(approvals).toBe(0);
    await expect(review.getByLabel('I checked the exact application, digest and dependencies.')).not.toBeChecked();
    await other.close();
    await review.clock.install(); await review.clock.fastForward(16 * 60 * 1000);
    await expect(review.locator('#creation-status')).toHaveText('Expired');
    await expect(review.getByRole('button', { name: 'Create application once' })).toBeDisabled();
    await context.clearCookies();
    await review.getByRole('button', { name: 'Refresh', exact: true }).click();
    await expect(review.locator('#gate-title')).toHaveText('Sign in to administer riAuth');
    await expect(review.locator('#creation-digest')).toHaveCount(0);
  } finally { await context.close(); }
});

test('public creation has no secret and a service response arriving after blur cannot reveal one', async ({ page, browser }) => {
  test.setTimeout(90000);
  await signIn(page, fixture.admin);
  const reviewContext = await browser.newContext(), executeContext = await browser.newContext();
  try {
    const review = await reviewContext.newPage(), execute = await executeContext.newPage();
    await signIn(review, reviewer); await signIn(execute, executor);
    for (const type of ['spa', 'service']) {
      const id = await stage(page, `browser-${type}`, type);
      await open(review, id); await acknowledge(review);
      await review.getByRole('button', { name: 'Approve exact application' }).click();
      await expect(review.locator('#creation-status')).toHaveText('Approved');
      await open(execute, id); await acknowledge(execute); await execute.bringToFront();
      let generated;
      if (type === 'service') await execute.route(`**/api/admin/client-creation-changes/${id}/execute`, async (route) => {
        const response = await route.fetch(); expect(response.ok()).toBe(true);
        generated = (await response.json()).client_secret;
        await execute.evaluate(() => window.dispatchEvent(new Event('blur')));
        await route.fulfill({ response });
      }, { times: 1 });
      await execute.getByRole('button', { name: 'Create application once' }).click();
      await expect(execute.locator('#creation-status')).toHaveText('Executed');
      await expect(execute.getByLabel('One-time client secret')).toHaveCount(0);
      if (type === 'service') {
        expect(generated).toMatch(/^ri_client_/);
        expect(await execute.content()).not.toContain(generated);
        await expect(execute.getByText(/one-time secret was erased/)).toBeVisible();
      }
      const clients = await (await page.request.get(`${fixture.issuer}/api/clients`, { headers: bearer() })).json();
      expect(clients.find((c) => c.client_id === `browser-${type}`).confidential).toBe(type === 'service');
    }
  } finally { await reviewContext.close(); await executeContext.close(); }
});
