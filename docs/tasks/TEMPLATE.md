# タスクテンプレート

ブランチごとに `docs/tasks/<branch-name>.md` を作成して使用する。

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

- [ ] 条件 1
- [ ] 条件 2

## 変更候補ファイル

- backend/src/...
- frontend/src/...

## 実行コマンド

- cargo test
- cargo clippy -- -D warnings
- npm test
- npm run build

## 進捗

- [ ] 調査
- [ ] 実装
- [ ] テスト
- [ ] PR 作成
- [ ] レビュー対応

## レビュー指摘

- 指摘が出たら追記する

## メモ

- 補足事項を書く
