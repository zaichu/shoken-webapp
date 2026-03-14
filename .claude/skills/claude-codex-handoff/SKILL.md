---
name: claude-codex-handoff
description: |
  Claude から Codex に実装タスクを委譲するときの依頼文を作成して Codex に渡す。
  Use when: 実装・テスト作業を Codex に委譲したいとき。
  タスクファイルが存在する場合や、設計方針が固まった後の実装フェーズで使う。
---

# Claude Codex Handoff

## Goal
Codex が最短で実装に着手できる依頼文を作成し、`codex exec "<依頼文>"` で渡す。

## Workflow

### 1. 依頼文を組み立てる

以下の要素をすべて含める:

```markdown
## タスク概要
<1〜2文で何をするか>

## 実装対象ファイル
- `path/to/file.ts`: <何を変えるか>
- `path/to/other.rs`: <何を変えるか>

## 期待する挙動
- 変更前: <現状>
- 変更後: <期待結果>

## 制約・非対象
- <やってはいけないこと>
- <スコープ外>

## 受け入れ条件
- [ ] <検証可能な条件1>
- [ ] <検証可能な条件2>

## 確認コマンド
```bash
# フロントエンド変更がある場合
cd frontend && npm run lint && npm test && npm run build

# バックエンド変更がある場合
cd backend && cargo clippy --all-targets -- -D warnings && cargo test
```

## 完了後の作業
1. コミット・push (`git add <files>; git commit; git push`)
2. PR 作成 (`gh pr create --base main`)
```

### 2. Codex に渡す

依頼文を作成したら以下で実行:

```bash
codex exec "<依頼文をここに貼る>"
```

非対話モードで実行するため `exec` サブコマンドを必ず使う（`codex "..."` は TTY なしで失敗する）。

## Rules
- 曖昧語を避ける（「いい感じに」「必要なら」は禁止）
- ファイルパスを必ず明示する（Codex が探索コストをかけないようにする）
- 非対象を明記する（ついでの変更を防ぐ）
- 受け入れ条件を検証可能に書く（テスト名、コマンド結果を具体化）

## Output
`codex exec` に渡す依頼文（Markdown）を出力し、そのまま実行する。
