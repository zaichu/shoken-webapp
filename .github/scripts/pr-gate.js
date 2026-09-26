'use strict';

const RUN_NAME_PREFIX = 'PR gate #';
const STATUS_CONTEXT = 'PR gate';
const WORKFLOW_FILE = 'pr-gate.yml';

function resolvePrNumber(context) {
  return (
    context.payload.pull_request?.number ??
    context.payload.issue?.number ??
    Number(context.payload.inputs?.pr)
  );
}

const prQuery = `
  query($owner: String!, $repo: String!, $pr: Int!) {
    repository(owner: $owner, name: $repo) {
      pullRequest(number: $pr) {
        body
        headRefOid
        isCrossRepository
        author {
          login
        }
        closingIssuesReferences(first: 1) {
          totalCount
        }
      }
    }
  }
`;

const listQuery = (field) => `
  query($owner: String!, $repo: String!, $pr: Int!, $cursor: String) {
    repository(owner: $owner, name: $repo) {
      pullRequest(number: $pr) {
        ${field}(first: 100, after: $cursor) {
          nodes {
            isMinimized
            body
            url
            author {
              __typename
              login
            }
          }
          pageInfo {
            hasNextPage
            endCursor
          }
        }
      }
    }
  }
`;

async function fetchAll(github, { owner, repo, prNumber }, field) {
  const nodes = [];
  let cursor = null;
  do {
    const result = await github.graphql(listQuery(field), {
      owner,
      repo,
      pr: prNumber,
      cursor,
    });
    const connection = result.repository.pullRequest[field];
    nodes.push(...connection.nodes.filter(Boolean));
    cursor = connection.pageInfo.hasNextPage
      ? connection.pageInfo.endCursor
      : null;
  } while (cursor);
  return nodes;
}

