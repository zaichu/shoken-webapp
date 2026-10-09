'use strict';

const fs = require('node:fs');
const path = require('node:path');

const MARKER = '<!-- mutation-testing-missed-mutants -->';
const MAX_LOCATIONS = 10;
const MAX_LOCATION_LENGTH = 200;

// missed.txt / timeout.txt は diff ジョブで必ず作る設計なので、
// 読めない場合は握り潰さず artifact 異常として失敗させる
function readLines(file) {
  return fs
    .readFileSync(file, 'utf8')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean);
}

// download-artifact の出力は mutants-<crate>-<shard>-diff/missed.txt の形になる。
// missed.txt の行は PR のソース由来の文字列なので、データとして扱い実行しない。
// required のとき、dir 自体が読めない・対象 artifact が0件・必須ファイル欠落は
// すべてエラーにする(0 件と誤報して古い警告を消さないため)
function collectResults(artifactsDir, { required = true } = {}) {
  const results = [];
  let entries;
  try {
    entries = fs.readdirSync(artifactsDir, { withFileTypes: true });
  } catch (error) {
    if (!required) return results;
    throw new Error(
      `cannot read artifacts dir ${artifactsDir}: ${error.message}`
    );
  }
  // 集約用: crate ごとに missed / timeout をマージ
  const byCrate = new Map();
  for (const entry of entries) {
    const match = /^mutants-(.+)-\d+-diff$/.exec(entry.name);
    if (!entry.isDirectory() || !match) continue;
    const crate = match[1];
    const dir = path.join(artifactsDir, entry.name);
    const missed = readLines(path.join(dir, 'missed.txt'));
    const timeouts = readLines(path.join(dir, 'timeout.txt')).length;
    if (!byCrate.has(crate)) {
      byCrate.set(crate, { crate, missed: [], timeouts: 0 });
    }
    const agg = byCrate.get(crate);
    agg.missed.push(...missed);
    agg.timeouts += timeouts;
  }
  if (required && byCrate.size === 0) {
    throw new Error(`no mutants-*-*-diff artifacts found in ${artifactsDir}`);
  }
  return Array.from(byCrate.values()).sort((a, b) => a.crate.localeCompare(b.crate));
}

function truncate(text) {
  const clean = text.replace(/`/g, "'");
  return clean.length > MAX_LOCATION_LENGTH
    ? `${clean.slice(0, MAX_LOCATION_LENGTH)}…`
    : clean;
}

function buildBody({ results, runUrl }) {
  const totalMissed = results.reduce((n, r) => n + r.missed.length, 0);
  const totalTimeouts = results.reduce((n, r) => n + r.timeouts, 0);
  const lines = [MARKER, '', '## Mutation testing (PR diff)', ''];
  if (totalMissed === 0) {
    lines.push(
      '最新の実行では missed mutant は **0 件**でした。以前の指摘は解消されています。'
    );
  } else {
    lines.push(
      `cargo-mutants が **${totalMissed} 件**の missed mutant(テストで検出されなかったミュータント)を報告しました。`
    );
    const locations = results.flatMap((r) =>
      r.missed.map((line) => `${r.crate}: ${line}`)
    );
    lines.push('', '主な箇所:', '');
    for (const loc of locations.slice(0, MAX_LOCATIONS)) {
      lines.push(`- \`${truncate(loc)}\``);
    }
    if (locations.length > MAX_LOCATIONS) {
      lines.push(`- ほか ${locations.length - MAX_LOCATIONS} 件`);
    }
  }
  if (totalTimeouts > 0) {
    lines.push('', `timeout も ${totalTimeouts} 件あります。`);
  }
  lines.push(
    '',
    '全件は artifact `mutants-<crate>-<shard>-diff` の `missed.txt`・`timeout.txt` を参照してください。'
  );
  if (runUrl) lines.push(`実行: ${runUrl}`);
  return lines.join('\n');
}

async function findMarkedComment(github, { owner, repo, issueNumber }) {
  const comments = await github.paginate(github.rest.issues.listComments, {
    owner,
    repo,
    issue_number: issueNumber,
    per_page: 100,
  });
  return comments.find((c) => (c.body ?? '').includes(MARKER)) ?? null;
}

async function run({ github, context, core }) {
  const pr = context.payload.pull_request;
  if (!pr) {
    core.warning('pull_request イベントではないためスキップします');
    return;
  }
  const { owner, repo } = context.repo;
  const artifactsDir = path.resolve(
    process.env.GITHUB_WORKSPACE ?? '.',
    process.env.MUTANTS_ARTIFACTS_DIR ?? 'mutants-artifacts'
  );
  // diff が skipped(対象変更なし)のときだけ空を正常系とする。
  // success/その他(未設定を含む)は artifact 欠落を異常として失敗させる
  const expectArtifacts = process.env.MUTANTS_DIFF_RESULT !== 'skipped';
  const results = collectResults(artifactsDir, { required: expectArtifacts });
  const totalMissed = results.reduce((n, r) => n + r.missed.length, 0);

  const runUrl = `${process.env.GITHUB_SERVER_URL}/${owner}/${repo}/actions/runs/${context.runId}`;
  const body = buildBody({ results, runUrl });

  const isCrossRepository =
    pr.head?.repo?.fork === true ||
    (pr.head?.repo?.full_name &&
      pr.head.repo.full_name !== pr.base?.repo?.full_name);

  try {
    const existing = await findMarkedComment(github, {
      owner,
      repo,
      issueNumber: pr.number,
    });
    if (existing) {
      // missed が 0 件でも必ず更新して古い警告を残さない
      await github.rest.issues.updateComment({
        owner,
        repo,
        comment_id: existing.id,
        body,
      });
      core.info(`updated missed-mutants comment ${existing.id}`);
    } else if (totalMissed > 0) {
      await github.rest.issues.createComment({
        owner,
        repo,
        issue_number: pr.number,
        body,
      });
      core.info('created missed-mutants comment');
    } else {
      core.info('no missed mutants and no existing comment');
    }
  } catch (error) {
    // fork PR では GITHUB_TOKEN が read-only に降格され書き込めない。
    // それ以外の通知失敗は握り潰さず失敗として表面化する
    if (isCrossRepository) {
      core.warning(
        `fork PR ではコメントを書けないため通知をスキップ: ${error.message}`
      );
      return;
    }
    throw error;
  }
}

module.exports = {
  MARKER,
  MAX_LOCATIONS,
  collectResults,
  buildBody,
  findMarkedComment,
  run,
};