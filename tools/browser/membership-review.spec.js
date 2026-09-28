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
  for (const user of [reviewer, executor, { username: 'm05-member', password: 'recipient fixture password 2026', admin: false }]) {
    const response = await request.post(`${fixture.issuer}/api/users`, { headers: bearer(), data: { admin: true, ...user } });
    expect(response.ok()).toBe(true);
  }
  const group = await request.post(`${fixture.issuer}/api/groups`, { headers: bearer(), data: { name: 'm05-protected' } });
  expect(group.ok()).toBe(true);
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
  await page.goto(`${fixture.issuer}/admin#/membership-review${id ? `/${id}` : ''}`);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(id ? 'Review membership change' : 'Reviewed membership');
}
async function draft(page, revoke = false) {
  await page.goto(`${fixture.issuer}/admin#/groups/m05-protected`);
  await page.getByRole('link', { name: 'Review membership change' }).click();
  await expect(page.getByLabel('Group name')).toHaveValue('m05-protected');
  await page.getByRole('button', { name: 'Load current members' }).click();
  await expect(page.getByRole('heading', { name: 'Proposed members of m05-protected' })).toBeVisible();
  await page.getByLabel('Complete proposed usernames').fill(revoke ? '' : 'm05-member');
  await page.getByLabel('I checked the complete replacement, including every member being removed.').check();
}
async function stage(page, revoke = false) {
  await draft(page, revoke);
  await page.getByRole('button', { name: 'Stage exact change' }).click();
  await expect(page.locator('#membership-status')).toHaveText('Awaiting review');
  return page.locator('#membership-id').innerText();
}
async function acknowledge(page) {
  await page.getByLabel('I checked the exact members, digest and dependencies.').check();
}

