'use strict';

const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const mod = require('./mutation-comment.js');

function writeArtifacts(t, files) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'mutants-artifacts-'));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  for (const [rel, content] of Object.entries(files)) {
    const p = path.join(dir, rel);
    fs.mkdirSync(path.dirname(p), { recursive: true });
    fs.writeFileSync(p, content);
  }
  return dir;
}

function useArtifactsDir(t, dir, { diffResult = 'success' } = {}) {
  process.env.MUTANTS_ARTIFACTS_DIR = dir;
  process.env.MUTANTS_DIFF_RESULT = diffResult;
  t.after(() => {
    delete process.env.MUTANTS_ARTIFACTS_DIR;
    delete process.env.MUTANTS_DIFF_RESULT;
  });
}

function makeGithub({ comments = [], writeError = null } = {}) {
  const calls = { created: [], updated: [] };
  const github = {
    paginate: async () => comments,
    rest: {
      issues: {
        listComments: async () => ({ data: comments }),
        createComment: async (args) => {
          if (writeError) throw writeError;
          calls.created.push(args);
          return { data: { id: 1 } };
        },
        updateComment: async (args) => {
          if (writeError) throw writeError;
          calls.updated.push(args);
          return { data: { id: args.comment_id } };
        },
      },
    },
  };
  return { github, calls };
}

function makeCore() {
  const logs = { info: [], warning: [] };
  return {
    logs,
    info: (m) => logs.info.push(m),
    warning: (m) => logs.warning.push(m),
  };
}

function ctx({ fork = false } = {}) {
  return {
    repo: { owner: 'o', repo: 'r' },
    runId: 100,
    payload: {
      pull_request: {
        number: 42,
        head: {
          repo: {
            fork,
            full_name: fork ? 'someone/r' : 'o/r',
          },
        },
        base: { repo: { full_name: 'o/r' } },
      },
    },
  };
}

test('collectResults はクレートごとの missed/timeout を集約する', (t) => {
  const dir = writeArtifacts(t, {
    'mutants-backend-diff/missed.txt': 'src/a.rs:1: x\n\nsrc/b.rs:2: y\n',
    'mutants-backend-diff/timeout.txt': 'src/c.rs:3: z\n',
    'mutants-frontend-diff/missed.txt': 'src/d.rs:4: w\n',
    'mutants-frontend-diff/timeout.txt': '',
    // 対象外の artifact 名や mutants.out 自体は無視する
    'other-artifact/missed.txt': 'src/ignore.rs:1: nope\n',
    'mutants-out/missed.txt': 'src/ignore2.rs:1: nope\n',
  });
  const results = mod.collectResults(dir);
  assert.deepEqual(
    results.map((r) => r.crate),
    ['backend', 'frontend']
  );
  const backend = results.find((r) => r.crate === 'backend');
  assert.equal(backend.missed.length, 2);
  assert.equal(backend.timeouts, 1);
  const frontend = results.find((r) => r.crate === 'frontend');
  assert.equal(frontend.missed.length, 1);
  assert.equal(frontend.timeouts, 0);
});

test('collectResults は required ならディレクトリ欠落・ファイル欠落・対象0件でエラーにする', (t) => {
  assert.throws(() => mod.collectResults('/nonexistent/dir'));

  const missingFiles = writeArtifacts(t, {
    'mutants-backend-diff/outcomes.json': '{}',
  });
  assert.throws(() => mod.collectResults(missingFiles));

  const empty = writeArtifacts(t, {});
  assert.throws(() => mod.collectResults(empty));
});

test('collectResults は required でないときだけディレクトリ欠落を空として扱う', () => {
  assert.deepEqual(
    mod.collectResults('/nonexistent/dir', { required: false }),
    []
  );
});

test('buildBody は件数・場所・marker を含み、上限を超えた分は省略する', () => {
  const results = [
    {
      crate: 'backend',
      missed: Array.from({ length: 12 }, (_, i) => `src/m.rs:${i}: desc`),
      timeouts: 0,
    },
  ];
  const body = mod.buildBody({ results, runUrl: 'https://example/run' });
  assert.ok(body.includes(mod.MARKER));
  assert.match(body, /12 件/);
  assert.match(body, /src\/m\.rs:0/);
  assert.match(body, /ほか 2 件/);
});

test('buildBody は missed 0 件では場所を出さない', () => {
  const body = mod.buildBody({
    results: [{ crate: 'backend', missed: [], timeouts: 2 }],
    runUrl: 'https://example/run',
  });
  assert.ok(body.includes(mod.MARKER));
  assert.doesNotMatch(body, /主な箇所/);
  assert.match(body, /timeout.*2 件/);
});

