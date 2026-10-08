// The My agents page's pure helpers, run on their own; no browser or server is driven.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const source = readFileSync(new URL('../../src/portal/self_service/agents.js', import.meta.url), 'utf8');
const start = source.indexOf('  // ---- Helpers');
const end = source.indexOf('  // ---- Page', start);
assert.ok(start >= 0 && end > start);
const helpers = runInNewContext(`${source.slice(start, end)}
({ PERSONAL, LIFETIMES, validAgentId, parsePermissions, agentStatus, sortAgents, openProposals, credentialFile,
  applicationLabel, applicationLifetimes, applicationScopes, applicationRequest, approvalStatus, sortApprovals,
  exchangeParameters, shellWord, agentTokenCommand });`, {});
// Values cross the vm boundary; compare their JSON form.
const plain = (value) => JSON.parse(JSON.stringify(value));

test('checked personal actions name self and advanced lines add exact permissions once', () => {
  const parsed = helpers.parsePermissions(['profile.read', 'sessions.read'],
    '\n  state.read=state/revision \r\nprofile.read=self\nuser.read=user/alice@example\n');
  assert.deepEqual(plain(parsed), { permissions: [
    { action: 'profile.read', resource: 'self' },
    { action: 'sessions.read', resource: 'self' },
    { action: 'state.read', resource: 'state/revision' },
    { action: 'user.read', resource: 'user/alice@example' },
  ] });
  assert.deepEqual(plain(helpers.parsePermissions([], '')), { permissions: [] });
});

test('an advanced line that is not one action=resource names its line', () => {
  for (const [text, line] of [['profile.read', 1], ['ok.read=self\nuser.read = user/a', 2],
    ['a=b=c', 1], ['=self', 1], ['x.read=', 1], ['x.read=a b', 1]]) {
    assert.equal(helpers.parsePermissions([], text).error, `Line ${line} is not one action=resource permission.`);
  }
});

test('agent names follow the server name rule', () => {
  for (const id of ['helper', 'mail.bot-1', 'a@b_c', 'x'.repeat(64)]) assert.equal(helpers.validAgentId(id), true, id);
  for (const id of ['', '.', '..', 'x'.repeat(65), 'with space', 'slash/name', 'ümlaut']) assert.equal(helpers.validAgentId(id), false, id);
});

test('status separates revoked, expired and active agents, and active ones sort first', () => {
  const now = 1000;
  const agents = [
    { id: 'b-revoked', enabled: false, expires_at: 2000 },
    { id: 'c-expired', enabled: true, expires_at: 1000 },
    { id: 'z-active', enabled: true, expires_at: 1001 },
    { id: 'a-active', enabled: true, expires_at: 5000 },
  ];
  assert.deepEqual(agents.map((agent) => helpers.agentStatus(agent, now)), ['revoked', 'expired', 'active', 'active']);
  assert.deepEqual(plain(helpers.sortAgents(agents, now).map((agent) => agent.id)), ['a-active', 'z-active', 'c-expired', 'b-revoked']);
});

test('open proposals exclude expired ones and names already issued', () => {
  const proposals = [
    { proposal_id: 'p1', expires_at: 2000, agent: { id: 'issued' } },
    { proposal_id: 'p2', expires_at: 900, agent: { id: 'late' } },
    { proposal_id: 'p3', expires_at: 2000, agent: { id: 'fresh' } },
  ];
  const open = helpers.openProposals(proposals, [{ id: 'issued' }], 1000);
  assert.deepEqual(plain(open.map((proposal) => proposal.proposal_id)), ['p3']);
});

test('the credential file is the compact JSON the CLI writes, in server order', () => {
  const credential = { agent_id: 'helper', expires_at: 1700000000, issuer: 'https://id.example.test', token: 'ri_agent_x' };
  assert.equal(helpers.credentialFile(credential),
    '{"agent_id":"helper","expires_at":1700000000,"issuer":"https://id.example.test","token":"ri_agent_x"}');
});

test('the form offers every personal action and the four lifetimes', () => {
  assert.deepEqual(plain(helpers.PERSONAL.map(([action]) => action)), ['profile.read', 'profile.write', 'sessions.read',
    'sessions.revoke', 'consents.read', 'consents.revoke', 'agents.read', 'agents.revoke']);
  assert.deepEqual(plain(helpers.LIFETIMES.map(([seconds]) => seconds)), [3600, 86400, 604800, 2592000]);
  const html = readFileSync(new URL('../../src/portal/self_service/agents.html', import.meta.url), 'utf8');
  for (const [seconds] of helpers.LIFETIMES) assert.ok(html.includes(`name="lifetime" value="${seconds}"`), String(seconds));
});