async function hasLinkedIssue(github, { owner, repo }, pr) {
  if (pr.closingIssuesReferences.totalCount > 0) return true;
  const refNumbers = [
    ...new Set(
      [...(pr.body ?? '').matchAll(/(?:Refs|Part of)\s+#(\d+)/gi)].map((m) =>
        Number(m[1])
      )
    ),
  ];
  for (const n of refNumbers) {
    try {
      const result = await github.graphql(
        `query($owner: String!, $repo: String!, $n: Int!) {
          repository(owner: $owner, name: $repo) {
            issue(number: $n) {
              state
            }
          }
        }`,
        { owner, repo, n }
      );
      if (result.repository.issue) return true;
    } catch (error) {
      // 実在しない番号や PR 番号は NOT_FOUND エラーになる → 紐づけなしとして扱う
      if (!(error.errors ?? []).every((e) => e.type === 'NOT_FOUND')) {
        throw error;
      }
    }
  }
  return false;
}

// author=null(退会済みアカウント)は bot と判定できないため人のコメントとして扱う
const isBot = (author) =>
  author?.__typename === 'Bot' || (author?.login ?? '').endsWith('[bot]');

function evaluateGate({ linked, nodes, issueLinkExempt = false }) {
  const unresolved = nodes.filter((node) => {
    const body = node.body ?? '';
    return (
      !isBot(node.author) &&
      !node.isMinimized &&
      body.trim() !== '' &&
      !body.includes('<!-- no-action -->')
    );
  });

  const problems = [];
  if (!linked) problems.push('Issue が紐づいていません');
  if (unresolved.length > 0) {
    problems.push(`未解決のコメントが ${unresolved.length} 件あります`);
  }

  // ステータス説明は 140 字制限があるため、収まる分だけ URL を連結する
  const statusParts = [...problems];
  for (const node of unresolved) {
    if ([...statusParts, node.url].join(' / ').length <= 140) {
      statusParts.push(node.url);
    }
  }
  const description =
    problems.length > 0
      ? statusParts.join(' / ')
      : issueLinkExempt
        ? 'Issue 紐づけ免除(dependabot)・未解決コメントなし'
        : 'Issue 紐づけ済み・未解決コメントなし';

  return { problems, unresolved, description };
}

// 評価も書込みも行わない状態の実行は「後続の判定」として数えない
const NON_PUBLISHING_RUN_STATES = new Set([
  'action_required',
  'cancelled',
  'skipped',
  'stale',
]);

// fork PR でステータスを書けるのは pull_request_target / workflow_dispatch の実行だけ
const WRITE_CAPABLE_EVENTS = new Set(['pull_request_target', 'workflow_dispatch']);

async function laterRunsExist(
  github,
  { owner, repo, prNumber, runId, isCrossRepository }
) {
  const { data: me } = await github.rest.actions.getWorkflowRun({
    owner,
    repo,
    run_id: runId,
  });
  const myCreatedAt = new Date(me.created_at).getTime();

  const candidates = [];
  let page = 1;
  // created で自実行以降にサーバ側絞り込みしつつ、件数が多い場合は全ページを見る
  for (;;) {
    const { data: list } = await github.rest.actions.listWorkflowRuns({
      owner,
      repo,
      workflow_id: WORKFLOW_FILE,
      created: `>=${me.created_at}`,
      per_page: 100,
      page,
    });
    candidates.push(...list.workflow_runs);
    if (list.workflow_runs.length < 100) break;
    page += 1;
  }

  const expectedName = `${RUN_NAME_PREFIX}${prNumber}`;
  const later = candidates.filter(
    (run) =>
      run.id !== runId &&
      run.display_title === expectedName &&
      !NON_PUBLISHING_RUN_STATES.has(run.conclusion ?? run.status) &&
      (!isCrossRepository || WRITE_CAPABLE_EVENTS.has(run.event)) &&
      // created_at は秒粒度のため、同刻の同時実行は run id(単調増加)で優劣を付ける
      (new Date(run.created_at).getTime() > myCreatedAt ||
        (new Date(run.created_at).getTime() === myCreatedAt && run.id > runId))
  );
  return { myCreatedAt, later };
}

async function run({ github, context, core }) {
  const { owner, repo } = context.repo;
  const prNumber = resolvePrNumber(context);
  if (!Number.isInteger(prNumber) || prNumber <= 0) {
    core.warning(`PR 番号を解決できないため判定を書きません: ${prNumber}`);
    return;
  }

  const prResult = await github.graphql(prQuery, { owner, repo, pr: prNumber });
  const pr = prResult.repository.pullRequest;
  if (!pr) {
    core.warning(`pull request #${prNumber} not found`);
    return;
  }

  // dependabot の PR は Issue 紐づけを免除する(未解決コメントの確認は通常どおり行う)
  const skipIssueLink = pr.author?.login === 'dependabot[bot]';
  const [comments, reviews, linked] = await Promise.all([
    fetchAll(github, { owner, repo, prNumber }, 'comments'),
    fetchAll(github, { owner, repo, prNumber }, 'reviews'),
    skipIssueLink ? true : hasLinkedIssue(github, { owner, repo }, pr),
  ]);

  const { problems, unresolved, description } = evaluateGate({
    linked,
    nodes: [...comments, ...reviews],
    issueLinkExempt: skipIssueLink,
  });

  core.info(`linked issue: ${linked}`);
  for (const node of unresolved) {
    core.info(`unresolved comment: ${node.url}`);
  }

  // concurrency で打ち切らない代わりに、自分より後に作られた同 PR の実行があれば書かずに終える
  const { later } = await laterRunsExist(github, {
    owner,
    repo,
    prNumber,
    runId: context.runId,
    isCrossRepository: pr.isCrossRepository,
  });
  if (later.length > 0) {
    core.info(
      `newer run(s) exist for PR #${prNumber}: ${later
        .map((r) => r.id)
        .join(', ')} — skip writing status`
    );
    return;
  }

  try {
    await github.rest.repos.createCommitStatus({
      owner,
      repo,
      sha: pr.headRefOid,
      state: problems.length > 0 ? 'failure' : 'success',
      context: STATUS_CONTEXT,
      description: description.slice(0, 140),
      target_url: `${process.env.GITHUB_SERVER_URL}/${owner}/${repo}/actions/runs/${context.runId}`,
    });
  } catch (error) {
    // fork PR へのコメント/レビューイベントでは write が read に降格される。ステータスは pull_request_target か手動再実行で付け直す
    if (!pr.isCrossRepository) throw error;
    core.warning(`commit status not written for fork PR: ${error.message}`);
  }
}

module.exports = {
  RUN_NAME_PREFIX,
  STATUS_CONTEXT,
  WORKFLOW_FILE,
  NON_PUBLISHING_RUN_STATES,
  WRITE_CAPABLE_EVENTS,
  resolvePrNumber,
  evaluateGate,
  hasLinkedIssue,
  laterRunsExist,
  run,
};
