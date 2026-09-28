// Focused lifecycle checks for the production browser helper; no browser or device is driven.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const source = readFileSync(new URL('../../src/portal/auth.js', import.meta.url), 'utf8');
const started = { ceremony: 'one-time-ceremony', public_key: { challenge: 'AQ', user: { id: 'Ag' } } };
const credential = { id: 'Aw', rawId: new Uint8Array([3]).buffer, type: 'public-key',
  response: { attestationObject: new Uint8Array([4]).buffer, clientDataJSON: new Uint8Array([5]).buffer } };
const deferred = () => { let resolve; const promise = new Promise((done) => { resolve = done; }); return { promise, resolve }; };
function helper(create) {
  const window = { addEventListener() {} };
  runInNewContext(source, { window, navigator: { credentials: { create } },
    document: { querySelector: () => ({ content: '/' }), addEventListener() {} },
    AbortController, DOMException, Uint8Array, atob, btoa, performance,
    setTimeout: () => 0, clearTimeout() {}, isSecureContext: true });
  return window.RiAuth;
}

test('native dismissal retries cached options; explicit cancellation consumes them and starts afresh', async () => {
  let prompts = 0, starts = 0, finishes = 0;
  const cancelled = [];
  const auth = helper(async () => {
    if (++prompts <= 2) throw new DOMException('Dismissed', 'NotAllowedError');
    return credential;
  });
  const flow = auth.passkeyFlow(async () => { starts += 1; return started; }, async () => { finishes += 1; }, true,
    async (options) => { cancelled.push(options.ceremony); });
  await assert.rejects(flow(), { name: 'NotAllowedError' });
  await assert.rejects(flow(), { name: 'NotAllowedError' });
  assert.equal(starts, 1); assert.equal(finishes, 0);
  assert.equal(await flow.cancel(), true);
  assert.deepEqual(cancelled, [started.ceremony]);
  await flow();
  assert.equal(starts, 2); assert.equal(finishes, 1);
});

test('cancel aborts an open authenticator and blocks a late credential response from finishing', async () => {
  const prompt = deferred(), opened = deferred();
  let signal, finishes = 0, cancels = 0;
  const auth = helper((options) => { signal = options.signal; opened.resolve(); return prompt.promise; });
  const flow = auth.passkeyFlow(async () => started, async () => { finishes += 1; }, true, async () => { cancels += 1; });
  const pending = flow();
  await opened.promise;
  await flow.cancel();
  assert.equal(signal.aborted, true);
  prompt.resolve(credential);
  await assert.rejects(pending, { name: 'AbortError' });
  assert.equal(finishes, 0); assert.equal(cancels, 1);
});

test('cancel while options are loading consumes the response without opening an authenticator', async () => {
  const options = deferred();
  let prompts = 0, finishes = 0, cancels = 0;
  const auth = helper(async () => { prompts += 1; return credential; });
  const flow = auth.passkeyFlow(() => options.promise, async () => { finishes += 1; }, true, async () => { cancels += 1; });
  const pending = flow();
  await flow.cancel(); options.resolve(started);
  await assert.rejects(pending, { name: 'AbortError' });
  assert.equal(prompts, 0); assert.equal(finishes, 0); assert.equal(cancels, 1);
});

test('a submitted finish cannot be reported as cancelled or replayed', async () => {
  const finish = deferred(), submitted = deferred();
  let finishes = 0, cancels = 0;
  const auth = helper(async () => credential);
  const flow = auth.passkeyFlow(async () => started, async () => {
    finishes += 1; submitted.resolve(); return finish.promise;
  }, true, async () => { cancels += 1; });
  const pending = flow();
  await submitted.promise;
  assert.equal(flow.finishing, true);
  assert.equal(await flow.cancel(), false);
  await assert.rejects(flow(), { name: 'InvalidStateError' });
  finish.resolve({ registered: true });
  assert.deepEqual(await pending, { registered: true });
  assert.equal(finishes, 1); assert.equal(cancels, 0); assert.equal(flow.finishing, false);
});
