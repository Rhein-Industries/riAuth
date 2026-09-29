import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { fixtureStartupMs, startFixture } from './fixture.js';

let fixture, stopFixture;
const reviewer = { username: 'm05-reviewer', password: 'reviewer fixture password 2026' };
const executor = { username: 'm05-executor', password: 'executor fixture password 2026' };
const bearer = () => ({ authorization: `Bearer ${fixture.token}` });
test.beforeAll(async ({ request }) => {
  test.setTimeout(fixtureStartupMs + 20000);
  ({ fixture, stop: stopFixture } = await startFixture());
  for (const user of [reviewer, executor, { username: 'm05-recipient', password: 'recipient fixture password 2026', admin: false }]) {
    const response = await request.post(`${fixture.issuer}/api/users`, { headers: bearer(), data: { admin: true, ...user } });
    expect(response.ok()).toBe(true);
  }
});
test.afterAll(async () => { await stopFixture?.(); });

async function signIn(page, user) {
  await page.goto(`${fixture.issuer}/apps`);
  await page.locator('#login-username').fill(user.username);
  await page.locator('#login-password').fill(user.password);
  await page.locator('#password-login').click();
  await expect(page.locator('#catalogue')).toBeVisible();
}
async function open(page, id = '') {
  await page.goto(`${fixture.issuer}/admin#/grant-review${id ? `/${id}` : ''}`);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(id ? 'Review grant change' : 'Reviewed grants');
}
async function draft(page, revoke = false) {
  await open(page);
  await page.getByLabel('Recipient username').fill('m05-recipient');
  await page.getByRole('button', { name: 'Load current grants' }).click();
  await expect(page.getByRole('heading', { name: 'Proposed grants for m05-recipient' })).toBeVisible();
  if (revoke) {
    await page.getByRole('button', { name: 'Remove grant 1', exact: true }).click();
  } else {
    await page.getByRole('button', { name: 'Add grant', exact: true }).click();
    await page.getByLabel('Role', { exact: true }).selectOption('security_administrator');
    await page.getByLabel('Exact target scope').fill('key/signing');
  }
  await page.getByLabel('I checked the complete replacement, including every grant being removed.').check();
}
async function stage(page, revoke = false) {
  await draft(page, revoke);
  await page.getByRole('button', { name: 'Stage exact change' }).click();
  await expect(page.locator('#grant-status')).toHaveText('Awaiting review');
  return page.locator('#grant-id').innerText();
}
async function acknowledge(page) {
  await page.getByLabel('I checked the exact grants, digest and dependencies.').check();
}

