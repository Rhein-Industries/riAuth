// Browser sign-in acceptance: the portal and the interaction page (OIDC sign-in, consent,
// terminal fallback and relying-party sign-out) in Chromium, Firefox and WebKit, against
// examples/portal_fixture and a stub relying party. Each engine gets its own fixture, and
// tests whose server state would collide (spent TOTP steps, remembered consent, enrolled
// passkeys) use separate accounts, so the file is not meant for --repeat-each.
import { test as base, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { createHash, randomBytes } from 'node:crypto';
import { fixtureStartupMs, startFixture, startRelyingParty } from './fixture.js';
import { totp, wrongCode } from './totp.js';

let fixture, stopFixture, relyingParty;
base.beforeAll(async () => {
  base.setTimeout(fixtureStartupMs + 5000);
  relyingParty = await startRelyingParty();
  ({ fixture, stop: stopFixture } = await startFixture({ relyingParty: relyingParty.origin }));
});
base.afterAll(async () => { await stopFixture?.(); await relyingParty?.close(); });

const CSP = /content.security.policy|csp violation|refused to (load|execute|apply|connect|frame)/i;

// Every test: no CSP violation or uncaught page error in any page of the context.
const test = base.extend({
  problems: [async ({ context }, use) => {
    const problems = [];
    await context.addInitScript(() => document.addEventListener('securitypolicyviolation',
      (event) => console.error(`CSP violation: ${event.violatedDirective} blocked ${event.blockedURI}`)));
    context.on('console', (message) => {
      if (['error', 'warning'].includes(message.type()) && CSP.test(message.text())) problems.push(`${message.type()}: ${message.text()}`);
    });
    context.on('weberror', (error) => problems.push(`page error: ${error.error().message}`));
    await use(problems);
    expect(problems, 'CSP violations and page errors').toEqual([]);
  }, { auto: true }]
});

// OIDC requests with PKCE computed here.
const b64u = (buffer) => buffer.toString('base64url');
function authorization(client, extra = {}) {
  const verifier = b64u(randomBytes(32)), state = b64u(randomBytes(12)), nonce = b64u(randomBytes(12));
  const query = new URLSearchParams({
    response_type: 'code', client_id: client, redirect_uri: fixture.redirect_uri, scope: 'openid profile email',
    state, nonce, code_challenge: b64u(createHash('sha256').update(verifier).digest()), code_challenge_method: 'S256', ...extra
  });
  return { client, verifier, state, nonce, url: `${fixture.issuer}/oauth/authorize?${query}` };
}
const atRelyingParty = (path) => (url) => url.origin === relyingParty.origin && url.pathname === path;
const isCallback = atRelyingParty('/callback');
// The callback carries code, state and iss (RFC 9207), or an error.
async function callback(page, request, timeout) {
  await page.waitForURL(isCallback, { timeout });
  const params = new URL(page.url()).searchParams;
  expect(params.get('state')).toBe(request.state);
  expect(params.get('iss')).toBe(fixture.issuer);
  return params;
}
async function redeem(request, code) {
  const response = await fetch(`${fixture.issuer}/oauth/token`, {
    method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded', accept: 'application/json' },
    body: new URLSearchParams({ grant_type: 'authorization_code', client_id: request.client, code, redirect_uri: fixture.redirect_uri, code_verifier: request.verifier })
  });
  const tokens = await response.json();
  expect(response.status, JSON.stringify(tokens)).toBe(200);
  const claims = JSON.parse(Buffer.from(tokens.id_token.split('.')[1], 'base64url').toString());
  expect(claims.nonce).toBe(request.nonce);
  expect(typeof claims.sid).toBe('string');
  return claims;
}
// A fresh terminal session: terminal approval needs a sign-in from the last five minutes.
async function terminalApprove(code) {
  const login = await fetch(`${fixture.issuer}/api/login`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ username: fixture.admin.username, password: fixture.admin.password })
  });
  expect(login.status).toBe(200);
  const { session_token: bearer } = await login.json();
  const decided = await fetch(`${fixture.issuer}/api/authorization/decision`, {
    method: 'POST', headers: { authorization: `Bearer ${bearer}`, 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify({ code, approve: true })
  });
  expect(decided.status, await decided.text()).toBe(200);
}

