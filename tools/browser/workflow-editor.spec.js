import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';

const html = readFileSync(new URL('../../src/portal/admin.html', import.meta.url), 'utf8').replaceAll('__BASE__', '/');
// Load the page's current dependencies so new review modules cannot silently leave
// this fixture at its loading screen. The instance capability response stays mocked.
const scripts = new Map([...html.matchAll(/<script src="\/portal\/assets\/([^"]+)"/g)].map(([, name]) =>
  [name, readFileSync(new URL(`../../src/portal/${name}`, import.meta.url), 'utf8')]));
const origin = 'http://127.0.0.1:9876';

async function mountAdmin(page, { workflows = [], groups = [], plan, apply } = {}) {
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.route(`${origin}/**`, async (route) => {
    const path = new URL(route.request().url()).pathname;
    const json = (value) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(value) });
    if (path === '/admin') return route.fulfill({ status: 200, contentType: 'text/html', body: html });
    if (path === '/portal/assets/capabilities.js') return route.fulfill({ status: 200, contentType: 'text/javascript', body:
      'window.RiAuthCapabilities={refresh:async()=>{},compiled:()=>false,usable:()=>false,apply:()=>{}};' });
    if (scripts.has(path.split('/').at(-1))) return route.fulfill({ status: 200, contentType: 'text/javascript', body: scripts.get(path.split('/').at(-1)) });
    if (path.startsWith('/portal/assets/')) return route.fulfill({ status: 200, body: '' });
    if (path === '/api/admin/session') return json({ user: { id: 'admin-id', username: 'admin', display_name: 'Admin', admin: true }, session_marker: 'session-one', edition: 'platform', revision: 0 });
    if (path === '/api/admin/workflows/plan') return json(await plan(route.request().postDataJSON()));
    if (path === '/api/admin/workflows/apply') return json(await apply(route.request().postDataJSON()));
    if (path === '/api/admin/workflows') return json(workflows);
    if (path === '/api/admin/groups') return json(groups);
    if (path === '/api/admin/invitations') return json({ invitations: [], delivery_configured: false, lifetime: 0 });
    if (path === '/api/admin/provisioning/deactivations') return json([]);
    if (['/api/admin/clients', '/api/admin/users', '/api/admin/audit'].includes(path)) return json([]);
    return route.fulfill({ status: 404, contentType: 'application/json', body: '{}' });
  });
  return errors;
}

for (const operation of ['plan', 'apply']) {
  test(`a rejected workflow ${operation} leaves its button usable for a corrected retry`, async ({ page }) => {
    const workflows = [];
    const errors = await mountAdmin(page, {
      workflows,
      plan: async (definition) => ({ plan_id: 'retry-plan', manifest: { workflows: [definition] }, changes: [{}] }),
      apply: async (input) => {
        workflows.push(input.plan.manifest.workflows[0]);
        return { applied: true };
      },
    });
    let attempts = 0;
    await page.route(`${origin}/api/admin/workflows/${operation}`, (route) => {
      if (++attempts > 1) return route.fallback();
      return route.fulfill({ status: 422, contentType: 'application/json',
        body: JSON.stringify({ error: 'invalid', error_description: 'Fixture validation rejected this request.' }) });
    });
    await page.goto(`${origin}/admin#/workflows/new`);
    await page.getByRole('button', { name: 'Start authentication workflow template' }).click();
    if (operation === 'apply') await page.getByRole('button', { name: 'Validate and preview' }).click();
    const button = page.getByRole('button', { name: operation === 'plan' ? 'Validate and preview' : 'Save definition' });
    await button.click();
    await expect(page.getByRole('alert')).toHaveText('Fixture validation rejected this request.');
    await expect(button).toBeEnabled();
    expect(errors).toEqual([]);
    await button.click();
    if (operation === 'plan') await expect(page.getByRole('button', { name: 'Save definition' })).toBeVisible();
    else await expect(page.locator('#toast')).toContainText('Saved workflow platform-authentication revision 1.');
    expect(attempts).toBe(2);
    expect(errors).toEqual([]);
  });
}

