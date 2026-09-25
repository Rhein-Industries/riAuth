// Starts examples/portal_fixture (build it first: cargo build --locked --example portal_fixture)
// and reads the JSON on its first stdout line. CARGO_TARGET_DIR is honoured, so a build in
// a custom target directory is found without copying it.
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';

// Key generation and password hashing belong to fixture setup, not the browser
// interaction deadline. Contended development hosts can exceed twenty seconds.
export const fixtureStartupMs = Number(process.env.RIAUTH_BROWSER_STARTUP_TIMEOUT_MS ?? 120000);
if (!Number.isSafeInteger(fixtureStartupMs) || fixtureStartupMs < 1 || fixtureStartupMs > 300000) {
  throw new Error('RIAUTH_BROWSER_STARTUP_TIMEOUT_MS must be an integer from 1 to 300000');
}

const repository = fileURLToPath(new URL('../..', import.meta.url));
export const fixtureExecutable = resolve(repository, process.env.CARGO_TARGET_DIR || 'target', 'debug', 'examples',
  process.platform === 'win32' ? 'portal_fixture.exe' : 'portal_fixture');

// Resolves to {fixture, stop}. `relyingParty` is the origin (http://localhost:PORT) the
// fixture registers its redirect URIs under. `stop()` sends SIGTERM and waits for the exit.
export async function startFixture({ relyingParty } = {}) {
  const env = relyingParty ? { ...process.env, RIAUTH_FIXTURE_RP_ORIGIN: relyingParty } : process.env;
  const service = spawn(fixtureExecutable, [], { env, stdio: ['ignore', 'pipe', 'inherit'] });
  const stop = async () => {
    if (service.exitCode !== null || service.signalCode !== null) return;
    const exited = new Promise((done) => service.once('exit', done));
    service.kill();
    await exited;
  };
  try {
    const fixture = await new Promise((resolveFixture, reject) => {
      const timeout = setTimeout(() => reject(new Error(`Fixture startup timed out after ${fixtureStartupMs} ms`)), fixtureStartupMs);
      service.once('error', (error) => { clearTimeout(timeout); reject(error); });
      service.once('exit', (code) => { clearTimeout(timeout); reject(new Error(`Fixture exited: ${code}`)); });
      createInterface({ input: service.stdout }).once('line', (line) => { clearTimeout(timeout); resolveFixture(JSON.parse(line)); });
    });
    return { fixture, stop };
  } catch (error) {
    await stop();
    throw error;
  }
}

// A stub relying party: every path answers with a small page, so redirects to the callback
// and the post-logout URI complete as real navigations (browser routes do not intercept a
// redirect target). Like the fixture, it listens on ::1 and 127.0.0.1 with one port.
export async function startRelyingParty() {
  const page = '<!doctype html><html lang="en"><head><title>Relying party</title></head><body><h1>Relying party</h1></body></html>';
  const handler = (request, response) => {
    response.writeHead(200, { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store' });
    response.end(page);
  };
  const listen = (host, port) => new Promise((done, fail) => {
    const server = createServer(handler);
    server.once('error', fail);
    server.listen({ host, port, ipv6Only: host === '::1' }, () => { server.off('error', fail); done(server); });
  });
  const close = (servers) => Promise.all(servers.map((server) => new Promise((done) => { server.closeAllConnections(); server.close(done); })));
  for (let attempt = 0; attempt < 5; attempt += 1) {
    let v6;
    try { v6 = await listen('::1', 0); } catch {
      const v4 = await listen('127.0.0.1', 0);
      return { origin: `http://localhost:${v4.address().port}`, close: () => close([v4]) };
    }
    const port = v6.address().port;
    try {
      const v4 = await listen('127.0.0.1', port);
      return { origin: `http://localhost:${port}`, close: () => close([v4, v6]) };
    } catch { await close([v6]); }
  }
  throw new Error('No loopback port was free on both 127.0.0.1 and ::1 after 5 attempts');
}
