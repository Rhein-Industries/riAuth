import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';

const html = readFileSync(new URL('../../src/portal/admin.html', import.meta.url), 'utf8').replaceAll('__BASE__', '/');
const script = readFileSync(new URL('../../src/portal/admin.js', import.meta.url), 'utf8');
const origin = 'http://127.0.0.1:9876';

async function mountAdmin(page, { workflows = [], plan, apply } = {}) {
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.route(`${origin}/**`, async (route) => {
    const path = new URL(route.request().url()).pathname;
    const json = (value) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(value) });
    if (path === '/admin') return route.fulfill({ status: 200, contentType: 'text/html', body: html });
    if (path === '/portal/assets/admin.js') return route.fulfill({ status: 200, contentType: 'text/javascript', body: script });
    if (path === '/portal/assets/capabilities.js') return route.fulfill({ status: 200, contentType: 'text/javascript', body:
      'window.RiAuthCapabilities={refresh:async()=>{},compiled:()=>false,usable:()=>false,apply:()=>{}};' });
    if (path.startsWith('/portal/assets/')) return route.fulfill({ status: 200, body: '' });
    if (path === '/api/admin/session') return json({ user: { id: 'admin-id', username: 'admin', display_name: 'Admin', admin: true }, edition: 'platform', revision: 0 });
    if (path === '/api/admin/workflows/plan') return json(await plan(route.request().postDataJSON()));
    if (path === '/api/admin/workflows/apply') return json(await apply(route.request().postDataJSON()));
    if (path === '/api/admin/workflows') return json(workflows);
    if (path === '/api/admin/invitations') return json({ invitations: [], delivery_configured: false, lifetime: 0 });
    if (['/api/admin/clients', '/api/admin/users', '/api/admin/groups', '/api/admin/audit'].includes(path)) return json([]);
    return route.fulfill({ status: 404, contentType: 'application/json', body: '{}' });
  });
  return errors;
}

test('late preview cannot attach an older plan to an edited draft or save it', async ({ page }) => {
  const workflows = [];
  let releaseFirst, firstSubmitted, applied;
  const firstStarted = new Promise((resolve) => { firstSubmitted = resolve; });
  const firstRelease = new Promise((resolve) => { releaseFirst = resolve; });
  const errors = await mountAdmin(page, {
    workflows,
    plan: async (definition) => {
      if (definition.steps[0].id === 'password') {
        firstSubmitted(definition);
        await firstRelease;
      }
      return { plan_id: `plan-${definition.steps[0].id}`, manifest: { workflows: [definition] }, changes: [{ resource: `workflow/${definition.id}` }] };
    },
    apply: async (input) => {
      applied = input;
      workflows.push(input.plan.manifest.workflows[0]);
      return { applied: true };
    },
  });
  await page.goto(`${origin}/admin#/workflows/new`);
  await page.getByRole('button', { name: 'Start authentication workflow template' }).click();
  await page.getByRole('button', { name: 'Validate and preview' }).click();
  const oldDraft = await firstStarted;
  expect(oldDraft.steps[0].id).toBe('password');

  // Editing remains possible while the first plan is pending.
  await page.locator('.workflow-controls input').first().fill('password-b');
  await page.locator('.workflow-controls input').first().press('Tab');
  await expect(page.locator('.workflow-node strong').first()).toHaveText('password-b');
  const lateResponse = page.waitForResponse((response) => response.url().endsWith('/api/admin/workflows/plan'));
  releaseFirst();
  await lateResponse;
  await expect(page.getByRole('button', { name: 'Save definition' })).toHaveCount(0);
  await expect(page.getByText('Server validation passed.')).toHaveCount(0);
  expect(applied).toBeUndefined();

  await page.getByRole('button', { name: 'Validate and preview' }).click();
  await expect(page.getByRole('button', { name: 'Save definition' })).toBeVisible();
  await page.getByRole('button', { name: 'Save definition' }).click();
  await expect.poll(() => applied?.plan.manifest.workflows[0].steps[0].id).toBe('password-b');
  await expect(page.locator('#toast')).toContainText('Saved workflow platform-authentication revision 1.');
  expect(errors).toEqual([]);
});

test('save reports the submitted revision when the draft changes during apply', async ({ page }) => {
  const workflows = [];
  let releaseApply, beginApply, applied;
  const applyStarted = new Promise((resolve) => { beginApply = resolve; });
  const applyRelease = new Promise((resolve) => { releaseApply = resolve; });
  const errors = await mountAdmin(page, {
    workflows,
    plan: async (definition) => ({ plan_id: 'plan-one', manifest: { workflows: [definition] }, changes: [{}] }),
    apply: async (input) => {
      applied = input;
      beginApply();
      await applyRelease;
      workflows.push(input.plan.manifest.workflows[0]);
      return { applied: true };
    },
  });
  await page.goto(`${origin}/admin#/workflows/new`);
  await page.getByRole('button', { name: 'Start authentication workflow template' }).click();
  await page.getByRole('button', { name: 'Validate and preview' }).click();
  await expect(page.getByRole('button', { name: 'Save definition' })).toBeVisible();
  await page.getByRole('button', { name: 'Save definition' }).click();
  await applyStarted;
  await page.locator('.workflow-controls input').first().fill('password-b');
  await page.locator('.workflow-controls input').first().press('Tab');
  releaseApply();
  await expect(page.locator('#toast')).toContainText('Saved workflow platform-authentication revision 1. The current draft has unsaved changes.');
  const current = JSON.parse(await page.locator('.settings-json').textContent());
  expect(current.steps[0].id).toBe('password-b');
  expect(current.revision).toBe(2);
  expect(applied.plan.manifest.workflows[0].steps[0].id).toBe('password');
  expect(errors).toEqual([]);
});

test('unsupported configured categories open in a safe read-only view', async ({ page }) => {
  const workflows = ['consent', 'sensitive_action'].map((category) => {
    const consent = category === 'consent';
    const second = consent ? 'consent' : 'password';
    return {
      format: 'riauth.workflow/v1', id: `platform-${category}`, revision: 3,
      category, origin: 'configured', entry: 'session', limits: { max_duration_seconds: 600, max_executions: 2 },
      steps: [
        { id: 'session', action: { type: 'resume_session' }, max_attempts: 1, timeout_seconds: 60, cancellable: true,
          transitions: [{ on: 'verified', to: second }, { on: 'failed', to: 'denied' }] },
        { id: second, action: { type: consent ? 'request_consent' : 'verify_password' }, max_attempts: 1, timeout_seconds: 300, cancellable: true,
          transitions: [{ on: consent ? 'granted' : 'verified', to: 'success' }, { on: consent ? 'denied' : 'failed', to: 'denied' }] },
      ],
      terminals: [{ id: 'success', outcome: consent ? 'consent_granted' : 'action_authorized', requires: [],
        ...(consent ? {} : { max_proof_age_seconds: 300 }) }, { id: 'denied', outcome: 'denied', requires: [] }],
    };
  });
  const errors = await mountAdmin(page, { workflows });
  for (const definition of workflows) {
    await page.goto(`${origin}/admin#/workflows/${definition.id}`);
    await expect(page.getByRole('heading', { level: 1, name: definition.id })).toBeVisible();
    await expect(page.getByText('This definition is read-only here.')).toBeVisible();
    await expect(page.locator('.settings-json')).toContainText(`"category": "${definition.category}"`);
    await expect(page.getByRole('button', { name: 'Save definition' })).toHaveCount(0);
  }
  expect(errors).toEqual([]);
});