test('finishing a save refresh preserves navigation to another administration page', async ({ page }) => {
  const workflows = [];
  const errors = await mountAdmin(page, {
    workflows,
    plan: async (definition) => ({ plan_id: 'navigation-plan', manifest: { workflows: [definition] }, changes: [{}] }),
    apply: async (input) => { workflows.push(input.plan.manifest.workflows[0]); return { applied: true }; },
  });
  let reads = 0, refreshStarted, releaseRefresh;
  const started = new Promise((resolve) => { refreshStarted = resolve; });
  const release = new Promise((resolve) => { releaseRefresh = resolve; });
  await page.route(`${origin}/api/admin/session`, async (route) => {
    if (++reads === 2) { refreshStarted(); await release; }
    return route.fallback();
  });
  await page.goto(`${origin}/admin#/workflows/new`);
  await page.getByRole('button', { name: 'Start authentication workflow template' }).click();
  await page.getByRole('button', { name: 'Validate and preview' }).click();
  await page.getByRole('button', { name: 'Save definition' }).click();
  await started;
  await page.locator('[data-section="people"]').click();
  await expect(page.getByRole('heading', { name: 'People', exact: true, level: 1 })).toBeVisible();
  releaseRefresh();
  await expect(page.locator('#toast')).toHaveText('Saved workflow platform-authentication revision 1.');
  await expect(page).toHaveURL(`${origin}/admin#/people`);
  await expect(page.getByRole('heading', { name: 'People', exact: true, level: 1 })).toBeVisible();
  expect(errors).toEqual([]);
});