async function axe(page) {
  const report = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
  expect(report.violations.map(({ id, nodes }) => ({ id, nodes: nodes.map(({ target, failureSummary }) => ({ target, failureSummary })) }))).toEqual([]);
}
const fitsWidth = (page) => page.evaluate(() => document.documentElement.scrollWidth <= innerWidth);
// Reflow at 320 CSS px, then 200 % text at the original width (WCAG 1.4.10 and 1.4.4).
async function reflows(page) {
  const original = page.viewportSize();
  for (const width of [640, 320]) {
    await page.setViewportSize({ width, height: 800 });
    expect(await fitsWidth(page), `no horizontal scroll at ${width} px`).toBe(true);
  }
  await page.setViewportSize(original);
  await page.evaluate(() => {
    const nodes = [...document.body.querySelectorAll('*')];
    const sizes = nodes.map((node) => parseFloat(getComputedStyle(node).fontSize));
    nodes.forEach((node, i) => { node.style.fontSize = `${sizes[i] * 2}px`; });
  });
  expect(await fitsWidth(page), 'no horizontal scroll with 200 % text').toBe(true);
}
// WebKit on macOS moves focus only between text fields on Tab, like Safari; Option-Tab
// reaches every control there. Other engines and WebKit on Linux use Tab.
const tabKey = (browserName) => (browserName === 'webkit' && process.platform === 'darwin' ? 'Alt+Tab' : 'Tab');
const focusedId = (page) => page.evaluate(() => document.activeElement?.id ?? '');
async function tabTo(page, browserName, id, limit = 25) {
  for (let i = 0; i < limit; i += 1) {
    await page.keyboard.press(tabKey(browserName));
    if (await focusedId(page) === id) return;
  }
  throw new Error(`Tab did not reach #${id}`);
}

const screen = (page, name) => expect(page.locator(`#signin-${name}`)).toBeVisible();
async function signIn(page, user, otp = null) {
  await screen(page, 'authenticate');
  if (!(await page.locator('#signin-username').evaluate((input) => input.readOnly))) await page.locator('#signin-username').fill(user.username);
  await page.locator('#signin-password').fill(user.password);
  await page.locator('#signin-otp').fill(otp ?? '');
  await page.locator('#signin-submit').click();
}
async function portalSignIn(page, user) {
  await page.goto(`${fixture.issuer}/apps`);
  await expect(page.locator('#auth')).toBeVisible();
  await page.locator('#login-username').fill(user.username);
  await page.locator('#login-password').fill(user.password);
  await page.locator('#password-login').click();
  await expect(page.locator('#catalogue')).toBeVisible();
}
// Signs bob in through the implicit-consent client, which returns straight to the callback.
async function signedInAsBob(page) {
  const request = authorization(fixture.clients.implicit);
  await page.goto(request.url);
  await signIn(page, fixture.users.bob);
  await callback(page, request);
}
const ssoCookie = async (context) => (await context.cookies()).find((c) => /riauth_sso$/.test(c.name));
const scopes = ['Confirm your riAuth identity', 'See your email address', 'See your name and username'];
const consentScopes = async (page) => (await page.locator('#consent-scopes li').allTextContents()).sort();
// riAuth's uniform answer to a wrong password, code, unknown user or locked account.
const INVALID = 'Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes.';

test('portal password and TOTP sign-in is accessible and keeps cookies out of JavaScript', async ({ page, context, browserName }) => {
  const user = fixture.users.totp1;
  await page.goto(`${fixture.issuer}/apps`);
  await expect(page.locator('#auth')).toBeVisible();
  await axe(page);
  await reflows(page);
  await page.reload();
  await expect(page.locator('#auth')).toBeVisible();
  // Keyboard only: no autofocus, so start from the top of the page.
  await tabTo(page, browserName, 'login-username');
  await page.keyboard.type(user.username);
  await page.keyboard.press('Tab');
  expect(await focusedId(page)).toBe('login-password');
  await page.keyboard.type(user.password);
  await page.keyboard.press('Tab');
  expect(await focusedId(page)).toBe('login-otp');
  await page.keyboard.type(totp(user.totp_secret));
  await page.keyboard.press('Enter');
  await expect(page.locator('#catalogue')).toBeVisible();
  await expect(page.locator('#account-name')).toHaveText('Tess One');
  // An MFA session: no "extra verification" notice.
  await expect(page.locator('#mfa-notice')).toBeHidden();
  await axe(page);
  expect(await page.evaluate(() => document.cookie)).not.toMatch(/riauth_/);
  const sso = await ssoCookie(context);
  expect(sso?.httpOnly).toBe(true);
  expect(sso?.sameSite).toBe('Lax');
});

