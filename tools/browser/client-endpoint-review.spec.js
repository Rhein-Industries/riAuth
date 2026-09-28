import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { fixtureStartupMs, startFixture } from './fixture.js';

let fixture, stopFixture;
const reviewer = { username: 'endpoint-reviewer', password: 'reviewer fixture password 2026' };
const executor = { username: 'endpoint-executor', password: 'executor fixture password 2026' };
const bearer = () => ({ authorization: `Bearer ${fixture.token}` });
test.beforeAll(async ({ request }) => {
  test.setTimeout(fixtureStartupMs + 20000);
  ({ fixture, stop: stopFixture } = await startFixture());
  for (const user of [reviewer, executor]) {
    expect((await request.post(`${fixture.issuer}/api/users`, { headers: bearer(), data: { admin: true, ...user } })).ok()).toBe(true);
  }
  for (const id of ['exact', 'stale', 'session', 'capability']) {
    expect((await request.post(`${fixture.issuer}/api/clients`, { headers: bearer(), data: {
      client_id: `endpoint-${id}`, name: `Endpoints ${id}`, confidential: true,
      redirect_uris: ['https://endpoint.example.test/callback'], scopes: ['openid', 'profile'],
    } })).ok()).toBe(true);
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
async function open(page, id) {
  await page.goto(`${fixture.issuer}/admin#/client-endpoint-review/${id}`);
  await expect(page.locator('#endpoint-digest')).toBeVisible();
}
async function draft(page, client, clear = false) {
  await page.goto(`${fixture.issuer}/admin#/applications/${client}`);
  // Existing-client Save changes must not offer a second redirect or origin writer.
  await expect(page.locator('#app-redirects')).toHaveCount(0);
  await expect(page.locator('#app-origins')).toHaveCount(0);
  await page.getByRole('link', { name: 'Review redirects and origins', exact: true }).click();
  await expect(page.getByLabel('Application', { exact: true })).toHaveValue(client);
  await page.getByLabel('Redirect URIs', { exact: true }).fill(clear ? '' : 'https://new.example.test/callback?tenant=one\nhttps://backup.example.test/cb');
  await page.getByLabel('Browser origins', { exact: true }).fill(clear ? '' : 'https://new.example.test');
  await page.getByLabel('I checked every redirect URI and browser origin, including removals.').check();
}
async function stage(page, client, clear = false) {
  await draft(page, client, clear);
  await page.getByRole('button', { name: 'Stage exact endpoints', exact: true }).click();
  await expect(page.locator('#endpoint-status')).toHaveText('Awaiting review');
  return page.locator('#endpoint-id').innerText();
}
const acknowledge = (page) => page.getByLabel('I checked the redirect URIs and origins, revocation effects, digest and dependencies.').check();
const requestShape = (request) => ({ body: request.postDataJSON(), key: request.headers()['idempotency-key'], revision: request.headers()['if-match'] });
async function rotateSession(context, user) {
  const other = await context.newPage();
  await other.goto(`${fixture.issuer}/admin`);
  await other.locator('#sign-out').click();
  await expect(other.locator('#gate-title')).toHaveText('Sign in to administer riAuth');
  await signIn(other, user);
  return other;
}

test('exact endpoint review recovers bound requests once and preserves ordinary application edits', async ({ page, browser }) => {
  test.setTimeout(90000);
  await signIn(page, fixture.admin);
  await page.goto(`${fixture.issuer}/admin#/applications/endpoint-exact`);
  await page.getByLabel('Name', { exact: true }).fill('Renamed without status changes');
  const write = page.waitForRequest((r) => r.method() === 'PATCH' && r.url().endsWith('/clients/endpoint-exact'));
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  expect((await write).postDataJSON()).toEqual({ name: 'Renamed without status changes' });
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Renamed without status changes');
  await draft(page, 'endpoint-exact');
  const attempts = [];
  let change, stagedReady, releaseStage;
  const staged = new Promise((resolve) => { stagedReady = resolve; });
  const lateResponse = new Promise((resolve) => { releaseStage = resolve; });
  await page.route('**/api/admin/clients/endpoint-exact/endpoint-changes', async (route) => {
    attempts.push(requestShape(route.request()));
    if (attempts.length === 1) {
      const response = await route.fetch(); expect(response.ok()).toBe(true);
      change = await response.json(); stagedReady(); await lateResponse;
      await route.fulfill({ status: 403, json: { error_description: 'Retired staging response' } });
    } else await route.continue();
  });
  await page.getByRole('button', { name: 'Stage exact endpoints', exact: true }).click();
  await staged;
  await page.getByRole('button', { name: 'Refresh', exact: true }).click();
  await expect(page.getByLabel('I checked every redirect URI and browser origin, including removals.')).not.toBeChecked();
  // A retired view's late error must not clear the retained request key or let
  // the replacement view submit a fresh proposal for an uncertain stage.
  const retired = page.waitForResponse((r) => r.url().endsWith('/clients/endpoint-exact/endpoint-changes') && r.status() === 403);
  releaseStage(); await retired;
  await expect(page.getByRole('alert')).toBeHidden();
  await expect(page.getByLabel('Redirect URIs', { exact: true })).toBeDisabled();
  await page.getByRole('button', { name: 'Retry same staging request' }).click();
  await expect(page.locator('#endpoint-status')).toHaveText('Awaiting review');
  expect(attempts).toHaveLength(2); expect(attempts[1]).toEqual(attempts[0]);
  expect(attempts[0].body).toEqual({ redirect_uris: ['https://new.example.test/callback?tenant=one', 'https://backup.example.test/cb'], origins: ['https://new.example.test'] });
  const id = change.proposal.id;
  await expect(page.locator('#endpoint-before')).toBeVisible();
  await expect(page.locator('#endpoint-after')).toBeVisible();
  await expect(page.locator('#endpoint-effects')).toBeVisible();
  expect(JSON.parse(await page.locator('#endpoint-effects').innerText())).toEqual(change.proposal.effects);
  await expect(page.getByText('Token families to revoke', { exact: true })).toBeVisible();
  expect(JSON.parse(await page.locator('#endpoint-before').innerText())).toEqual(change.proposal.before);
  expect(JSON.parse(await page.locator('#endpoint-after').innerText())).toEqual(change.proposal.after);
  await expect(page.locator('#endpoint-digest')).toHaveText(change.digest);
  await expect(page.getByText(change.proposal.resource_revision, { exact: true })).toBeVisible();
  await expect(page.getByText(change.proposal.policy_revision, { exact: true })).toBeVisible();
  await expect(page.getByLabel('Review link')).toHaveValue(`${fixture.issuer}/admin#/client-endpoint-review/${id}`);
  await acknowledge(page);
  await expect(page.getByRole('button', { name: 'Approve exact endpoints' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Apply endpoints once' })).toBeDisabled();
  const accessibility = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  expect(accessibility.violations.map(({ id }) => id)).toEqual([]);
  await page.screenshot({ path: '/tmp/riauth-m05-endpoint-review-desktop.png', fullPage: true });
  await page.setViewportSize({ width: 320, height: 900 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: '/tmp/riauth-m05-endpoint-review-mobile.png', fullPage: true });

  const reviewContext = await browser.newContext(), executeContext = await browser.newContext();
  try {
    const review = await reviewContext.newPage(), execute = await executeContext.newPage();
    await signIn(review, reviewer); await signIn(execute, executor);
    await open(review, id); await acknowledge(review);
    const approvals = [];
    await review.route(`**/api/admin/client-endpoint-changes/${id}/approve`, async (route) => {
      approvals.push(requestShape(route.request()));
      const response = await route.fetch(); expect(response.ok()).toBe(true);
      if (approvals.length === 1) {
        // A success response with altered content must not certify the original approval.
        const altered = await response.json(); altered.proposal.after.redirect_uris[0] = 'https://substituted.example/callback';
        await route.fulfill({ response, json: altered });
      } else await route.fulfill({ response });
    });
    await review.getByRole('button', { name: 'Approve exact endpoints' }).click();
    await expect(review.locator('#endpoint-status')).toHaveText('Outcome unknown');
    await expect(review.getByRole('button', { name: 'Apply endpoints once' })).toBeDisabled();
    await review.getByRole('button', { name: 'Recover same request' }).click();
    await expect(review.locator('#endpoint-status')).toHaveText('Approved');
    expect(approvals).toHaveLength(2); expect(approvals[1]).toEqual(approvals[0]);
    expect(approvals[0].body).toEqual({ digest: change.digest });
    await acknowledge(review);
    await expect(review.getByRole('button', { name: 'Apply endpoints once' })).toBeDisabled();
    await open(execute, id); await acknowledge(execute);
    const executions = [];
    await execute.route(`**/api/admin/client-endpoint-changes/${id}/execute`, async (route) => {
      executions.push(requestShape(route.request()));
      if (executions.length === 1) { const response = await route.fetch(); expect(response.ok()).toBe(true); await route.abort('failed'); }
      else await route.continue();
    });
    await execute.getByRole('button', { name: 'Apply endpoints once' }).click();
    await expect(execute.locator('#endpoint-status')).toHaveText('Outcome unknown');
    await execute.getByRole('button', { name: 'Refresh change' }).click();
    await expect(execute.locator('#endpoint-status')).toHaveText('Outcome unknown');
    await execute.getByRole('button', { name: 'Recover same request' }).click();
    await expect(execute.locator('#endpoint-status')).toHaveText('Executed');
    expect(executions).toHaveLength(2); expect(executions[1]).toEqual(executions[0]);
    expect(executions[0].body).toEqual({ digest: change.digest });
    await expect(execute.locator('#secret-value')).toHaveValue('');
    expect(await execute.content()).not.toContain('ri_client_');
    const replay = await execute.request.post(`${fixture.issuer}/api/admin/client-endpoint-changes/${id}/execute`, {
      headers: { 'x-riauth-portal': '1', origin: new URL(fixture.issuer).origin }, data: { digest: change.digest },
    });
    expect(replay.status()).toBe(409);
    const events = await (await page.request.get(`${fixture.issuer}/api/audit?limit=100`, { headers: bearer() })).json();
    expect(events.filter((e) => e.action === 'reviewed_client_endpoints.execute' && e.details.change_id === id)).toHaveLength(1);
    const clients = await (await page.request.get(`${fixture.issuer}/api/clients`, { headers: bearer() })).json();
    expect(clients.find((c) => c.client_id === 'endpoint-exact')).toMatchObject({ name: 'Renamed without status changes', redirect_uris: ['https://new.example.test/callback?tenant=one', 'https://backup.example.test/cb'] });
    // Removing every callback and origin follows the same review path.
    const enable = await stage(page, 'endpoint-exact', true);
    expect(JSON.parse(await page.locator('#endpoint-after').innerText())).toEqual({ redirect_uris: [], origins: [] });
    await open(review, enable); await acknowledge(review); await review.getByRole('button', { name: 'Approve exact endpoints' }).click();
    await expect(review.locator('#endpoint-status')).toHaveText('Approved');
    await open(execute, enable); await acknowledge(execute); await execute.getByRole('button', { name: 'Apply endpoints once' }).click();
    await expect(execute.locator('#endpoint-status')).toHaveText('Executed');
  } finally { await reviewContext.close(); await executeContext.close(); }
});

test('draft revisions, credential rotation, expiry and safe authorization errors keep stale reviews closed', async ({ page, browser }) => {
  await signIn(page, fixture.admin); await draft(page, 'endpoint-stale');
  const revision = await page.locator('#endpoint-draft-revision').innerText();
  expect((await page.request.patch(`${fixture.issuer}/api/clients/endpoint-stale`, { headers: bearer(), data: { name: 'Changed during draft' } })).ok()).toBe(true);
  await page.getByRole('button', { name: 'Refresh', exact: true }).click();
  await expect(page.getByLabel('I checked every redirect URI and browser origin, including removals.')).not.toBeChecked();
  await expect(page.locator('#endpoint-draft-revision')).toHaveText(revision);
  await expect(page.getByLabel('Redirect URIs', { exact: true })).toHaveValue('https://new.example.test/callback?tenant=one\nhttps://backup.example.test/cb');
  await page.getByLabel('I checked every redirect URI and browser origin, including removals.').check();
  await page.getByRole('button', { name: 'Stage exact endpoints', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('server refused');
  await page.getByRole('button', { name: 'Load current endpoints' }).click();
  await expect(page.getByLabel('Redirect URIs', { exact: true })).toHaveValue('https://endpoint.example.test/callback');
  await page.getByLabel('Redirect URIs', { exact: true }).fill('https://new.example.test/callback?tenant=one\nhttps://backup.example.test/cb');
  await page.getByLabel('Browser origins', { exact: true }).fill('https://new.example.test');
  await page.getByLabel('I checked every redirect URI and browser origin, including removals.').check();
  await page.getByRole('button', { name: 'Stage exact endpoints', exact: true }).click();
  await expect(page.locator('#endpoint-status')).toHaveText('Awaiting review');
  const id = await page.locator('#endpoint-id').innerText();
  const context = await browser.newContext();
  try {
    const review = await context.newPage(); await signIn(review, reviewer); await open(review, id); await acknowledge(review);
    await review.getByRole('button', { name: 'Approve exact endpoints' }).click();
    await expect(review.locator('#endpoint-status')).toHaveText('Approved');
    // The ordinary rotation control remains immediate and stales this exact dependency.
    await page.goto(`${fixture.issuer}/admin#/applications/endpoint-stale`);
    await page.bringToFront();
    await page.getByRole('button', { name: 'Rotate secret', exact: true }).click();
    await page.locator('#confirm-ok').click();
    await expect(page.locator('#secret-value')).toHaveValue(/^ri_client_/);
    const secret = await page.locator('#secret-value').inputValue();
    await page.locator('#secret-close').click();
    await open(page, id);
    await expect(page.locator('#endpoint-status')).toHaveText('Stale');
    await expect(page.locator('#secret-value')).toHaveValue('');
    expect(await page.content()).not.toContain(secret);
    await page.getByRole('button', { name: 'Cancel change' }).click();
    await expect(page.locator('#endpoint-status')).toHaveText('Cancelled');
    const denied = await stage(page, 'endpoint-stale');
    await open(review, denied); await acknowledge(review);
    await review.route(`**/api/admin/client-endpoint-changes/${denied}/approve`, (route) => route.fulfill({
      status: 403, contentType: 'application/json', body: JSON.stringify({ error_description: '<img src=x onerror=alert(1)> private server detail' }),
    }));
    await review.getByRole('button', { name: 'Approve exact endpoints' }).click();
    await expect(review.getByRole('alert')).toContainText('currently authorized full administrators');
    await expect(review.locator('#endpoint-status')).toHaveText('Stale');
    expect(await review.content()).not.toContain('private server detail');
    await page.clock.install(); await page.clock.fastForward(16 * 60 * 1000);
    await expect(page.locator('#endpoint-status')).toHaveText('Expired');
    await expect(page.getByRole('button', { name: 'Apply endpoints once' })).toBeDisabled();
  } finally { await context.close(); }
});

test('same-account session changes discard drafts and uncertain decisions before another action', async ({ page, browser }) => {
  await signIn(page, fixture.admin); await draft(page, 'endpoint-session');
  const other = await rotateSession(page.context(), fixture.admin);
  await page.evaluate(() => document.querySelector('#refresh').click());
  await expect(page.getByLabel('Redirect URIs', { exact: true })).toHaveValue('https://endpoint.example.test/callback');
  await expect(page.getByLabel('I checked every redirect URI and browser origin, including removals.')).not.toBeChecked();
  await other.close();
  const id = await stage(page, 'endpoint-session');
  const context = await browser.newContext();
  try {
    const review = await context.newPage(); await signIn(review, reviewer); await open(review, id); await acknowledge(review);
    let approvals = 0;
    await review.route(`**/api/admin/client-endpoint-changes/${id}/approve`, async (route) => {
      approvals += 1; const response = await route.fetch(); expect(response.ok()).toBe(true); await route.abort('failed');
    });
    await review.getByRole('button', { name: 'Approve exact endpoints' }).click();
    await expect(review.locator('#endpoint-status')).toHaveText('Outcome unknown');
    const otherReview = await rotateSession(context, reviewer);
    await review.evaluate(() => [...document.querySelectorAll('button')].find((b) => b.textContent === 'Recover same request')?.click());
    await expect(review.locator('#endpoint-status')).toHaveText('Approved');
    await expect(review.getByRole('button', { name: 'Recover same request' })).toBeHidden();
    expect(approvals).toBe(1);
    await expect(review.getByLabel('I checked the redirect URIs and origins, revocation effects, digest and dependencies.')).not.toBeChecked();
    await otherReview.close();
    await context.clearCookies();
    await review.getByRole('button', { name: 'Refresh change' }).click();
    await expect(review.locator('#gate-title')).toHaveText('Sign in to administer riAuth');
    await expect(review.locator('#endpoint-digest')).toHaveCount(0);
  } finally { await context.close(); }
});

test('unavailable provider capabilities prevent staging and malformed proposals cannot enable actions', async ({ page, request }) => {
  await signIn(page, fixture.admin);
  await page.route('**/api/admin/clients', async (route) => {
    const response = await route.fetch(), clients = await response.json();
    clients.find((c) => c.client_id === 'endpoint-capability').settings.saml = {};
    await route.fulfill({ response, json: clients });
  });
  await page.route('**/api/capabilities', async (route) => {
    const response = await route.fetch(), capabilities = await response.json();
    capabilities.feature_states['saml.idp_signed_browser_sso'].compiled = false;
    capabilities.feature_states['saml.idp_signed_browser_sso'].usable = false;
    await route.fulfill({ response, json: capabilities });
  });
  await page.goto(`${fixture.issuer}/admin#/applications/endpoint-capability`);
  await expect(page.getByText("Redirect and origin review is unavailable for this application's current provider or capabilities.")).toBeVisible();
  await expect(page.getByRole('link', { name: 'Review redirects and origins', exact: true })).toHaveCount(0);
  await page.goto(`${fixture.issuer}/admin#/client-endpoint-review/client%3Aendpoint-capability`);
  await expect(page.getByRole('button', { name: 'Load current endpoints' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Stage exact endpoints', exact: true })).toHaveCount(0);
  await page.unrouteAll({ behavior: 'wait' });
  const staged = await request.post(`${fixture.issuer}/api/clients/endpoint-capability/endpoint-changes`, { headers: bearer(), data: { redirect_uris: ['https://new.example.test/callback?tenant=one', 'https://backup.example.test/cb'], origins: ['https://new.example.test'] } });
  expect(staged.ok()).toBe(true); const change = await staged.json(), id = change.proposal.id;
  await page.route(`**/api/admin/client-endpoint-changes/${id}`, async (route) => {
    const altered = structuredClone(change); altered.proposal.after.enabled = false;
    await route.fulfill({ json: altered });
  });
  await page.goto(`${fixture.issuer}/admin#/client-endpoint-review/${id}`);
  await expect(page.getByRole('alert')).toContainText('response was lost or incomplete');
  await expect(page.getByRole('button', { name: 'Approve exact endpoints' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Apply endpoints once' })).toHaveCount(0);
});