test('exact membership review separates participants and recovers lost responses without replay', async ({ page, browser }) => {
  test.setTimeout(90000);
  await signIn(page, fixture.admin);
  await draft(page);
  const keys = [];
  let staged;
  await page.route('**/api/admin/groups/m05-protected/membership-changes', async (route) => {
    keys.push(route.request().headers()['idempotency-key']);
    if (keys.length === 1) {
      const response = await route.fetch();
      expect(response.ok()).toBe(true);
      staged = await response.json();
      await route.abort('failed');
    } else await route.continue();
  });
  await page.getByRole('button', { name: 'Stage exact change' }).click();
  await expect(page.getByRole('alert')).toContainText('response was lost or incomplete', { timeout: 30000 });
  await expect(page.getByLabel('Complete proposed usernames')).toBeDisabled();
  await page.getByRole('button', { name: 'Refresh', exact: true }).click();
  await expect(page.getByLabel('I checked the complete replacement, including every member being removed.')).not.toBeChecked();
  await expect(page.getByLabel('Complete proposed usernames')).toHaveValue('m05-member');
  await expect(page.getByLabel('Complete proposed usernames')).toBeDisabled();
  await page.getByRole('button', { name: 'Retry same staging request' }).click();
  await expect(page.locator('#membership-status')).toHaveText('Awaiting review');
  expect(keys).toHaveLength(2); expect(keys[0]).toBeTruthy(); expect(keys[1]).toBe(keys[0]);
  const id = await page.locator('#membership-id').innerText();
  expect(id).toBe(staged.proposal.id);
  const digest = await page.locator('#membership-digest').innerText();
  expect(digest).toBe(staged.digest);
  await expect(page.getByText(staged.proposal.resource_revision, { exact: true })).toBeVisible();
  await expect(page.getByText(staged.proposal.policy_revision, { exact: true })).toBeVisible();
  await expect(page.getByLabel('Review link')).toHaveValue(`${fixture.issuer}/admin#/membership-review/${id}`);
  await acknowledge(page);
  await page.reload();
  await expect(page.locator('#membership-digest')).toHaveText(digest);
  await expect(page.getByLabel('I checked the exact members, digest and dependencies.')).not.toBeChecked();
  await acknowledge(page);
  await expect(page.getByRole('button', { name: 'Approve exact change' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Execute once' })).toBeDisabled();
  const report = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  expect(report.violations.map(({ id }) => id)).toEqual([]);
  await page.setViewportSize({ width: 320, height: 900 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole('heading', { level: 1 }).focus();
  await page.evaluate(() => scrollTo(0, 0));
  await page.screenshot({ path: '/tmp/riauth-m05-membership-review-mobile.png', fullPage: true });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.evaluate(() => scrollTo(0, 0));
  await page.screenshot({ path: '/tmp/riauth-m05-membership-review-desktop.png', fullPage: true });

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
    await expect(review.locator('#membership-status')).toHaveText('Approved');
    await acknowledge(review);
    await expect(review.getByRole('button', { name: 'Execute once' })).toBeDisabled();
    await open(execute, id); await acknowledge(execute);
    await execute.route(`**/api/admin/group-membership-changes/${id}/execute`, async (route) => {
      expect(route.request().postDataJSON()).toEqual({ digest });
      const response = await route.fetch();
      expect(response.ok()).toBe(true);
      await route.abort('failed');
    }, { times: 1 });
    await execute.getByRole('button', { name: 'Execute once' }).click();
    await expect(execute.locator('#membership-status')).toHaveText('Outcome unknown');
    await expect(execute.getByRole('button', { name: 'Execute once' })).toBeDisabled();
    await execute.getByRole('button', { name: 'Refresh change' }).click();
    await expect(execute.locator('#membership-status')).toHaveText('Executed');
    const current = await (await page.request.get(`${fixture.issuer}/api/groups`, { headers: bearer() })).json();
    expect(current.find((g) => g.name === 'm05-protected').members).toEqual(staged.proposal.after.map((m) => m.user_id));
    const replay = await execute.request.post(`${fixture.issuer}/api/admin/group-membership-changes/${id}/execute`, {
      headers: { 'x-riauth-portal': '1', origin: new URL(fixture.issuer).origin }, data: { digest },
    });
    expect(replay.status()).toBe(409);
    const events = await (await page.request.get(`${fixture.issuer}/api/audit?limit=100`, { headers: bearer() })).json();
    expect(events.filter((e) => e.action === 'reviewed_memberships.execute' && e.details.change_id === id)).toHaveLength(1);

    const revoke = await stage(page, true);
    await expect(page.getByText('Execution will remove every durable member from this group.')).toBeVisible();
    await open(review, revoke); await acknowledge(review);
    await review.getByRole('button', { name: 'Approve exact change' }).click();
    await expect(review.locator('#membership-status')).toHaveText('Approved');
    await open(execute, revoke); await acknowledge(execute);
    await execute.getByRole('button', { name: 'Execute once' }).click();
    await expect(execute.locator('#membership-status')).toHaveText('Executed');
    const groups = await (await page.request.get(`${fixture.issuer}/api/groups`, { headers: bearer() })).json();
    expect(groups.find((g) => g.name === 'm05-protected').members).toEqual([]);
  } finally { await reviewerContext.close(); await executorContext.close(); }
});

test('stale, cancelled, expired and refused reviews stay closed and errors reveal no server detail', async ({ page, browser }) => {
  await signIn(page, fixture.admin);
  await draft(page);
  await page.getByRole('button', { name: 'Refresh', exact: true }).click();
  await expect(page.getByLabel('Complete proposed usernames')).toHaveValue('m05-member');
  await expect(page.getByLabel('I checked the complete replacement, including every member being removed.')).not.toBeChecked();
  await expect(page.getByRole('button', { name: 'Stage exact change' })).toBeDisabled();
  await page.getByLabel('I checked the complete replacement, including every member being removed.').check();
  await page.getByRole('button', { name: 'Stage exact change' }).click();
  await expect(page.locator('#membership-status')).toHaveText('Awaiting review');
  const stale = await page.locator('#membership-id').innerText();
  const updated = await page.request.patch(`${fixture.issuer}/api/users/m05-member`, { headers: bearer(), data: { display_name: 'Changed after staging' } });
  expect(updated.ok()).toBe(true);
  await page.getByRole('button', { name: 'Refresh change' }).click();
  await expect(page.locator('#membership-status')).toHaveText('Stale');
  await expect(page.getByRole('button', { name: 'Approve exact change' })).toBeDisabled();
  await page.getByRole('button', { name: 'Cancel change' }).click();
  await expect(page.locator('#membership-status')).toHaveText('Cancelled');
  await expect(page.getByRole('button', { name: 'Cancel change' })).toBeDisabled();

  const denied = await stage(page);
  const context = await browser.newContext();
  try {
    const review = await context.newPage();
    await signIn(review, reviewer);
    await draft(page);
    await page.getByLabel('Complete proposed usernames').fill('m05-reviewer');
    await page.getByLabel('I checked the complete replacement, including every member being removed.').check();
    await page.getByRole('button', { name: 'Stage exact change' }).click();
    await expect(page.locator('#membership-status')).toHaveText('Awaiting review');
    const affected = await page.locator('#membership-id').innerText();
    await open(review, affected); await acknowledge(review);
    await expect(review.getByText('Your membership changes in this proposal. You cannot author, review or execute it.')).toBeVisible();
    await expect(review.getByRole('button', { name: 'Approve exact change' })).toBeDisabled();
    await expect(review.getByRole('button', { name: 'Execute once' })).toBeDisabled();
    await page.getByRole('button', { name: 'Cancel change' }).click();
    await expect(page.locator('#membership-status')).toHaveText('Cancelled');
    await open(review, denied); await acknowledge(review);
    const secret = 'private-database-detail <img src=x onerror=alert(1)>';
    await review.route(`**/api/admin/group-membership-changes/${denied}/approve`, (route) => route.fulfill({
      status: 403, contentType: 'application/json', body: JSON.stringify({ error: 'access_denied', error_description: secret }),
    }));
    await review.getByRole('button', { name: 'Approve exact change' }).click();
    await expect(review.getByRole('alert')).toContainText('currently authorized administrators');
    await expect(review.locator('#membership-status')).toHaveText('Stale');
    expect(await review.content()).not.toContain(secret);
    await expect(review.getByRole('button', { name: 'Execute once' })).toBeDisabled();
    await review.unrouteAll({ behavior: 'wait' });
    await open(review, stale);
    await expect(review.locator('#membership-status')).toHaveText('Cancelled');

    await open(review, denied);
    await expect(review.locator('#membership-status')).toHaveText('Awaiting review');
    await acknowledge(review);
    // A new account in another tab must not inherit the checked action.
    let approvals = 0;
    review.on('request', (request) => { if (request.method() === 'POST' && request.url().endsWith(`/${denied}/approve`)) approvals += 1; });
    await context.clearCookies();
    const otherTab = await context.newPage();
    await signIn(otherTab, executor);
    await review.evaluate(() => [...document.querySelectorAll('button')].find((b) => b.textContent === 'Approve exact change').click());
    await expect(review.locator('#account-detail')).toContainText('m05-executor');
    await expect(review.locator('#membership-status')).toHaveText('Awaiting review');
    expect(approvals).toBe(0);
    await expect(review.getByLabel('I checked the exact members, digest and dependencies.')).not.toBeChecked();
    await otherTab.close();
    await review.clock.install();
    await review.clock.fastForward(16 * 60 * 1000);
    await expect(review.locator('#membership-status')).toHaveText('Expired');
    await expect(review.getByRole('button', { name: 'Approve exact change' })).toBeDisabled();
    await expect(review.getByRole('button', { name: 'Cancel change' })).toBeDisabled();
    await context.clearCookies();
    await review.getByRole('button', { name: 'Refresh', exact: true }).click();
    await expect(review.locator('#gate-title')).toHaveText('Sign in to administer riAuth');
    await expect(review.locator('#membership-digest')).toHaveCount(0);
  } finally { await context.close(); }
});