test('wrong code is announced and retry succeeds', async ({ page }) => {
  const user = fixture.users.totp2;
  const request = authorization(fixture.clients.consent, { prompt: 'consent' });
  await page.goto(request.url);
  await signIn(page, user, wrongCode(user.totp_secret));
  const error = page.getByRole('alert').filter({ hasText: INVALID });
  await expect(error).toBeVisible();
  await expect(page.locator('#signin-error')).toBeFocused();
  for (const field of ['#signin-username', '#signin-password', '#signin-otp']) await expect(page.locator(field)).toHaveAttribute('aria-invalid', 'true');
  await expect(page.locator('#signin-password')).toHaveValue('');
  await expect(page.locator('#signin-otp')).toHaveValue('');
  await signIn(page, user, totp(user.totp_secret));
  await screen(page, 'consent');
  await page.getByRole('button', { name: 'Allow' }).click();
  expect((await callback(page, request)).get('code')).toBeTruthy();
});

test('double submit is ignored while a request is in flight', async ({ page }) => {
  const request = authorization(fixture.clients.consent, { prompt: 'consent' });
  let posts = 0, release;
  const held = new Promise((resolve) => { release = resolve; });
  await page.route(/\/oauth\/resume\/[0-9a-f-]{36}\/password$/, async (route) => { posts += 1; await held; await route.continue(); });
  await page.goto(request.url);
  await screen(page, 'authenticate');
  await page.locator('#signin-username').fill(fixture.users.bob.username);
  await page.locator('#signin-password').fill(fixture.users.bob.password);
  await page.locator('#signin-submit').dblclick();
  await expect(page.locator('#signin-submit')).toHaveAttribute('aria-busy', 'true');
  await expect(page.locator('#signin-submit')).toBeDisabled();
  await page.locator('#signin-username').press('Enter');
  await page.locator('#signin-password').press('Enter');
  await expect.poll(() => posts).toBe(1);
  release();
  await screen(page, 'consent');
  await expect(page.locator('#signin-error')).toBeHidden();
  expect(posts).toBe(1);
});

test('OIDC sign-in with password and TOTP, consent, callback and silent return', async ({ page }) => {
  const user = fixture.users.totp3;
  const request = authorization(fixture.clients.consent);
  await page.goto(request.url);
  await screen(page, 'authenticate');
  await expect(page.locator('#signin-title')).toHaveText('Sign in to continue to Consent Test App');
  await expect(page.locator('#signin-app-host')).toHaveText("You'll return to localhost");
  await expect(page.locator('#signin-expiry')).toHaveText(/^This request expires at /);
  await expect(page).toHaveTitle('Sign in to Consent Test App · riAuth');
  await signIn(page, user, totp(user.totp_secret));
  await screen(page, 'consent');
  await expect(page.locator('#consent-title')).toHaveText('Consent Test App wants to use your riAuth account');
  await expect(page.locator('#consent-title')).toBeFocused();
  await expect(page.locator('#consent-account')).toHaveText('Signed in as Tess Three (@totp3)');
  expect(await consentScopes(page)).toEqual(scopes);
  await expect(page.getByRole('checkbox', { name: 'Remember this decision for 30 days' })).toBeChecked();
  await page.getByRole('button', { name: 'Allow' }).click();
  const params = await callback(page, request);
  const claims = await redeem(request, params.get('code'));
  expect(claims.amr).toEqual(['pwd', 'otp']);
  expect(claims.acr).toBe('urn:riauth:acr:mfa');
  expect(claims.preferred_username).toBe('totp3');
  // The remembered consent and the browser session make the next request silent.
  const again = authorization(fixture.clients.consent);
  await page.goto(again.url);
  const silent = await redeem(again, (await callback(page, again)).get('code'));
  expect(silent.sid).toBe(claims.sid);
});