test('missed があり既存コメントがなければ新規作成する', async (t) => {
  const dir = writeArtifacts(t, {
    'mutants-backend-diff/missed.txt': 'src/a.rs:1: replace foo\n',
    'mutants-backend-diff/timeout.txt': '',
  });
  useArtifactsDir(t, dir);
  const { github, calls } = makeGithub();
  await mod.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.created.length, 1);
  assert.equal(calls.updated.length, 0);
  const body = calls.created[0].body;
  assert.ok(body.includes(mod.MARKER));
  assert.match(body, /src\/a\.rs:1/);
});

test('既存 marker コメントは新規作成せず更新する', async (t) => {
  const dir = writeArtifacts(t, {
    'mutants-backend-diff/missed.txt': 'src/a.rs:1: replace foo\n',
    'mutants-backend-diff/timeout.txt': '',
  });
  useArtifactsDir(t, dir);
  const { github, calls } = makeGithub({
    comments: [
      { id: 7, body: 'unrelated comment' },
      { id: 9, body: `old warning\n${mod.MARKER}` },
    ],
  });
  await mod.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.created.length, 0);
  assert.equal(calls.updated.length, 1);
  assert.equal(calls.updated[0].comment_id, 9);
});

test('missed が 0 件になった再実行では既存コメントを解消済みに更新する', async (t) => {
  const dir = writeArtifacts(t, {
    'mutants-backend-diff/missed.txt': '',
    'mutants-backend-diff/timeout.txt': '',
  });
  useArtifactsDir(t, dir);
  const { github, calls } = makeGithub({
    comments: [
      { id: 9, body: `${mod.MARKER}\n- \`backend: src/old.rs:9: old\`` },
    ],
  });
  await mod.run({ github, context: ctx(), core: makeCore() });
  assert.equal(calls.created.length, 0);
  assert.equal(calls.updated.length, 1);
  const body = calls.updated[0].body;
  assert.ok(body.includes(mod.MARKER));
  assert.doesNotMatch(body, /src\/old\.rs/);
});

test('diff 成功なのに artifacts dir が無い場合は誤報せず失敗させる', async (t) => {
  const dir = path.join(os.tmpdir(), `mutants-missing-${process.pid}`);
  useArtifactsDir(t, dir, { diffResult: 'success' });
  const { github, calls } = makeGithub({
    comments: [{ id: 9, body: `${mod.MARKER}\n- \`backend: src/old.rs:9: old\`` }],
  });
  await assert.rejects(
    mod.run({ github, context: ctx(), core: makeCore() })
  );
  // 既存の警告コメントを「0 件・解消済み」に更新してはいけない
  assert.equal(calls.created.length, 0);
  assert.equal(calls.updated.length, 0);
});

test('diff skipped なら artifact なしを正常な空として扱う', async (t) => {
  const dir = path.join(os.tmpdir(), `mutants-missing-${process.pid}`);
  useArtifactsDir(t, dir, { diffResult: 'skipped' });
  const { github, calls } = makeGithub();
  const core = makeCore();
  await mod.run({ github, context: ctx(), core });
  assert.equal(calls.created.length, 0);
  assert.equal(calls.updated.length, 0);
});

test('missed 0 件で既存コメントもなければ書き込まない', async (t) => {
  const dir = writeArtifacts(t, {
    'mutants-backend-diff/missed.txt': '',
    'mutants-backend-diff/timeout.txt': '',
  });
  useArtifactsDir(t, dir);
  const { github, calls } = makeGithub();
  const core = makeCore();
  await mod.run({ github, context: ctx(), core });
  assert.equal(calls.created.length, 0);
  assert.equal(calls.updated.length, 0);
});

test('fork PR での書き込み失敗は warning で終えてジョブを失敗させない', async (t) => {
  const dir = writeArtifacts(t, {
    'mutants-backend-diff/missed.txt': 'src/a.rs:1: x\n',
    'mutants-backend-diff/timeout.txt': '',
  });
  useArtifactsDir(t, dir);
  const { github } = makeGithub({ writeError: new Error('Resource not accessible') });
  const core = makeCore();
  await mod.run({ github, context: ctx({ fork: true }), core });
  assert.match(core.logs.warning.join('\n'), /fork PR/);
});

test('同一リポジトリ PR での通知失敗は握り潰さず再 throw する', async (t) => {
  const dir = writeArtifacts(t, {
    'mutants-backend-diff/missed.txt': 'src/a.rs:1: x\n',
    'mutants-backend-diff/timeout.txt': '',
  });
  useArtifactsDir(t, dir);
  const { github } = makeGithub({ writeError: new Error('boom') });
  await assert.rejects(
    mod.run({ github, context: ctx(), core: makeCore() }),
    /boom/
  );
});