// Exercise the production OTP helper and styles without a server or host clipboard.
// The visual slots must preserve one labeled native input for editing and autofill.
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readFileSync } from 'node:fs';

const origin = 'http://127.0.0.1:9876';
const assets = new Map(['auth.js', 'app.css'].map((name) =>
  [name, readFileSync(new URL(`../../src/portal/${name}`, import.meta.url), 'utf8')]));
const csp = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'";

async function mount(page, { sixOnly = false } = {}) {
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (/content.security.policy|csp violation|refused to (load|execute|apply)/i.test(message.text())) errors.push(message.text());
  });
  await page.route(`${origin}/**`, (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/otp') return route.fulfill({ status: 200, contentType: 'text/html', headers: { 'content-security-policy': csp }, body: `
      <!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
      <meta name="riauth-base" content="/"><title>Authenticator code</title>
      <link rel="stylesheet" href="/app.css"><link rel="stylesheet" href="/fixture.css"><script src="/auth.js" defer></script></head>
      <body><main><h1>Confirm it is you</h1><form id="code-form">
      <div class="field"><label for="code">Authenticator code</label>
      <input id="code" class="otp-input" data-otp ${sixOnly ? 'data-otp-digits="6"' : ''} name="otp" autocomplete="one-time-code"
        inputmode="numeric" autocapitalize="none" spellcheck="false" aria-describedby="code-hint">
      <p id="code-hint" class="field-hint">Enter the code from your authenticator app.</p>
      <button id="code-mode" class="text-button otp-switch" type="button" aria-controls="code">Use a recovery code</button></div>
      <button class="button primary" type="submit">Continue</button><button class="button secondary" type="reset">Reset</button>
      </form></main></body></html>` });
    if (path === '/fixture.css') return route.fulfill({ status: 200, contentType: 'text/css',
      body: 'body{padding:24px}main{max-width:420px;margin:0 auto}h1{margin-bottom:24px}form>.button{margin:4px}' });
    const name = path.slice(1);
    if (assets.has(name)) return route.fulfill({ status: 200,
      contentType: name.endsWith('.css') ? 'text/css' : 'text/javascript', body: assets.get(name) });
    return route.fulfill({ status: 404, body: '' });
  });
  await page.goto(`${origin}/otp`);
  await expect(page.locator('.otp-slot')).toHaveCount(6);
  return { input: page.locator('#code'), slots: page.locator('.otp-slot'), errors };
}

async function paste(input, value) {
  await input.evaluate((node, text) => {
    // Firefox strips ClipboardEvent clipboardData on synthetic events.
    const event = new Event('paste', { bubbles: true, cancelable: true });
    Object.defineProperty(event, 'clipboardData', { value: { getData: (type) => type === 'text/plain' ? text : '' } });
    node.dispatchEvent(event);
  }, value);
}
const slotText = (slots) => slots.evaluateAll((nodes) => nodes.map((node) => node.textContent).join(''));

test('six visual boxes retain one labeled native input and normal keyboard editing', async ({ page, browserName }) => {
  const { input, slots, errors } = await mount(page);
  await expect(page.locator('.otp-control input')).toHaveCount(1);
  await expect(page.getByRole('textbox', { name: 'Authenticator code', exact: true })).toHaveCount(1);
  await expect(page.locator('.otp-slots')).toHaveAttribute('aria-hidden', 'true');
  await expect(input).toHaveAttribute('autocomplete', 'one-time-code');
  await expect(input).toHaveAttribute('inputmode', 'numeric');
  await input.focus();
  await input.pressSequentially('012345');
  await expect(input).toHaveValue('012345');
  await expect.poll(() => slotText(slots)).toBe('012345');
  await input.press('ArrowLeft');
  await expect(input).toHaveJSProperty('selectionStart', 5);
  await input.press('Backspace');
  await expect(input).toHaveValue('01235');
  await input.pressSequentially('9');
  await expect(input).toHaveValue('012395');
  await expect.poll(() => slotText(slots)).toBe('012395');
  expect(await input.evaluate((node) => new FormData(node.form).getAll('otp'))).toEqual(['012395']);
  await input.press(process.platform === 'darwin' && browserName === 'webkit' ? 'Alt+Tab' : 'Tab');
  await expect(page.getByRole('button', { name: 'Use a recovery code' })).toBeFocused();
  expect(errors).toEqual([]);
});

test('clicking a filled box selects that digit for replacement', async ({ page }) => {
  const { input, slots, errors } = await mount(page);
  await input.fill('012345');
  const box = await slots.nth(2).boundingBox();
  await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
  await expect(input).toBeFocused();
  await expect(input).toHaveJSProperty('selectionStart', 2);
  await expect(input).toHaveJSProperty('selectionEnd', 3);
  await expect(page.locator('.otp-slot.is-selected')).toHaveCount(1);
  await input.pressSequentially('9');
  await expect(input).toHaveValue('019345');
  await expect.poll(() => slotText(slots)).toBe('019345');
  expect(errors).toEqual([]);
});

test('whole-code paste and autofill preserve leading zeroes and expand imported eight-digit codes', async ({ page }) => {
  const { input, slots, errors } = await mount(page);
  await paste(input, '012 345');
  await expect(input).toHaveValue('012345');
  await expect(slots).toHaveCount(6);
  await expect.poll(() => slotText(slots)).toBe('012345');
  await paste(input, '01 234 567');
  await expect(input).toHaveValue('01234567');
  await expect(slots).toHaveCount(8);
  await expect.poll(() => slotText(slots)).toBe('01234567');
  expect(await page.evaluate(() => RiAuth.otp('code').valid())).toBe(true);
  await input.evaluate((node) => {
    node.value = '000042';
    node.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertReplacementText', data: node.value }));
  });
  await expect(slots).toHaveCount(6);
  await expect.poll(() => slotText(slots)).toBe('000042');
  await input.evaluate((node) => { node.value = '000043'; node.dispatchEvent(new Event('change', { bubbles: true })); });
  await expect.poll(() => slotText(slots)).toBe('000043');
  await input.fill('1234567');
  await expect(slots).toHaveCount(8);
  expect(await page.evaluate(() => RiAuth.otp('code').valid())).toBe(false);
  await page.evaluate(() => RiAuth.otp('code').clear());
  await expect(input).toHaveValue('');
  await expect(slots).toHaveCount(6);
  await expect.poll(() => slotText(slots)).toBe('');
  expect(errors).toEqual([]);
});

test('new authenticator enrollment keeps its six-digit limit', async ({ page }) => {
  const { input, slots, errors } = await mount(page, { sixOnly: true });
  await expect(input).toHaveAttribute('maxlength', '6');
  await input.pressSequentially('01234567');
  await expect(input).toHaveValue('012345');
  await expect(slots).toHaveCount(6);
  expect(await page.evaluate(() => RiAuth.otp('code').valid())).toBe(true);
  // Programmatic values are still validated; an imported code cannot silently
  // turn a new six-digit enrollment into an eight-digit authenticator.
  await input.evaluate((node) => { node.value = '01234567'; node.dispatchEvent(new Event('input', { bubbles: true })); });
  expect(await page.evaluate(() => RiAuth.otp('code').valid())).toBe(false);
  await expect(slots).toHaveCount(6);
  expect(errors).toEqual([]);
});

test('recovery codes use the same plain input and clear/reset preserve the intended mode', async ({ page }) => {
  const { input, slots, errors } = await mount(page);
  await input.fill('012345');
  await page.getByRole('button', { name: 'Use a recovery code' }).click();
  await expect(input).toBeFocused();
  await expect(input).toHaveValue('');
  await expect(input).toHaveAccessibleName('Recovery code');
  await expect(input).toHaveAttribute('inputmode', 'text');
  await expect(input).toHaveAttribute('maxlength', '128');
  await expect(page.locator('.otp-slots')).toBeHidden();
  await expect.poll(() => slotText(slots)).toBe('');
  const code = `ri_recovery_${'a'.repeat(43)}`;
  await input.fill(code);
  expect(await page.evaluate(() => RiAuth.otp('code').value())).toBe(code);
  expect(await page.evaluate(() => RiAuth.otp('code').valid())).toBe(true);
  await page.evaluate(() => RiAuth.otp('code').clear());
  await expect(input).toHaveValue('');
  await expect(input).toHaveAccessibleName('Recovery code');
  await expect(page.locator('.otp-slots')).toBeHidden();
  await input.fill(code);
  await page.getByRole('button', { name: 'Reset', exact: true }).click();
  await expect(input).toHaveValue('');
  await expect(input).toHaveAccessibleName('Authenticator code');
  await expect(slots).toHaveCount(6);
  await expect(page.locator('.otp-slots')).toBeVisible();
  await expect.poll(() => slotText(slots)).toBe('');
  await input.fill('01234567');
  await page.evaluate(() => RiAuth.otp('code').reset());
  await expect(input).toHaveValue('');
  await expect(slots).toHaveCount(6);
  expect(errors).toEqual([]);
});

test('disabled and invalid states remain visible on the boxes', async ({ page }) => {
  const { input, slots, errors } = await mount(page);
  const paint = () => slots.first().evaluate((node) => {
    const slot = getComputedStyle(node), region = getComputedStyle(node.parentElement);
    return [slot.color, slot.backgroundColor, slot.borderColor, slot.opacity, region.opacity];
  });
  const normal = await paint();
  await input.evaluate((node) => { node.disabled = true; });
  await expect(input).toBeDisabled();
  await expect.poll(paint).not.toEqual(normal);
  await input.evaluate((node) => { node.disabled = false; node.setAttribute('aria-invalid', 'true'); });
  await expect(input).toBeEnabled();
  await expect(input).toHaveAttribute('aria-invalid', 'true');
  await expect.poll(paint).not.toEqual(normal);
  await input.evaluate((node) => { node.removeAttribute('aria-invalid'); });
  await expect.poll(paint).toEqual(normal);
  expect(errors).toEqual([]);
});

test('six and eight boxes fit narrow layouts and expose no extra accessibility controls', async ({ page }) => {
  const { input, slots, errors } = await mount(page);
  for (const width of [320, 768]) {
    await page.setViewportSize({ width, height: 800 });
    for (const code of ['012345', '01234567']) {
      await input.fill(code);
      await expect(slots).toHaveCount(code.length);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
      const bounds = await slots.evaluateAll((nodes) => nodes.map((node) => {
        const box = node.getBoundingClientRect();
        return { width: box.width, left: box.left, right: box.right };
      }));
      expect(bounds.every((box) => box.width > 0 && box.left >= 0 && box.right <= width)).toBe(true);
    }
  }
  const report = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  expect(report.violations.map(({ id, nodes }) => ({ id, targets: nodes.map(({ target }) => target) }))).toEqual([]);
  expect(errors).toEqual([]);
});

// Keep the real page markup, scripts, and shared styles in these integration checks;
// only the server replies are synthetic so they run without an embedded-asset rebuild.
async function mountPortalPage(page, name, respond) {
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  const replacements = { BASE: '/', AUTHORIZATION: 'test', STAGE: 'test', RESUME: 'api/stage/resume',
    CANCEL: 'api/stage/cancel', CONTINUE: '/done' };
  const html = readFileSync(new URL(`../../src/portal/${name}.html`, import.meta.url), 'utf8')
    .replace(/__([A-Z_]+)__/g, (_, key) => replacements[key]);
  const pageAssets = new Map(['appearance.js', 'auth.js', 'app.css', 'capabilities.js', `${name}.js`].map((asset) =>
    [asset, readFileSync(new URL(`../../src/portal/${asset}`, import.meta.url), 'utf8')]));
  pageAssets.set('security.css', readFileSync(new URL('../../src/portal/self_service/security.css', import.meta.url), 'utf8'));
  await page.route(`${origin}/**`, async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path.startsWith('/api/')) return respond(route, path);
    if (path === `/${name}`) return route.fulfill({ status: 200, contentType: 'text/html',
      headers: { 'content-security-policy': `${csp}; connect-src 'self'` }, body: html });
    const asset = path.replace('/portal/assets/', '');
    if (pageAssets.has(asset)) return route.fulfill({ status: 200,
      contentType: asset.endsWith('.css') ? 'text/css' : 'text/javascript', body: pageAssets.get(asset) });
    return route.fulfill({ status: 404, body: '' });
  });
  await page.goto(`${origin}/${name}`);
  return errors;
}
const invalidCode = (route) => route.fulfill({ status: 400, json: {
  error: 'invalid_code', error_description: 'The code was not accepted. Try another code.'
} });

for (const flow of [
  { name: 'sources', input: 'review-otp', submit: 'review-confirm', error: 'review-error' },
  { name: 'source-stage', input: 'stage-otp', submit: 'stage-resume', error: 'stage-error' }
]) {
  test(`${flow.name} requires a code and clears rejected digits without losing recovery mode`, async ({ page }) => {
    const attempts = [];
    const errors = await mountPortalPage(page, flow.name, async (route, path) => {
      const body = route.request().postDataJSON();
      if (path === '/api/portal/sources/review') return route.fulfill({ status: 200, json: {
        status: 'review', provider: 'Example provider', subject: 'example-person', local_otp_required: true,
        local_user: { username: 'alice', display_name: 'Alice' }, linking: false
      } });
      if (path === '/api/stage/resume' && !('otp' in body)) return route.fulfill({ status: 200,
        json: { status: 'local_factor_required', code_issued: false } });
      attempts.push(body);
      return invalidCode(route);
    });
    const input = page.locator(`#${flow.input}`), slots = page.locator('.otp-slot');
    const submit = page.locator(`#${flow.submit}`), error = page.locator(`#${flow.error}`);
    await expect(input).toBeVisible();
    await expect(input).toHaveJSProperty('required', true);
    await expect(slots).toHaveCount(6);
    await submit.click();
    await expect(input).toHaveAttribute('aria-invalid', 'true');
    await expect(error).toContainText(/Enter.*code/);
    expect(attempts).toEqual([]);
    await paste(input, '01 234 567');
    await expect(input).not.toHaveAttribute('aria-invalid');
    await expect(slots).toHaveCount(8);
    await submit.click();
    await expect(error).toContainText('The code was not accepted');
    expect(attempts).toHaveLength(1);
    expect(attempts[0].otp).toBe('01234567');
    await expect(input).toHaveValue('');
    await expect(slots).toHaveCount(6);
    await expect.poll(() => slotText(slots)).toBe('');
    await page.locator(`#${flow.input}-mode`).click();
    const recovery = `ri_recovery_${'b'.repeat(43)}`;
    await input.fill(recovery);
    await submit.click();
    await expect(input).toHaveValue('');
    expect(attempts).toHaveLength(2);
    expect(attempts[1].otp).toBe(recovery);
    await expect(input).toHaveAccessibleName('Recovery code');
    await expect(page.locator('.otp-slots')).toBeHidden();
    expect(errors).toEqual([]);
  });
}

test('device sign-in keeps alphanumeric device codes separate from optional authenticator boxes', async ({ page }) => {
  const attempts = [];
  const errors = await mountPortalPage(page, 'device', async (route, path) => {
    if (path === '/api/capabilities') return route.fulfill({ status: 200, json: {
      scope: 'instance', schema_version: 'riauth.capabilities/v2', feature_states: { 'oidc.device': { usable: true } }
    } });
    if (path === '/api/device/browser/ABCDE-12345') return route.fulfill({ status: 401, json: { error: 'unauthorized' } });
    if (path === '/api/portal/login/password') {
      attempts.push(route.request().postDataJSON());
      return invalidCode(route);
    }
    return route.fulfill({ status: 404, json: {} });
  });
  const deviceCode = page.locator('#device-code');
  await expect(deviceCode).toBeVisible();
  await expect(page.locator('.otp-control #device-code')).toHaveCount(0);
  await deviceCode.fill('abcde12345');
  await page.locator('#device-code-submit').click();
  await expect(page.locator('#device-signin-code')).toHaveText('ABCDE-12345');
  const input = page.locator('#device-otp'), slots = page.locator('.otp-slot');
  await expect(input).toBeVisible();
  await expect(input).toHaveJSProperty('required', false);
  await expect(slots).toHaveCount(6);
  await page.locator('#device-username').fill('alice');
  await page.locator('#device-password').fill('synthetic password');
  await input.fill('12345');
  await page.locator('#device-signin-submit').click();
  await expect(input).toHaveAttribute('aria-invalid', 'true');
  expect(attempts).toEqual([]);
  await paste(input, '012 345');
  await page.locator('#device-signin-submit').click();
  await expect(page.locator('#device-signin-error')).toContainText('The code was not accepted');
  expect(attempts).toHaveLength(1);
  expect(attempts[0].otp).toBe('012345');
  await expect(input).toHaveValue('');
  await expect.poll(() => slotText(slots)).toBe('');
  await page.locator('#device-password').fill('synthetic password');
  await page.locator('#device-signin-submit').click();
  await expect.poll(() => attempts.length).toBe(2);
  expect(attempts[1].otp).toBeNull();
  expect(errors).toEqual([]);
});