test('implicit consent returns straight to the application', async ({ page }) => {
  const request = authorization(fixture.clients.implicit);
  await page.goto(request.url);
  await screen(page, 'authenticate');
  await expect(page.locator('#signin-title')).toHaveText('Sign in to continue to Trusted Test App');
  // No consent screen: the page reaches the application without a decision from the user.
  const decisions = [];
  page.on('request', (r) => { if (r.method() === 'POST' && r.url().endsWith('/decision')) decisions.push(r.url()); });
  await signIn(page, fixture.users.bob);
  expect((await callback(page, request)).get('code')).toBeTruthy();
  expect(decisions).toEqual([]);
});

test('an MFA-only application refuses a password-only sign-in with the right message', async ({ page, context }) => {
  const request = authorization(fixture.clients.mfa);
  let posts = 0;
  page.on('request', (r) => { if (r.method() === 'POST' && r.url().endsWith('/password')) posts += 1; });
  await page.goto(request.url);
  await screen(page, 'authenticate');
  // The page gives a browser without an SSO cookie a placeholder that signs nothing in.
  const placeholder = await ssoCookie(context);
  expect(placeholder?.value).toMatch(/^ri_sso_/);
  await expect(page.locator('#signin-requirement')).toHaveText('Secure Test App requires a passkey or an authenticator code. No passkey yet? Open your applications portal to add one.');
  await expect(page.locator('#signin-otp')).toHaveAttribute('required', '');
  // No code: refused before anything is sent.
  await signIn(page, fixture.users.bob);
  await expect(page.locator('#signin-error')).toHaveText('Enter your authenticator or recovery code, or sign in with a passkey.');
  expect(posts).toBe(0);
  // bob has no second factor, so any code is ignored and the server refuses the password.
  await signIn(page, fixture.users.bob, '123456');
  const error = page.locator('#signin-error');
  await expect(error).toHaveText('Secure Test App requires a passkey or an authenticator code. Add a passkey in your applications portal first. Open your applications portal');
  await expect(error).toBeFocused();
  await expect(error.getByRole('link', { name: 'Open your applications portal' })).toHaveAttribute('href', `${new URL(fixture.issuer).pathname}/apps`);
  expect(posts).toBe(1);
  await screen(page, 'authenticate');
  // No session: the placeholder is unchanged and the portal is still signed out.
  expect((await ssoCookie(context))?.value).toBe(placeholder.value);
  expect(await page.evaluate((url) => fetch(url).then((r) => r.status), `${fixture.issuer}/api/portal`)).toBe(401);
  await axe(page);
});

test('prompt=login asks the pinned account to confirm', async ({ page }) => {
  await signedInAsBob(page);
  const request = authorization(fixture.clients.implicit, { prompt: 'login' });
  await page.goto(request.url);
  await screen(page, 'authenticate');
  await expect(page.locator('#signin-title')).toHaveText("Confirm it's you");
  await expect(page.locator('#signin-reason')).toHaveText('Trusted Test App asks you to sign in again.');
  await expect(page.locator('#signin-account-text')).toHaveText('Signed in as Bob Example (@bob)');
  await expect(page.locator('#signin-username')).toHaveValue('bob');
  expect(await page.locator('#signin-username').evaluate((input) => input.readOnly)).toBe(true);
  await axe(page);
  await signIn(page, { ...fixture.users.bob, password: 'not-the-password' });
  await expect(page.locator('#signin-error')).toHaveText(INVALID);
  await signIn(page, fixture.users.bob);
  expect((await callback(page, request)).get('code')).toBeTruthy();
  // "Use another account" signs this browser out and unpins the form.
  const other = authorization(fixture.clients.implicit, { prompt: 'login' });
  await page.goto(other.url);
  await expect(page.locator('#signin-account')).toBeVisible();
  await page.getByRole('button', { name: 'Use another account' }).click();
  await expect(page.locator('#signin-account')).toBeHidden();
  expect(await page.locator('#signin-username').evaluate((input) => input.readOnly)).toBe(false);
  await signIn(page, fixture.users.bob);
  expect((await callback(page, other)).get('code')).toBeTruthy();
});

