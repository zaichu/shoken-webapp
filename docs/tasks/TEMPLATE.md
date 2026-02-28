# タスクテンプレート

ブランチごとに `docs/tasks/<branch-name>.md` を作成して使用する。
共通のテスト・レビュー・PR フローは `CLAUDE.md` / `.claude/rules/*.md` に従い、ここにはタスク固有の差分だけを書く。
task file はローカルの一時メモであり、作業完了または中止時に削除する。

例:
- `feature/add-login-timeout` -> `docs/tasks/feature-add-login-timeout.md`
- `fix/openapi-coverage` -> `docs/tasks/fix-openapi-coverage.md`

## タスク名

短い要約を書く。

## 目的

このタスクで達成したいことを 1-3 行で書く。

## スコープ

- やること 1
- やること 2

## 非対象

- やらないこと 1
- やらないこと 2

## 受け入れ条件

- [ ] このタスク固有の完了条件 1
- [ ] このタスク固有の完了条件 2

## 変更候補ファイル

- backend/src/...
- frontend/src/...

## タスク固有コマンド（任意）

- `cargo run --bin generate_openapi`
- `npm run generate:types`
- 手動確認の手順があれば追記する

## 進捗

- [ ] 調査
- [ ] 実装
- [ ] テスト
- [ ] PR 作成
- [ ] レビュー対応

## レビュー指摘

- 指摘が出たら追記する

## メモ

- 共通ルールに書いてある内容は繰り返さない
- 補足事項だけを書く