const mail = { client_id: 'jmap', name: 'Mail', scopes: ['contacts', 'mail'],
  resources: [{ uri: 'https://mail.example.test/jmap', scopes: ['mail'] }] };

test('applications read as their name with the client id, or the id alone', () => {
  const listed = [mail, { client_id: 'notes', name: 'notes', scopes: ['notes'], resources: [] }];
  assert.equal(helpers.applicationLabel(listed, 'jmap'), 'Mail (jmap)');
  assert.equal(helpers.applicationLabel(listed, 'notes'), 'notes');
  assert.equal(helpers.applicationLabel(listed, 'gone'), 'gone');
});

test('a resource narrows the scopes on offer to those it covers', () => {
  assert.deepEqual(plain(helpers.applicationScopes(mail, '')), ['contacts', 'mail']);
  assert.deepEqual(plain(helpers.applicationScopes(mail, 'https://mail.example.test/jmap')), ['mail']);
  assert.deepEqual(plain(helpers.applicationScopes(mail, 'https://unknown.example.test')), []);
});

test('an approval request names at least one offered scope, and a resource and lifetime only when chosen', () => {
  assert.equal(helpers.applicationRequest(undefined, '', ['mail'], null).error, 'Choose an application.');
  assert.equal(helpers.applicationRequest(mail, '', [], null).error, 'Choose at least one scope.');
  // A scope the chosen resource does not cover is left out, so nothing may remain.
  assert.equal(helpers.applicationRequest(mail, 'https://mail.example.test/jmap', ['contacts'], null).error,
    'Choose at least one scope.');
  assert.deepEqual(plain(helpers.applicationRequest(mail, '', ['mail', 'contacts'], null)),
    { request: { client_id: 'jmap', scopes: ['contacts', 'mail'] } });
  assert.deepEqual(plain(helpers.applicationRequest(mail, 'https://mail.example.test/jmap', ['mail', 'contacts'], 3600)),
    { request: { client_id: 'jmap', scopes: ['mail'], resource: 'https://mail.example.test/jmap', ttl: 3600 } });
});

test('application lifetimes are only those shorter than the agent has left', () => {
  assert.deepEqual(plain(helpers.applicationLifetimes(1000 + 86400, 1000).map(([seconds]) => seconds)), [3600]);
  assert.deepEqual(plain(helpers.applicationLifetimes(1000 + 86401, 1000).map(([seconds]) => seconds)), [3600, 86400]);
  assert.deepEqual(plain(helpers.applicationLifetimes(1000 + 600, 1000)), []);
});

test('approvals separate revoked, expired and active ones, usable and newest first', () => {
  const approvals = [
    { id: 'revoked', created_at: 30, expires_at: 5000, revoked_at: 40 },
    { id: 'expired', created_at: 20, expires_at: 1000, revoked_at: null },
    { id: 'older', created_at: 10, expires_at: 5000, revoked_at: null },
    { id: 'newer', created_at: 50, expires_at: 5000, revoked_at: null },
  ];
  assert.deepEqual(approvals.map((approval) => helpers.approvalStatus(approval, 1000)), ['revoked', 'expired', 'active', 'active']);
  assert.deepEqual(plain(helpers.sortApprovals(approvals, 1000).map((approval) => approval.id)), ['newer', 'older', 'expired', 'revoked']);
});

test('the exchange names the agent credential as subject and the client id as audience', () => {
  const approval = { client_id: 'jmap', scopes: ['contacts', 'mail'], resource: null };
  assert.deepEqual(plain(helpers.exchangeParameters(approval)), [
    ['grant_type', 'urn:ietf:params:oauth:grant-type:token-exchange'],
    ['subject_token', "the agent credential's token (ri_agent_…)"],
    ['subject_token_type', 'urn:riauth:params:oauth:token-type:agent'],
    ['audience', 'jmap'],
    ['scope', 'contacts mail'],
  ]);
  const bound = helpers.exchangeParameters({ ...approval, resource: 'https://mail.example.test/jmap' });
  assert.deepEqual(plain(bound.at(-1)), ['resource', 'https://mail.example.test/jmap']);
  assert.equal(helpers.agentTokenCommand('mail-helper', { ...approval, resource: 'https://mail.example.test/jmap?a=1&b' }),
    "riauth --agent-file riauth-agent-mail-helper.json agent-token jmap --scope contacts --scope mail " +
    "--resource 'https://mail.example.test/jmap?a=1&b' --output-file jmap-token.json");
});

test('shell words are quoted only when needed, including embedded quotes', () => {
  assert.equal(helpers.shellWord('mail.read'), 'mail.read');
  assert.equal(helpers.shellWord('a b'), "'a b'");
  assert.equal(helpers.shellWord("it's"), "'it'\\''s'");
  assert.equal(helpers.shellWord('$(x)'), "'$(x)'");
});