test('terminal approval from the sign-in page', async ({ page, context }) => {
  const request = authorization(fixture.clients.consent, { prompt: 'consent' });
  await page.goto(request.url);
  await screen(page, 'authenticate');
  await page.getByText('Use your terminal instead').click();
  await expect(page.locator('#signin-terminal')).toHaveAttribute('open', '');
  const code = (await page.locator('#signin-code').innerText()).trim();
  expect(code).toMatch(/^[A-Z0-9-]{10,}$/);
  await expect(page.locator('#signin-command')).toHaveText(`riauth --server '${fixture.issuer}' request approve ${code}`);
  await axe(page);
  await terminalApprove(code);
  // Two-second polling while the panel is open.
  const params = await callback(page, request, 5000);
  const claims = await redeem(request, params.get('code'));
  expect(claims.preferred_username).toBe(fixture.admin.username);
  // The browser now holds the approving account's session.
  expect(await ssoCookie(context)).toBeDefined();
  await page.goto(`${fixture.issuer}/apps`);
  await expect(page.locator('#account-name')).toHaveText('Test administrator');
  // A terminal approval can be phished, so this browser signs in itself before it may
  // change passkeys; the terminal session stays as it is.
  await page.locator('#account-security').click();
  await expect(page.locator('#reauth-hint')).toHaveText("This browser uses your terminal's sign-in. Sign in here to change your passkeys.");
  await expect(page.locator('#reauth-panel')).toBeVisible();
  await page.locator('#reauth-password').fill(fixture.admin.password);
  await page.locator('#reauth-confirm').click();
  await expect(page.locator('#reauth-panel')).toBeHidden();
  await expect(page.locator('#security-status')).toHaveText('You have no passkeys yet.');
});

test('consent deny returns access_denied to the application', async ({ page }) => {
  const request = authorization(fixture.clients.consent, { prompt: 'consent' });
  await page.goto(request.url);
  await signIn(page, fixture.users.bob);
  await screen(page, 'consent');
  await page.getByRole('button', { name: 'Deny' }).click();
  const params = await callback(page, request);
  expect(params.get('error')).toBe('access_denied');
  expect(params.get('code')).toBeNull();
});

test('RP logout confirmation signs this browser out', async ({ page, context }) => {
  await signedInAsBob(page);
  const logout = (state) => `${fixture.issuer}/oauth/logout?${new URLSearchParams({ client_id: fixture.clients.implicit, post_logout_redirect_uri: fixture.post_logout_redirect_uri, state })}`;
  const signedOut = atRelyingParty('/signed-out');
  await page.goto(logout('stay'));
  await screen(page, 'logout');
  await expect(page.locator('#logout-title')).toHaveText('Sign out of riAuth?');
  await expect(page.locator('#logout-title')).toBeFocused();
  await expect(page.locator('#logout-text')).toHaveText("You're signed in as Bob Example. This signs you out of all applications that use riAuth in this browser.");
  await expect(page.locator('#logout-account')).toHaveText('Signed in as Bob Example (@bob)');
  await axe(page);
  // Staying signed in keeps the session: the next request is silent.
  await page.getByRole('button', { name: 'Stay signed in' }).click();
  await page.waitForURL(signedOut);
  expect(new URL(page.url()).searchParams.get('state')).toBe('stay');
  expect(await ssoCookie(context)).toBeDefined();
  const kept = authorization(fixture.clients.implicit);
  await page.goto(kept.url);
  await callback(page, kept);
  await page.goto(logout('bye'));
  await page.getByRole('button', { name: 'Sign out', exact: true }).click();
  await page.waitForURL(signedOut);
  expect(new URL(page.url()).searchParams.get('state')).toBe('bye');
  expect(await ssoCookie(context)).toBeUndefined();
  const after = authorization(fixture.clients.implicit);
  await page.goto(after.url);
  await screen(page, 'authenticate');
  await expect(page.locator('#signin-title')).toHaveText('Sign in to continue to Trusted Test App');
  await expect(page.locator('#signin-account')).toBeHidden();
});

