# CLAUDE.md

このファイルはプロジェクト全体のガイド（正本）です。

## 読み込み場所

- プロジェクト全体: `./CLAUDE.md`
- 個人設定: `./CLAUDE.local.md`（gitignore 推奨）
- サブディレクトリ作業: `backend/CLAUDE.md`, `frontend/CLAUDE.md`
- 全プロジェクト共通のルール（agent-config が正本。必ず適用）: `~/.claude/CLAUDE.md`

## タスク管理

- ブランチ単位の task file は `docs/tasks/<branch-name>.md` を使用する
- task file はローカルの一時メモとして扱い、ユーザーの明示指示がない限りコミットや PR に含めない
- task file が存在する場合は、作業開始前に必ず読む
- task file が存在する場合は、進捗・受け入れ条件・レビュー指摘を作業に合わせて更新する
- 新しい中規模以上のタスクでは `docs/tasks/TEMPLATE.md` を元に task file を作成する
- task file がある場合は、その内容を優先してスコープと非対象を守る
- task 完了時または作業中止時には、対応する task file を削除する

## 設計方針

- API 契約の正本は `docs/openapi.json` とし、backend の API 変更時は `frontend/src/generated/api.ts` まで必ず同期する
- CSV 取り込みは原則 backend で `parse / validate / import` する。frontend はファイル送信と結果表示を優先する
- unrelated な修正は同じブランチに混在させない。`1 ブランチ = 1 タスク` を守る

## Agent Assets

- 全プロジェクト共通の skill は `~/.agents/skills` を正本とする
- repo 内の agent 設定とプロジェクト固有 skill の正本は `./.claude` とする
- `./.agents` と `./.codex` は `./.claude` を指す symlink として維持する
- symlink が壊れた場合は `./scripts/repair-agent-links.sh` を実行して復旧する

## エージェント運用

- どのエージェントがどの役割（実装/設計/統合）を担うかの唯一の正本は `docs/agent-roles.md`
- 委譲の手順・依頼テンプレート・受け取り時の検証は `agent-delegation` skill（全プロジェクト共通）
- 複数エージェントが同時に動き得る場合の重複防止・git状態分離は `parallel-agent-coordination` skill
  - **実装を委譲する前（spawn/依頼プロンプトを渡す前）に、必ず対象Issueへ着手を宣言する**
    （`~/.claude/skills/parallel-agent-coordination/scripts/claim-issue.sh`）。規模を問わず省略しない
- Issue振り分けの補助（Jev）は `issue-triage` skill（全プロジェクト共通）。**実装を委譲する前（着手宣言の前）に必ず回す。** 判定の使い方は `docs/agent-roles.md` の当該セクションを参照。結果は提案であり決定ではない

## 参照ルール

1. `.claude/rules/00-general.md`
2. `.claude/rules/01-testing.md`
3. `.claude/rules/02-security.md`
4. `.claude/rules/03-git.md`
5. UIレビュー時のみ `.claude/rules/04-frontend-ui-review.md`

## サブプロジェクトルール

- バックエンド作業時は `backend/CLAUDE.md` を追加で適用
- フロントエンド作業時は `frontend/CLAUDE.md` を追加で適用
