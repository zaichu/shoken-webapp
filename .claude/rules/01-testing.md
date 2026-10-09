# テスト共通ルール

## テスト原則

- 本番データをテストに使用しない
- 日本語テストデータ（銘柄名など）を適切に使用
- テストとテストデータの置き場所は `frontend/README.md`(フロント)と `backend/CLAUDE.md`(バックエンド)の決まりに従う
- ローカルでフロントエンドとバックエンドを同時に使う検証は、**必ず DB → バックエンド → フロントエンドの順で起動**する

## 何をテストするか

開発の速度を落とさないため、テストは挙動に絞る。

- テストを書くのは、計算・分岐・エラー時の扱い・データの受け渡し
- 見た目の値(列幅・色・余白・クラス名・文言)を完全一致で固定するテストは書かない。必要なら「はみ出さない・重ならない・表示される・操作できる」といった利用者から見た性質で確かめる

## 手元の確認と CI の分担

- 手元の確認は `cargo fmt`・`cargo clippy`(-D warnings)・触ったクレートの `cargo test`・`npm --prefix frontend run check:css` まで(API 契約を変えたときは `bash scripts/check-openapi.sh` も)。Cargo には `env RUSTC_WRAPPER=sccache` を付ける
- E2E は手元で全件を回さず、CI の分割実行に任せる。CI で落ちたら、落ちた spec だけ手元で直して確かめる(`env LEPTOS_E2E_PORT=<ほかと重ならない番号>` を付ける)
- PR を出したら CI の完了を待たずに終えてよい。CI の待ちとマージは統合担当が行う
- 変更前後の画面の画像は、Issue で求められたときだけ撮る。撮影用の spec はコミットしない

## ローカル起動順ルール

ローカルデバッグ / E2E / UIレビューでは起動順を固定する:

1. ローカルDB起動（例: `cd backend && make db-up`）
2. バックエンド起動（`cd backend && make run` = wrangler dev、`http://127.0.0.1:8787/ready` が200になるまで待つ）
3. フロントエンド起動（バックエンドURLをローカル向けに指定）

推奨: プロジェクトルートで `./scripts/start-local.sh` を実行すると、上記 1-3 をまとめて起動できる。

## CI/CD でのテスト

GitHub Actions でのテスト実行:

```yaml
- name: Run frontend tests
  working-directory: frontend
  run: cargo test

- name: Run backend tests
  working-directory: backend
  run: cargo test
```