// Playwright's WebAuthn shim answers navigator.credentials in every engine. It tests the
// pages and the server, not the browser's own WebAuthn. It is installed before the page
// exists, as Playwright documents.
test('passkey enrollment and passwordless sign-in', async ({ context }) => {
  const user = fixture.users.passkey;
  await context.credentials.install();
  const page = await context.newPage();
  await page.goto(`${fixture.issuer}/apps`);
  test.skip(!(await page.evaluate(() => 'PublicKeyCredential' in window && isSecureContext)),
    'This engine build has no WebAuthn, so the pages hide their passkey buttons');
  await portalSignIn(page, user);
  await page.locator('#account-security').click();
  await expect(page.getByRole('dialog', { name: 'Passkeys' })).toBeVisible();
  await expect(page.locator('#security-status')).toHaveText('You have no passkeys yet.');
  await axe(page);
  await page.getByRole('button', { name: 'Add a passkey' }).click();
  await expect(page.locator('#toast')).toHaveText('Passkey added. Sign in with it to continue.');
  await expect(page.locator('#auth')).toBeVisible();
  await expect(page.locator('#passkey-login')).toBeFocused();
  expect(await context.credentials.get({ rpId: 'localhost' })).toHaveLength(1);
  // Discoverable sign-in in the portal gives an MFA session.
  await page.keyboard.press('Enter');
  await expect(page.locator('#catalogue')).toBeVisible();
  await expect(page.locator('#account-name')).toHaveText('Pat Passkey');
  await expect(page.locator('#mfa-notice')).toBeHidden();
  await page.getByRole('button', { name: 'Sign out' }).click();
  await expect(page.locator('#auth')).toBeVisible();
  // Discoverable sign-in on the interaction page satisfies an MFA-only application.
  const request = authorization(fixture.clients.mfa);
  await page.goto(request.url);
  await screen(page, 'authenticate');
  await page.getByRole('button', { name: 'Sign in with a passkey' }).click();
  await screen(page, 'consent');
  await expect(page.locator('#consent-account')).toHaveText('Signed in as Pat Passkey (@passkey1)');
  await page.getByRole('button', { name: 'Allow' }).click();
  const claims = await redeem(request, (await callback(page, request)).get('code'));
  expect(claims.preferred_username).toBe('passkey1');
  expect(claims.amr).toContain('webauthn');
  // prompt=login: the pinned account confirms with the same passkey.
  const again = authorization(fixture.clients.mfa, { prompt: 'login' });
  await page.goto(again.url);
  await expect(page.locator('#signin-title')).toHaveText("Confirm it's you");
  await expect(page.locator('#signin-account-text')).toHaveText('Signed in as Pat Passkey (@passkey1)');
  await page.getByRole('button', { name: 'Sign in with a passkey' }).click();
  const confirmed = await redeem(again, (await callback(page, again)).get('code'));
  expect(confirmed.sid).toBe(claims.sid);
  expect(confirmed.auth_time).toBeGreaterThanOrEqual(claims.auth_time);
});