for (const [kind, navigate] of [['group', true], ['person', true], ['group', false]]) {
  test(`pending ${kind} creation ${navigate ? 'preserves later navigation' : 'opens the created record when the form stays open'}`, async ({ page }) => {
    const groups = [];
    const errors = await mountAdmin(page, { groups });
    let submitted, releaseWrite;
    const started = new Promise((resolve) => { submitted = resolve; });
    const release = new Promise((resolve) => { releaseWrite = resolve; });
    const endpoint = kind === 'group' ? 'groups' : 'users';
    await page.route(`${origin}/api/admin/${endpoint}`, async (route) => {
      if (route.request().method() !== 'POST') return route.fallback();
      submitted(); await release;
      if (kind === 'group') groups.push({ name: 'engineering', members: [] });
      return route.fulfill({ status: 200, contentType: 'application/json', body: '{}' });
    });
    await page.goto(`${origin}/admin#/${kind === 'group' ? 'groups' : 'people/new'}`);
    if (kind === 'group') await page.getByLabel('New group').fill('engineering');
    else {
      await page.getByLabel('Username', { exact: true }).fill('new-person');
      await page.getByLabel('Initial password', { exact: true }).fill('synthetic browser password');
    }
    await page.getByRole('button', { name: `Create ${kind}`, exact: true }).click();
    await started;
    if (navigate) {
      await page.locator('[data-section="people"]').click();
      await expect(page.getByRole('heading', { name: 'People', exact: true, level: 1 })).toBeVisible();
    }
    releaseWrite();
    await expect(page.locator('#toast')).toHaveText(`Created ${kind === 'group' ? 'engineering' : 'new-person'}.`);
    await expect(page).toHaveURL(`${origin}/admin#/${navigate ? 'people' : 'groups/engineering'}`);
    await expect(page.getByRole('heading', { name: navigate ? 'People' : 'engineering', exact: true, level: 1 })).toBeVisible();
    expect(errors).toEqual([]);
  });
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

test('consent and sensitive-action definitions edit through the canonical plan and apply path', async ({ page }) => {
  const workflows = ['consent', 'sensitive_action'].map((category) => {
    const consent = category === 'consent';
    const second = consent ? 'consent' : 'password';
    return {
      format: 'riauth.workflow/v1', id: `platform-${category}`, revision: 3,
      category, origin: 'configured', entry: 'session', limits: { max_duration_seconds: 900, max_executions: 5 },
      steps: [
        { id: 'session', action: { type: 'resume_session' }, max_attempts: 1, timeout_seconds: 60, cancellable: true,
          transitions: [{ on: 'verified', to: second }, { on: 'failed', to: 'reject' }] },
        { id: second, action: { type: consent ? 'request_consent' : 'verify_password' }, max_attempts: 1, timeout_seconds: 300, cancellable: true,
          transitions: [{ on: consent ? 'granted' : 'verified', to: 'allow' }, { on: consent ? 'denied' : 'failed', to: 'reject' }] },
      ],
      terminals: [{ id: 'allow', outcome: consent ? 'consent_granted' : 'action_authorized', requires: [],
        ...(consent ? {} : { max_proof_age_seconds: 300 }) }, { id: 'reject', outcome: 'denied', requires: [] }],
    };
  });
  const applied = [];
  const errors = await mountAdmin(page, { workflows,
    plan: async (definition) => ({ plan_id: `plan-${definition.category}`, manifest: { workflows: [definition] }, changes: [{}] }),
    apply: async (input) => {
      const definition = input.plan.manifest.workflows[0];
      applied.push(definition);
      workflows[workflows.findIndex((row) => row.id === definition.id)] = definition;
      return { applied: true };
    },
  });
  for (const [index, definition] of workflows.entries()) {
    await page.goto(`${origin}/admin#/workflows/${definition.id}`);
    await expect(page.getByRole('heading', { level: 1, name: `Edit ${definition.id}` })).toBeVisible();
    await expect(page.getByText('Saving a definition does not activate it or make unsupported shapes executable.', { exact: false })).toBeVisible();
    await expect(page.getByText('The server selects only supported shapes from its runtime configuration.', { exact: false })).toBeVisible();
    await page.locator('.workflow-graph button').nth(1).click();
    const action = page.locator('.workflow-controls select').first();
    await action.selectOption(definition.category === 'consent' ? 'resume_session' : 'verify_passkey');
    let draft = JSON.parse(await page.locator('.settings-json').textContent());
    expect(draft.steps[1].transitions).toEqual([{ on: 'verified', to: 'allow' }, { on: 'failed', to: 'reject' }]);
    await action.selectOption(definition.steps[1].action.type);
    draft = JSON.parse(await page.locator('.settings-json').textContent());
    expect(draft.steps[1].transitions).toEqual(definition.steps[1].transitions);
    await page.locator('.workflow-controls input[type="number"]').first().fill('2');
    await page.locator('.workflow-controls input[type="number"]').first().press('Tab');
    await page.getByRole('button', { name: 'Validate and preview' }).click();
    await expect(page.getByRole('button', { name: 'Save definition' })).toBeVisible();
    await page.getByRole('button', { name: 'Save definition' }).click();
    await expect.poll(() => applied.length).toBe(index + 1);
    expect(applied[index]).toMatchObject({ category: definition.category, revision: 4,
      steps: [{ action: { type: 'resume_session' } }, { action: definition.steps[1].action, max_attempts: 2,
        transitions: definition.steps[1].transitions }], terminals: definition.terminals });
  }
  expect(errors).toEqual([]);
});

test('consent and sensitive-action starters show canonical proof paths before save', async ({ page }) => {
  const submitted = [];
  const errors = await mountAdmin(page, { workflows: [],
    plan: async (definition) => {
      submitted.push(definition);
      return { plan_id: `plan-${definition.category}`, manifest: { workflows: [definition] }, changes: [{}] };
    },
  });
  for (const category of ['consent', 'sensitive_action']) {
    await page.goto(`${origin}/admin#/workflows/new`);
    await page.getByRole('button', { name: `Start ${category} workflow template` }).click();
    await expect(page.getByText('The graph and validation plan are static previews; they do not execute credentials.', { exact: false })).toBeVisible();
    const definition = JSON.parse(await page.locator('.settings-json').textContent());
    expect(definition).toMatchObject({ format: 'riauth.workflow/v1', category, origin: 'configured', revision: 1,
      entry: 'session', terminals: [{ outcome: category === 'consent' ? 'consent_granted' : 'action_authorized' }, { outcome: 'denied' }] });
    expect(definition.steps.map((step) => step.action.type)).toEqual(category === 'consent'
      ? ['resume_session', 'request_consent'] : ['resume_session', 'verify_password', 'verify_totp']);
    if (category === 'consent') {
      expect(definition.steps[1].transitions).toEqual([{ on: 'granted', to: 'success' }, { on: 'denied', to: 'denied' }]);
      expect(definition.terminals[0]).not.toHaveProperty('max_proof_age_seconds');
    } else {
      expect(definition.steps[1].transitions[0]).toEqual({ on: 'verified', when: { type: 'account_has', credential: 'totp' }, to: 'totp' });
      expect(definition.terminals[0].max_proof_age_seconds).toBe(300);
    }
    await page.getByRole('button', { name: 'Validate and preview' }).click();
    await expect(page.getByRole('button', { name: 'Save definition' })).toBeVisible();
  }
  expect(submitted.map((definition) => definition.category)).toEqual(['consent', 'sensitive_action']);
  expect(errors).toEqual([]);
});
