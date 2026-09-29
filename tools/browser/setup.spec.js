// Browser acceptance against the shipped CLI and HTTP server. Build `riauth` first.
// Proofs are provisioned through private files; fixture credentials never go in URLs.
import { test, expect } from '@playwright/test';
import { spawn, execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, writeFile, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:net';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Network traces can contain setup POST bodies. Keep ownership proofs out of artifacts.
test.use({ trace: 'off', screenshot: 'off' });
const repository = fileURLToPath(new URL('../..', import.meta.url));
const binary = resolve(repository, process.env.CARGO_TARGET_DIR || 'target', 'debug', process.platform === 'win32' ? 'riauth.exe' : 'riauth');
const password = 'browser-setup-fixture-password';

async function start({ expiresIn = 900 } = {}) {
  const parent = resolve(repository, 'target/browser-setup');
  await mkdir(parent, { recursive: true });
  const dir = await mkdtemp(`${parent}/instance-`);
  const reservation = createServer();
  await new Promise((done) => reservation.listen(0, '127.0.0.1', done));
  const port = reservation.address().port;
  await new Promise((done) => reservation.close(done));
  const issuer = `http://127.0.0.1:${port}/identity`;
  const config = resolve(dir, 'riauth.toml'), proofFile = resolve(dir, 'proof');
  await writeFile(config, `issuer="${issuer}"\nlisten="127.0.0.1:${port}"\ndata_dir="data"\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n`, { mode: 0o600 });
  const env = { ...process.env }; delete env.RIAUTH_SERVER;
  execFileSync(binary, ['--config', config, '--non-interactive', 'prepare-setup', '--proof-file', proofFile,
    '--expires-in', String(expiresIn)], { env, stdio: 'pipe' });
  const proof = await readFile(proofFile, 'utf8');
  const service = spawn(binary, ['--config', config, 'serve'], { env, stdio: 'ignore' });
  const stop = async () => {
    if (service.exitCode === null && service.signalCode === null) {
      const exited = new Promise((done) => service.once('exit', done)); service.kill(); await exited;
    }
    await rm(dir, { recursive: true, force: true });
  };
  try {
    await expect.poll(async () => {
      if (service.exitCode !== null) throw new Error('Setup service exited');
      try { return (await fetch(`${issuer}/setup`)).status; } catch { return 0; }
    }).toBe(200);
    return { issuer, proof, stop };
  } catch (error) { await stop(); throw error; }
}

test('private ownership proof is single use, then ordinary cookie sign-in works', async ({ page, context, browser }) => {
  const fixture = await start();
  const problems = [], navigations = [];
  page.on('pageerror', (e) => problems.push(e.message));
  await page.addInitScript(() => document.addEventListener('securitypolicyviolation', (e) => {
    // Firefox probes favicon.ico on the script-free closed page; the strict CSP blocks it.
    if (e.violatedDirective === 'img-src' && e.blockedURI.endsWith('/favicon.ico')) return;
    console.error(`CSP: ${e.violatedDirective}`);
  }));
  page.on('console', (message) => {
    if (message.type() === 'error' && !message.text().includes('/favicon.ico')
      && /CSP:|content.security.policy|csp violation|refused to (load|execute|apply|connect|frame)/i.test(message.text())) problems.push(message.text());
  });
  page.on('framenavigated', (frame) => { navigations.push(frame.url()); });
  try {
    const response = await page.goto(`${fixture.issuer}/setup`);
    expect(response.status()).toBe(200);
    // Keep the private proof out of assertion output if HTML ever regresses.
    expect((await response.text()).includes(fixture.proof)).toBe(false);
    // Reload/prefetch consumes no ownership state, and storage holds no secrets.
    await page.reload();
    expect(await page.evaluate(() => [localStorage.length, sessionStorage.length])).toEqual([0, 0]);
    await page.getByLabel('Administrator username').fill('owner');
    await page.getByLabel('Display name').fill('Browser administrator');
    await page.getByLabel('Setup proof', { exact: true }).fill('ri_setup_' + 'x'.repeat(43));
    await page.getByLabel('Password', { exact: true }).fill(password);
    await page.getByLabel('Confirm password').fill(password + 'wrong');
    await page.getByRole('button', { name: 'Create administrator' }).click();
    await expect(page.getByRole('alert')).toHaveText('Your passwords do not match.');
    await page.getByLabel('Confirm password').fill(password);
    await page.getByRole('button', { name: 'Create administrator' }).click();
    await expect(page.getByRole('alert')).toContainText('invalid or expired');
    await expect(page.getByLabel('Setup proof', { exact: true })).toHaveValue('');
    await expect(page.getByLabel('Password', { exact: true })).toHaveValue('');
    await page.getByLabel('Setup proof', { exact: true }).fill(fixture.proof);
    await page.getByLabel('Password', { exact: true }).fill(password);
    await page.getByLabel('Confirm password').fill(password);
    await page.getByRole('button', { name: 'Create administrator' }).click();
    await expect(page.getByRole('heading', { name: 'Your administrator is ready' })).toBeVisible();
    await page.getByRole('link', { name: 'Continue to sign in' }).click();
    const passwordForm = page.locator('#password-form');
    await expect(passwordForm.getByLabel('Username', { exact: true })).toBeVisible();
    await passwordForm.getByLabel('Username', { exact: true }).fill('owner');
    await passwordForm.getByLabel('Password', { exact: true }).fill(password);
    await passwordForm.getByRole('button', { name: 'Sign in', exact: true }).click();
    await expect(page.locator('#auth')).toBeHidden();
    const cookies = await context.cookies();
    expect(cookies.some((c) => c.httpOnly && c.sameSite === 'Lax')).toBe(true);
    const stored = await page.evaluate(() => [localStorage, sessionStorage].flatMap((storage) =>
      Array.from({ length: storage.length }, (_, i) => {
        const key = storage.key(i); return `${key}\n${storage.getItem(key)}`;
      })).join('\n'));
    expect(stored.includes(fixture.proof) || stored.includes(password)).toBe(false);
    expect(navigations.every((url) => !url.includes(fixture.proof) && !url.includes(password))).toBe(true);
    const replay = await context.request.post(`${fixture.issuer}/api/setup`, {
      headers: { Origin: new URL(fixture.issuer).origin, 'X-Riauth-Portal': '1' },
      data: { proof: fixture.proof, username: 'intruder', password, display_name: 'Intruder', email: null }
    });
    expect(replay.status()).toBe(409);
    const closed = await page.goto(`${fixture.issuer}/setup`);
    expect(closed.status()).toBe(409);
    await expect(page.getByRole('heading', { name: 'Setup is complete' })).toBeVisible();
    await expect(page.getByLabel('Setup proof')).toHaveCount(0);
    const secondBrowser = await browser.newContext();
    try {
      const replayPage = await secondBrowser.newPage();
      const replayNavigation = await replayPage.goto(`${fixture.issuer}/setup`);
      expect(replayNavigation.status()).toBe(409);
      await expect(replayPage.getByRole('heading', { name: 'Setup is complete' })).toBeVisible();
      await expect(replayPage.getByLabel('Setup proof')).toHaveCount(0);
    } finally { await secondBrowser.close(); }
    expect(problems).toEqual([]);
  } finally { await fixture.stop(); }
});

test('a visitor without the private proof cannot claim the first administrator', async ({ browser }) => {
  const fixture = await start();
  const visitor = await browser.newContext();
  try {
    const page = await visitor.newPage();
    // Setup links carry no credential; a query-shaped proof cannot initialize a node.
    const suspiciousLink = await page.goto(`${fixture.issuer}/setup?proof=ri_setup_${'x'.repeat(43)}`);
    expect(suspiciousLink.status()).toBe(400);
    const response = await page.goto(`${fixture.issuer}/setup`);
    expect(response.status()).toBe(200);
    expect((await response.text()).includes(fixture.proof)).toBe(false);
    await page.getByLabel('Administrator username').fill('intruder');
    await page.getByLabel('Display name').fill('Uninvited visitor');
    await page.getByLabel('Setup proof', { exact: true }).fill('ri_setup_' + 'x'.repeat(43));
    await page.getByLabel('Password', { exact: true }).fill(password);
    await page.getByLabel('Confirm password').fill(password);
    await page.getByRole('button', { name: 'Create administrator' }).click();
    await expect(page.getByRole('alert')).toContainText('invalid or expired');
    await expect(page.getByLabel('Setup proof', { exact: true })).toHaveValue('');
    await expect(page.getByLabel('Password', { exact: true })).toHaveValue('');
    // A denied visitor cannot spend the claim; only a separate owner context gets the proof.
    const owner = await browser.newContext();
    try {
      const ownerPage = await owner.newPage();
      await ownerPage.goto(`${fixture.issuer}/setup`);
      await ownerPage.getByLabel('Administrator username').fill('owner');
      await ownerPage.getByLabel('Display name').fill('Browser administrator');
      await ownerPage.getByLabel('Setup proof', { exact: true }).fill(fixture.proof);
      await ownerPage.getByLabel('Password', { exact: true }).fill(password);
      await ownerPage.getByLabel('Confirm password').fill(password);
      await ownerPage.getByRole('button', { name: 'Create administrator' }).click();
      await expect(ownerPage.getByRole('heading', { name: 'Your administrator is ready' })).toBeVisible();
      const closed = await page.goto(`${fixture.issuer}/setup`);
      expect(closed.status()).toBe(409);
      await expect(page.getByRole('heading', { name: 'Setup is complete' })).toBeVisible();
    } finally { await owner.close(); }
  } finally { await visitor.close(); await fixture.stop(); }
});

test('an expired proof leaves the setup URL pending without an administrator', async ({ page }) => {
  const fixture = await start({ expiresIn: 1 });
  try {
    await page.goto(`${fixture.issuer}/setup`);
    // The verifier uses whole Unix seconds; two seconds passes the 1-second boundary
    // regardless of when preparation occurred within its first second.
    await new Promise((done) => setTimeout(done, 2100));
    await page.getByLabel('Administrator username').fill('owner');
    await page.getByLabel('Display name').fill('Browser administrator');
    await page.getByLabel('Setup proof', { exact: true }).fill(fixture.proof);
    await page.getByLabel('Password', { exact: true }).fill(password);
    await page.getByLabel('Confirm password').fill(password);
    await page.getByRole('button', { name: 'Create administrator' }).click();
    await expect(page.getByRole('alert')).toContainText('invalid or expired');
    await expect(page.getByLabel('Setup proof', { exact: true })).toHaveValue('');
    await expect(page.getByLabel('Password', { exact: true })).toHaveValue('');
    await page.goto(`${fixture.issuer}/apps`);
    await expect(page).toHaveURL(`${fixture.issuer}/setup`);
    await expect(page.getByRole('heading', { name: 'Set up your administrator' })).toBeVisible();
  } finally { await fixture.stop(); }
});