test('chromium native WebAuthn virtual authenticator', async ({ page, context, browserName }) => {
  test.skip(browserName !== 'chromium', 'The CDP WebAuthn virtual authenticator exists only in Chromium; the shim test covers the pages in every engine');
  const user = fixture.users.native;
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  const { authenticatorId } = await cdp.send('WebAuthn.addVirtualAuthenticator', {
    options: { protocol: 'ctap2', transport: 'internal', hasResidentKey: true, hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true }
  });
  await portalSignIn(page, user);
  await page.locator('#account-security').click();
  await page.getByRole('button', { name: 'Add a passkey' }).click();
  await expect(page.locator('#toast')).toHaveText('Passkey added. Sign in with it to continue.');
  const { credentials } = await cdp.send('WebAuthn.getCredentials', { authenticatorId });
  expect(credentials).toHaveLength(1);
  expect(credentials[0].isResidentCredential).toBe(true);
  const request = authorization(fixture.clients.mfa);
  await page.goto(request.url);
  await page.getByRole('button', { name: 'Sign in with a passkey' }).click();
  await screen(page, 'consent');
  await page.getByRole('button', { name: 'Allow' }).click();
  const claims = await redeem(request, (await callback(page, request)).get('code'));
  expect(claims.preferred_username).toBe('passkey2');
  expect(claims.amr).toContain('webauthn');
  // A failed user verification is reported; the retry reuses the kept options.
  const again = authorization(fixture.clients.mfa, { prompt: 'login' });
  let starts = 0;
  page.on('request', (r) => { if (r.method() === 'POST' && r.url().endsWith('/passkey/start')) starts += 1; });
  await page.goto(again.url);
  await expect(page.locator('#signin-title')).toHaveText("Confirm it's you");
  await cdp.send('WebAuthn.setUserVerified', { authenticatorId, isUserVerified: false });
  await page.getByRole('button', { name: 'Sign in with a passkey' }).click();
  await expect(page.locator('#signin-error')).toHaveText('Passkey sign-in was cancelled or timed out. Select the button to try again.');
  await cdp.send('WebAuthn.setUserVerified', { authenticatorId, isUserVerified: true });
  await page.getByRole('button', { name: 'Sign in with a passkey' }).click();
  const confirmed = await redeem(again, (await callback(page, again)).get('code'));
  expect(confirmed.sid).toBe(claims.sid);
  expect(starts).toBe(1);
});

test('state polling survives a 503', async ({ page }) => {
  const request = authorization(fixture.clients.consent, { prompt: 'consent' });
  let failNext = 1, failed = 0;
  await page.route(/\/oauth\/resume\/[0-9a-f-]{36}\/state$/, (route) => {
    if (failNext <= 0) return route.continue();
    failNext -= 1; failed += 1;
    return route.fulfill({ status: 503, contentType: 'application/json', body: JSON.stringify({ error: 'temporarily_unavailable', error_description: 'riAuth is busy' }) });
  });
  // One 503 on load is retried inside the same fetch.
  await page.goto(request.url);
  await screen(page, 'authenticate');
  expect(failed).toBe(1);
  // Three in a row exhaust one poll's retries; the page keeps its screen and polls again.
  failNext = 3;
  await page.getByText('Use your terminal instead').click();
  const code = (await page.locator('#signin-code').innerText()).trim();
  await expect.poll(() => failed, { timeout: 15000 }).toBe(4);
  await screen(page, 'authenticate');
  await expect(page.locator('#signin-message')).toBeHidden();
  await terminalApprove(code);
  expect((await callback(page, request, 10000)).get('code')).toBeTruthy();
});

test('sign-in and consent work with the keyboard alone and pass axe', async ({ page, browserName }) => {
  const request = authorization(fixture.clients.consent, { prompt: 'consent' });
  await page.goto(request.url);
  await screen(page, 'authenticate');
  await expect(page.locator('#signin-title')).toBeFocused();
  await axe(page);
  await reflows(page);
  await page.reload();
  await screen(page, 'authenticate');
  await tabTo(page, browserName, 'signin-username');
  await page.keyboard.type(fixture.users.bob.username);
  await page.keyboard.press('Tab');
  expect(await focusedId(page)).toBe('signin-password');
  await page.keyboard.type('wrong-password');
  await page.keyboard.press('Enter');
  await expect(page.locator('#signin-error')).toBeFocused();
  await axe(page);
  await tabTo(page, browserName, 'signin-password');
  await page.keyboard.type(fixture.users.bob.password);
  await page.keyboard.press('Enter');
  await screen(page, 'consent');
  await expect(page.locator('#consent-title')).toBeFocused();
  await axe(page);
  // Keep this decision out of the remembered consents, then allow.
  await tabTo(page, browserName, 'consent-remember');
  await page.keyboard.press('Space');
  await expect(page.locator('#consent-remember')).not.toBeChecked();
  await tabTo(page, browserName, 'consent-allow');
  // Decisions are ignored for a moment after the screen appears (double-click-jacking).
  await expect(page.locator('#consent-allow')).not.toHaveAttribute('aria-disabled', 'true');
  await page.keyboard.press('Enter');
  expect((await callback(page, request)).get('code')).toBeTruthy();
});

