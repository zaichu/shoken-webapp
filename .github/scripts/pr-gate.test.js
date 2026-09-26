'use strict';

const { test } = require('node:test');
const assert = require('node:assert/strict');
const gate = require('./pr-gate.js');

const basePr = {
  body: 'Closes #1',
  headRefOid: 'abc123',
  isCrossRepository: false,
  closingIssuesReferences: { totalCount: 1 },
};

function makeGithub({ pr = basePr, runs = [], myRunId = 100, statusError = null }) {
  const calls = { statuses: [] };
  const connection = { nodes: [], pageInfo: { hasNextPage: false, endCursor: null } };
  const github = {
    graphql: async () => ({
      repository: {
        pullRequest: pr
          ? { ...pr, comments: connection, reviews: connection }
          : null,
      },
    }),
    rest: {
      actions: {
        getWorkflowRun: async ({ run_id }) => {
          assert.equal(run_id, myRunId);
          return {
            data: runs.find((r) => r.id === run_id) ?? {
              id: run_id,
              created_at: '2026-01-02T00:00:00Z',
            },
          };
        },
        listWorkflowRuns: async () => ({ data: { workflow_runs: runs } }),
      },
      repos: {
        createCommitStatus: async (args) => {
          if (statusError) throw statusError;
          calls.statuses.push(args);
        },
      },
    },
  };
  return { github, calls };
}

const makeCore = () => {
  const logs = { info: [], warning: [] };
  return {
    logs,
    info: (m) => logs.info.push(m),
    warning: (m) => logs.warning.push(m),
  };
};

const ctx = (over = {}) => ({
  repo: { owner: 'o', repo: 'r' },
  runId: 100,
  payload: { pull_request: { number: 42 } },
  ...over,
});

test('resolvePrNumber はイベント種別ごとの番号を返す', () => {
  assert.equal(
    gate.resolvePrNumber({ payload: { pull_request: { number: 7 } } }),
    7
  );
  assert.equal(
    gate.resolvePrNumber({ payload: { issue: { number: 8 } } }),
    8
  );
  assert.equal(
    gate.resolvePrNumber({ payload: { inputs: { pr: '9' } } }),
    9
  );
});

test('evaluateGate は bot・最小化・no-action・空コメントを未解決から外す', () => {
  const { problems, unresolved } = gate.evaluateGate({
    linked: true,
    nodes: [
      { author: { __typename: 'Bot', login: 'x[bot]' }, body: 'bot' },
      { author: { login: 'user' }, isMinimized: true, body: 'hidden' },
      { author: { login: 'user' }, body: 'memo <!-- no-action -->' },
      { author: { login: 'user' }, body: '   ' },
    ],
  });
  assert.equal(unresolved.length, 0);
  assert.deepEqual(problems, []);
});

test('evaluateGate は未紐づけと未解決コメントを検出する', () => {
  const { problems, description } = gate.evaluateGate({
    linked: false,
    nodes: [{ author: { login: 'user' }, body: 'c', url: 'u1' }],
  });
  assert.deepEqual(problems, [
    'Issue が紐づいていません',
    '未解決のコメントが 1 件あります',
  ]);
  assert.match(description, /u1/);
});

test('自分より後の同 PR 実行がある場合はステータスを書かない', async () => {
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-01T00:00:00Z' },
    { id: 101, display_title: 'PR gate #42', created_at: '2026-01-03T00:00:00Z' },
  ];
  const { github, calls } = makeGithub({ runs });
  const core = makeCore();
  await gate.run({ github, context: ctx(), core });
  assert.equal(calls.statuses.length, 0);
  assert.match(core.logs.info.join('\n'), /newer run/);
});

test('他 PR・過去・同名でない実行は後続とみなさない', async () => {
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-03T00:00:00Z' },
    { id: 99, display_title: 'PR gate #42', created_at: '2026-01-01T00:00:00Z' },
    { id: 102, display_title: 'PR gate #43', created_at: '2026-01-05T00:00:00Z' },
    { id: 103, display_title: 'other', created_at: '2026-01-05T00:00:00Z' },
  ];
  const { github, calls } = makeGithub({ runs });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses.length, 1);
  assert.equal(calls.statuses[0].state, 'success');
  assert.equal(calls.statuses[0].sha, 'abc123');
});

test('created_at が同じ同時実行は run id で優劣を付ける', async () => {
  const sameTime = '2026-01-02T00:00:00Z';
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: sameTime },
    { id: 99, display_title: 'PR gate #42', created_at: sameTime },
  ];
  const { github, calls } = makeGithub({ runs });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses.length, 1);

  const { github: gh2, calls: c2 } = makeGithub({
    runs: [
      { id: 100, display_title: 'PR gate #42', created_at: sameTime },
      { id: 101, display_title: 'PR gate #42', created_at: sameTime },
    ],
  });
  await gate.run({ github: gh2, context: ctx(), core: makeCore() });
  assert.equal(c2.statuses.length, 0);
});

test('紐づけなしの場合は failure ステータスを書く', async () => {
  const pr = {
    ...basePr,
    body: 'no refs',
    closingIssuesReferences: { totalCount: 0 },
  };
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-03T00:00:00Z' },
  ];
  const { github, calls } = makeGithub({ pr, runs });
  github.graphql = async (query) => {
    if (query.includes('issue(number: $n)')) {
      return { repository: { issue: null } };
    }
    return {
      repository: {
        pullRequest: {
          ...pr,
          comments: { nodes: [], pageInfo: { hasNextPage: false, endCursor: null } },
          reviews: { nodes: [], pageInfo: { hasNextPage: false, endCursor: null } },
        },
      },
    };
  };
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses[0].state, 'failure');
});

test('PR が見つからない・番号不明でも例外なく成功で終える', async () => {
  const { github, calls } = makeGithub({ pr: null });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses.length, 0);

  const { github: gh2, calls: c2 } = makeGithub({});
  const badCtx = ctx({ payload: {} });
  await gate.run({ github: gh2, context: badCtx, core: makeCore() });
  assert.equal(c2.statuses.length, 0);
});

test('fork PR のステータス書き込み失敗は warning で終える', async () => {
  const pr = { ...basePr, isCrossRepository: true };
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-03T00:00:00Z' },
  ];
  const { github, calls } = makeGithub({ pr, runs, statusError: new Error('ro') });
  const core = makeCore();
  await gate.run({ github, context: ctx(), core });
  assert.equal(calls.statuses.length, 0);
  assert.match(core.logs.warning.join('\n'), /fork PR/);
});