test('exact browser review separates participants and recovers lost responses without replaying grants', async ({ page, browser }) => {
  test.setTimeout(90000);
  await signIn(page, fixture.admin);
  await draft(page);
  const keys = [];
  let staged;
  await page.route('**/api/admin/users/m05-recipient/delegated-grants/changes', async (route) => {
    keys.push(route.request().headers()['idempotency-key']);
    if (keys.length === 1) {
      const response = await route.fetch();
      expect(response.ok()).toBe(true);
      staged = await response.json();
      await route.abort('failed');
    } else await route.continue();
  });
  await page.getByRole('button', { name: 'Stage exact change' }).click();
  await expect(page.getByRole('alert')).toContainText('staging response was lost');
  await expect(page.getByLabel('Exact target scope')).toBeDisabled();
  await page.getByRole('button', { name: 'Retry same staging request' }).click();
  await expect(page.locator('#grant-status')).toHaveText('Awaiting review');
  expect(keys).toHaveLength(2); expect(keys[0]).toBeTruthy(); expect(keys[1]).toBe(keys[0]);
  const id = await page.locator('#grant-id').innerText();
  expect(id).toBe(staged.proposal.id);
  const digest = await page.locator('#grant-digest').innerText();
  expect(digest).toBe(staged.digest);
  await expect(page.getByText(staged.proposal.resource_revision, { exact: true })).toBeVisible();
  await expect(page.getByText(staged.proposal.policy_revision, { exact: true })).toBeVisible();
  await expect(page.getByLabel('Review link')).toHaveValue(`${fixture.issuer}/admin#/grant-review/${id}`);
  await acknowledge(page);
  await expect(page.getByRole('button', { name: 'Approve exact change' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Execute once' })).toBeDisabled();
  const report = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  expect(report.violations.map(({ id }) => id)).toEqual([]);
  await page.setViewportSize({ width: 320, height: 900 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole('heading', { level: 1 }).focus();
  await page.evaluate(() => scrollTo(0, 0));
  await page.screenshot({ path: '/tmp/riauth-m05-review-mobile.png', fullPage: true });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.evaluate(() => scrollTo(0, 0));
  await page.screenshot({ path: '/tmp/riauth-m05-review-desktop.png', fullPage: true });

  const reviewerContext = await browser.newContext();
  const executorContext = await browser.newContext();
  try {
    const review = await reviewerContext.newPage();
    const execute = await executorContext.newPage();
    await signIn(review, reviewer); await signIn(execute, executor);
    await open(review, id); await acknowledge(review);
    await expect(review.getByRole('button', { name: 'Execute once' })).toBeDisabled();
    const approval = review.waitForRequest((request) => request.url().endsWith(`/${id}/approve`) && request.method() === 'POST');
    await review.getByRole('button', { name: 'Approve exact change' }).click();
    expect((await approval).postDataJSON()).toEqual({ digest });
    await expect(review.locator('#grant-status')).toHaveText('Approved');
    await acknowledge(review);
    await expect(review.getByRole('button', { name: 'Execute once' })).toBeDisabled();
    await open(execute, id); await acknowledge(execute);
    await execute.route(`**/api/admin/delegated-grant-changes/${id}/execute`, async (route) => {
      expect(route.request().postDataJSON()).toEqual({ digest });
      const response = await route.fetch();
      expect(response.ok()).toBe(true);
      await route.abort('failed');
    }, { times: 1 });
    await execute.getByRole('button', { name: 'Execute once' }).click();
    await expect(execute.locator('#grant-status')).toHaveText('Outcome unknown');
    await expect(execute.getByRole('button', { name: 'Execute once' })).toBeDisabled();
    await execute.getByRole('button', { name: 'Refresh change' }).click();
    await expect(execute.locator('#grant-status')).toHaveText('Executed');
    const replay = await execute.request.post(`${fixture.issuer}/api/admin/delegated-grant-changes/${id}/execute`, {
      headers: { 'x-riauth-portal': '1', origin: new URL(fixture.issuer).origin }, data: { digest },
    });
    expect(replay.status()).toBe(409);
    const events = await (await page.request.get(`${fixture.issuer}/api/audit?limit=100`, { headers: bearer() })).json();
    expect(events.filter((e) => e.action === 'reviewed_grants.execute' && e.details.change_id === id)).toHaveLength(1);

    const revoke = await stage(page, true);
    await expect(page.getByText('Execution will revoke all delegated grants for this recipient.')).toBeVisible();
    await open(review, revoke); await acknowledge(review);
    await review.getByRole('button', { name: 'Approve exact change' }).click();
    await expect(review.locator('#grant-status')).toHaveText('Approved');
    await open(execute, revoke); await acknowledge(execute);
    await execute.getByRole('button', { name: 'Execute once' }).click();
    await expect(execute.locator('#grant-status')).toHaveText('Executed');
    const grants = await (await page.request.get(`${fixture.issuer}/api/users/m05-recipient/delegated-grants`, { headers: bearer() })).json();
    expect(grants.grants).toEqual([]);
  } finally { await reviewerContext.close(); await executorContext.close(); }
});

test('stale, cancelled, expired and refused reviews stay closed and errors reveal no server detail', async ({ page, browser }) => {
  await signIn(page, fixture.admin);
  const stale = await stage(page);
  const updated = await page.request.patch(`${fixture.issuer}/api/users/m05-recipient`, { headers: bearer(), data: { display_name: 'Changed after staging' } });
  expect(updated.ok()).toBe(true);
  await page.getByRole('button', { name: 'Refresh change' }).click();
  await expect(page.locator('#grant-status')).toHaveText('Stale');
  await expect(page.getByRole('button', { name: 'Approve exact change' })).toBeDisabled();
  await page.getByRole('button', { name: 'Cancel change' }).click();
  await expect(page.locator('#grant-status')).toHaveText('Cancelled');
  await expect(page.getByRole('button', { name: 'Cancel change' })).toBeDisabled();

  const denied = await stage(page);
  const context = await browser.newContext();
  try {
    const review = await context.newPage();
    await signIn(review, reviewer); await open(review, denied); await acknowledge(review);
    const secret = 'private-database-detail <img src=x onerror=alert(1)>';
    await review.route(`**/api/admin/delegated-grant-changes/${denied}/approve`, (route) => route.fulfill({
      status: 403, contentType: 'application/json', body: JSON.stringify({ error: 'access_denied', error_description: secret }),
    }));
    await review.getByRole('button', { name: 'Approve exact change' }).click();
    await expect(review.getByRole('alert')).toContainText('currently authorized administrators');
    await expect(review.locator('#grant-status')).toHaveText('Stale');
    expect(await review.content()).not.toContain(secret);
    await expect(review.getByRole('button', { name: 'Execute once' })).toBeDisabled();
    await review.unrouteAll({ behavior: 'wait' });
    await open(review, stale);
    await expect(review.locator('#grant-status')).toHaveText('Cancelled');

    await open(review, denied);
    await expect(review.locator('#grant-status')).toHaveText('Awaiting review');
    await acknowledge(review);
    // A new account in another tab must not inherit the checked action.
    let approvals = 0;
    review.on('request', (request) => { if (request.method() === 'POST' && request.url().endsWith(`/${denied}/approve`)) approvals += 1; });
    await context.clearCookies();
    const otherTab = await context.newPage();
    await signIn(otherTab, executor);
    await review.evaluate(() => [...document.querySelectorAll('button')].find((b) => b.textContent === 'Approve exact change').click());
    await expect(review.locator('#account-detail')).toContainText('m05-executor');
    await expect(review.locator('#grant-status')).toHaveText('Awaiting review');
    expect(approvals).toBe(0);
    await expect(review.getByLabel('I checked the exact grants, digest and dependencies.')).not.toBeChecked();
    await otherTab.close();
    await review.clock.install();
    await review.clock.fastForward(16 * 60 * 1000);
    await expect(review.locator('#grant-status')).toHaveText('Expired');
    await expect(review.getByRole('button', { name: 'Approve exact change' })).toBeDisabled();
    await expect(review.getByRole('button', { name: 'Cancel change' })).toBeDisabled();
    await context.clearCookies();
    await review.getByRole('button', { name: 'Refresh', exact: true }).click();
    await expect(review.locator('#gate-title')).toHaveText('Sign in to administer riAuth');
    await expect(review.locator('#grant-digest')).toHaveCount(0);
  } finally { await context.close(); }
});