// Double-click-jacking: a cross-site page opens a popup over its own tab, moves that tab to
// a riAuth decision screen and closes the popup on the first click of a double-click, so the
// second click lands on the decision. The page is never framed, so frame-ancestors cannot
// help; decision buttons ignore clicks for a moment after the window is shown or focused.
test('decision buttons ignore a click right after the window regains focus', async ({ page, context }) => {
  const request = authorization(fixture.clients.consent, { prompt: 'consent' });
  await page.goto(request.url);
  await signIn(page, fixture.users.bob);
  await screen(page, 'consent');
  const decisions = [];
  page.on('request', (r) => { if (r.method() === 'POST' && r.url().endsWith('/decision')) decisions.push(r.url()); });
  const allow = page.locator('#consent-allow');
  await expect(allow).not.toHaveAttribute('aria-disabled', 'true');
  const box = await allow.boundingBox();
  // The popup closes and this window regains focus; the second click follows at once.
  await page.evaluate(() => window.dispatchEvent(new Event('focus')));
  await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
  await expect(allow).toHaveAttribute('aria-disabled', 'true');
  await page.waitForTimeout(700);
  expect(decisions).toEqual([]);
  await screen(page, 'consent');
  await expect(allow).not.toHaveAttribute('aria-disabled', 'true');
  // A deliberate click afterwards decides as usual.
  await allow.click();
  expect((await callback(page, request)).get('code')).toBeTruthy();
  expect(decisions).toHaveLength(1);
  // The same holds for the sign-out confirmation.
  await page.goto(`${fixture.issuer}/oauth/logout?${new URLSearchParams({ client_id: fixture.clients.implicit, post_logout_redirect_uri: fixture.post_logout_redirect_uri, state: 'jacked' })}`);
  await screen(page, 'logout');
  const confirm = page.locator('#logout-confirm');
  await expect(confirm).toHaveText('Sign out');
  await expect(confirm).not.toHaveAttribute('aria-disabled', 'true');
  await page.evaluate(() => { window.dispatchEvent(new Event('focus')); document.getElementById('logout-confirm').click(); });
  await page.waitForTimeout(700);
  await screen(page, 'logout');
  expect(await ssoCookie(context)).toBeDefined();
  expect(await page.evaluate((url) => fetch(url).then((r) => r.status), `${fixture.issuer}/api/portal`)).toBe(200);
});

test.describe('on a phone-sized screen', () => {
  test.use({ viewport: { width: 390, height: 844 } });
  test('sign-in and consent fit without horizontal scrolling', async ({ page }) => {
    const request = authorization(fixture.clients.consent, { prompt: 'consent' });
    await page.goto(request.url);
    await screen(page, 'authenticate');
    expect(await fitsWidth(page)).toBe(true);
    // Inputs and the terminal summary are at least 44 px high (the app.css contract).
    for (const selector of ['#signin-username', '#signin-password', '#signin-otp', '#signin-terminal > summary']) {
      expect((await page.locator(selector).boundingBox()).height, `${selector} height`).toBeGreaterThanOrEqual(44);
    }
    await signIn(page, fixture.users.bob);
    await screen(page, 'consent');
    await page.setViewportSize({ width: 320, height: 640 });
    expect(await fitsWidth(page)).toBe(true);
    expect(await consentScopes(page)).toEqual(scopes);
    // Buttons meet the WCAG 2.2 AA target size (24 px) and stay inside the viewport.
    for (const selector of ['#consent-allow', '#consent-deny', '#consent-remember-label']) {
      const box = await page.locator(selector).boundingBox();
      expect(box.height, `${selector} height`).toBeGreaterThanOrEqual(24);
      expect(box.x + box.width, `${selector} right edge`).toBeLessThanOrEqual(320);
    }
    expect((await page.locator('#consent-remember-label').boundingBox()).height).toBeGreaterThanOrEqual(44);
    await axe(page);
    await page.locator('#consent-remember').uncheck();
    await page.getByRole('button', { name: 'Allow' }).click();
    expect((await callback(page, request)).get('code')).toBeTruthy();
  });
});
