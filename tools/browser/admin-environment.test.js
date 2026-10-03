import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { spawnSync } from 'node:child_process';

const source = readFileSync(new URL('../../src/portal/admin.js', import.meta.url), 'utf8');
const start = source.indexOf('  function envSnippet(');
const end = source.indexOf('\n  // riAuth\'s findings', start);
assert.ok(start >= 0 && end > start);
function snippet(connection, secret) {
  let copied;
  const context = {
    connection, secret,
    h: (...args) => args,
    copyButton: (text) => { copied = text; },
  };
  runInNewContext(source.slice(start, end) + '\nenvSnippet(connection, secret);', context);
  return copied;
}

test('copied shell assignments preserve literal configuration values', () => {
  const connection = {
    issuer: 'https://example.test/identity?literal=$HOME',
    client_id: "app's-client",
    redirect_uris: ['https://example.test/callback?x=$(printf unexpected >&2)'],
    scopes: ['openid', 'literal`printf unexpected >&2`', "quote's", '${HOME}'],
  };
  const secret = "space ; $HOME $(printf unexpected >&2) `printf unexpected >&2` ' \" \\ newline\nend";
  const text = snippet(connection, secret);
  const result = spawnSync('/bin/sh', ['-c', text + '\nprintf "%s\\0" "$OIDC_ISSUER" "$OIDC_CLIENT_ID" "$OIDC_CLIENT_SECRET" "$OIDC_REDIRECT_URI" "$OIDC_SCOPES"'],
    { encoding: 'utf8', env: { PATH: '/usr/bin:/bin', HOME: '/literal-test-home' }, timeout: 2000 });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0);
  assert.equal(result.stderr, '');
  assert.deepEqual(result.stdout.split('\0'), [connection.issuer, connection.client_id, secret,
    connection.redirect_uris[0], connection.scopes.join(' '), '']);
});

test('public-client snippet omits absent secret and redirect assignments', () => {
  const text = snippet({ issuer: 'https://example.test', client_id: 'public', redirect_uris: [], scopes: [] });
  assert.ok(!text.includes('OIDC_CLIENT_SECRET='));
  assert.ok(!text.includes('OIDC_REDIRECT_URI='));
  assert.ok(text.includes("OIDC_SCOPES=''"));
});
