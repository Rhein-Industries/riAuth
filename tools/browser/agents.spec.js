// Browser presentation checks against deterministic responses, including retained grants
// after self-service is disabled. Server authorization is covered by the Rust agent suites.
import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';

const origin = 'http://127.0.0.1:9876';
const portal = new URL('../../src/portal/', import.meta.url);
const html = readFileSync(new URL('self_service/agents.html', portal), 'utf8').replaceAll('__BASE__', '/');
const assets = new Map(['auth.js', 'capabilities.js', 'self_service/agents.js'].map((path) =>
  [path.split('/').at(-1), readFileSync(new URL(path, portal), 'utf8')]));

for (const allowed of [false, true]) {
  test(`existing application approvals stay visible and revocable with self-service ${allowed ? 'on' : 'off'}`, async ({ page }) => {
    const now = Math.floor(Date.now() / 1000), writes = [], errors = [];
    const approval = { id: 'approval-one', client_id: 'mail', scopes: ['mail.read'],
      created_at: now - 60, expires_at: now + 3600, revoked_at: null };
    page.on('pageerror', (error) => errors.push(error.message));
    await page.route(`${origin}/**`, (route) => {
      const path = new URL(route.request().url()).pathname;
      const json = (value) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(value) });
      if (path === '/account/agents') return route.fulfill({ status: 200, contentType: 'text/html', body: html });
      if (path.startsWith('/portal/assets/')) return route.fulfill({ status: 200,
        contentType: 'text/javascript', body: assets.get(path.split('/').at(-1)) || '' });
      if (path === '/api/capabilities') return json({ scope: 'instance', schema_version: 'riauth.capabilities/v2', feature_states: {} });
      if (path === '/api/portal/security') return json({ user: { id: 'owner-id', username: 'owner', display_name: 'Owner' },
        current_session_id: 'session-one', browser_owned: true, password_available: true });
      if (path === '/api/portal/agents') return json({ self_service: { allowed }, proposals: [], agents: [{
        id: 'helper', enabled: true, created_at: now - 120, expires_at: now + 7200,
        permissions: [{ action: 'profile.read', resource: 'self' }], effective_permissions: [{ action: 'profile.read', resource: 'self' }],
      }] });
      if (path === '/api/portal/changes') return json({ changes: [] });
      if (path === '/api/portal/agent-applications') return json([{ client_id: 'mail', name: 'Mail', scopes: ['mail.read'], resources: [] }]);
      if (path === '/api/portal/agents/helper/applications') return json({ applications: [approval] });
      if (path === '/api/portal/agents/helper/applications/approval-one/revoke') {
        writes.push({ method: route.request().method(), body: route.request().postDataJSON() });
        approval.revoked_at = now;
        return json({ revoked: true });
      }
      return route.fulfill({ status: 404, contentType: 'application/json', body: '{}' });
    });
    await page.goto(`${origin}/account/agents`);
    await expect(page.locator('#agents-content')).toBeVisible();
    await expect(page.locator('#create-section')).toBeVisible({ visible: allowed });
    await expect(page.getByRole('button', { name: 'Replace credential of helper' })).toBeVisible({ visible: allowed });
    const showApplications = page.getByRole('button', { name: 'Show applications of helper' });
    await expect(showApplications).toBeVisible();
    await showApplications.click();
    await expect(page.locator('.application-list')).toContainText('Mail (mail)');
    await expect(page.locator('.application-form')).toHaveCount(allowed ? 1 : 0);
    const revoke = page.getByRole('button', { name: 'Revoke Mail (mail) for helper' });
    await expect(revoke).toBeVisible();
    page.once('dialog', (dialog) => dialog.accept());
    await revoke.click();
    await expect(page.locator('#status')).toHaveText('helper can no longer use Mail (mail).');
    await expect(page.locator('.application-list')).toContainText('Revoked');
    expect(writes).toEqual([{ method: 'POST', body: { expected_user_id: 'owner-id', expected_session_id: 'session-one' } }]);
    expect(errors).toEqual([]);
  });
}
