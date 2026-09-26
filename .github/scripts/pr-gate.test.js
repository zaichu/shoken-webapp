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

function makeGithub({
  pr = basePr,
  runs = [],
  myRunId = 100,
  statusError = null,
  nodes = [],
}) {
  const calls = { statuses: [] };
  const connection = {
    nodes,
    pageInfo: { hasNextPage: false, endCursor: null },
  };
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
        listWorkflowRuns: async ({ created, per_page = 100, page = 1 }) => {
          const since = created?.startsWith('>=')
            ? new Date(created.slice(2)).getTime()
            : -Infinity;
          const filtered = runs.filter(
            (r) => new Date(r.created_at).getTime() >= since
          );
          const start = (page - 1) * per_page;
          return {
            data: { workflow_runs: filtered.slice(start, start + per_page) },
          };
        },
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

test('後続実行が2ページ目にある場合も見つける', async () => {
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-02T00:00:00Z' },
    ...Array.from({ length: 150 }, (_, i) => ({
      id: 200 + i,
      display_title: 'PR gate #99',
      created_at: '2026-01-03T00:00:00Z',
    })),
    { id: 400, display_title: 'PR gate #42', created_at: '2026-01-04T00:00:00Z' },
  ];
  const { github, calls } = makeGithub({ runs });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses.length, 0);
});

test('cancelled・action_required の後続実行は後続とみなさない', async () => {
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-02T00:00:00Z' },
    {
      id: 101,
      display_title: 'PR gate #42',
      created_at: '2026-01-03T00:00:00Z',
      status: 'completed',
      conclusion: 'cancelled',
    },
    {
      id: 102,
      display_title: 'PR gate #42',
      created_at: '2026-01-04T00:00:00Z',
      status: 'action_required',
      conclusion: null,
    },
  ];
  const { github, calls } = makeGithub({ runs });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses.length, 1);
});

test('fork PR では書込み可能なイベントの後続実行だけを数える', async () => {
  const pr = { ...basePr, isCrossRepository: true };
  const commentRun = {
    id: 101,
    display_title: 'PR gate #42',
    created_at: '2026-01-03T00:00:00Z',
    event: 'issue_comment',
    status: 'completed',
    conclusion: 'success',
  };
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-02T00:00:00Z' },
    commentRun,
  ];
  const { github, calls } = makeGithub({ pr, runs });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses.length, 1);

  const runs2 = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-02T00:00:00Z' },
    commentRun,
    {
      id: 102,
      display_title: 'PR gate #42',
      created_at: '2026-01-04T00:00:00Z',
      event: 'pull_request_target',
      status: 'in_progress',
      conclusion: null,
    },
  ];
  const { github: gh2, calls: c2 } = makeGithub({ pr, runs: runs2 });
  await gate.run({ github: gh2, context: ctx(), core: makeCore() });
  assert.equal(c2.statuses.length, 0);
});

test('ワークフローの run-name が RUN_NAME_PREFIX 付きでクォートされている', () => {
  const fs = require('node:fs');
  const path = require('node:path');
  const yamlText = fs.readFileSync(
    path.join(__dirname, '..', 'workflows', gate.WORKFLOW_FILE),
    'utf8'
  );
  const line = yamlText.match(/^run-name:\s*(.+)$/m);
  assert.ok(line, 'run-name が見つかる');
  const m = line[1].trim().match(/^"(.*)"$|^'(.*)'$/);
  assert.ok(m, 'run-name はクォート必須(" #" はコメント開始になる)');
  assert.ok((m[1] ?? m[2]).startsWith(gate.RUN_NAME_PREFIX));
});

// GraphQL の Actor.login は "dependabot" を返す(REST の "dependabot[bot]" と異なる)
test('dependabot の PR は Issue 紐づけなしでも成功になる', async () => {
  const pr = {
    ...basePr,
    author: { __typename: 'Bot', login: 'dependabot' },
    body: 'Bumps foo from 1.0.0 to 1.0.1',
    closingIssuesReferences: { totalCount: 0 },
  };
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-02T00:00:00Z' },
  ];
  const { github, calls } = makeGithub({ pr, runs });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses.length, 1);
  assert.equal(calls.statuses[0].state, 'success');
  assert.match(calls.statuses[0].description, /紐づけ免除/);
});

test('dependabot の PR でも未解決コメントは検出する', async () => {
  const pr = {
    ...basePr,
    author: { __typename: 'Bot', login: 'dependabot' },
    body: 'Bumps foo from 1.0.0 to 1.0.1',
    closingIssuesReferences: { totalCount: 0 },
  };
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-02T00:00:00Z' },
  ];
  const nodes = [{ author: { login: 'user' }, body: 'c', url: 'u1' }];
  const { github, calls } = makeGithub({ pr, runs, nodes });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses[0].state, 'failure');
  assert.match(calls.statuses[0].description, /未解決のコメント/);
});

test('dependabot という login の人間アカウントは免除しない', async () => {
  const pr = {
    ...basePr,
    author: { __typename: 'User', login: 'dependabot' },
    body: 'no refs',
    closingIssuesReferences: { totalCount: 0 },
  };
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-02T00:00:00Z' },
  ];
  const { github, calls } = makeGithub({ pr, runs });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses[0].state, 'failure');
  assert.match(calls.statuses[0].description, /Issue が紐づいていません/);
});

test('REST 形式の dependabot[bot] login でも免除される', async () => {
  const pr = {
    ...basePr,
    author: { login: 'dependabot[bot]' },
    body: 'Bumps foo from 1.0.0 to 1.0.1',
    closingIssuesReferences: { totalCount: 0 },
  };
  const runs = [
    { id: 100, display_title: 'PR gate #42', created_at: '2026-01-02T00:00:00Z' },
  ];
  const { github, calls } = makeGithub({ pr, runs });
  await gate.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.statuses[0].state, 'success');
  assert.match(calls.statuses[0].description, /紐づけ免除/);
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
