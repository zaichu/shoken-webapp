# 撮影環境と再現手順

## モックと通信制御

- `frontend/scripts/ui-shots/data.mjs` の架空9900番台銘柄・架空口座・金額・明細を使用する。
- `frontend/scripts/ui-shots/screens.mjs` の `prepareContext()` が `/api/v1/session` と各画面 API を架空 JSON に差し替える。管理者画面は `admin: true`、ログアウト画面は `anonymous: true` (session 401) で起動する。
- 同ファイルの `installGuards()` は localhost 以外の通信を中断し、画面 API モックに一致しない `/api/` 通信も中断して記録する。
- 実 API、本番 URL、秘密情報、`.env` は使用しない。

## 実行手順

作業ディレクトリはリポジトリの `frontend/`。依存がない場合のみ `npm ci` を実行する。

```bash
cd frontend
env -u NO_COLOR trunk build --release --dist test-results/dist-phase1
node scripts/prepare-dist.mjs test-results/dist-phase1 ''
node scripts/serve-dist.mjs test-results/dist-phase1 8259
```

別ターミナルから `frontend/` で実行する。既存15系列を一度に撮り直す場合:

```bash
node scripts/ui-shots/shoot.mjs \
  --port 8259 \
  --out ../docs/design/work/phase1-codex/shots \
  --only admin-logins,admin-logins-empty,home,asset-all,asset-stock,asset-fund,receipts-dividend,receipts-stock,receipts-fund,search,settings,settings-login-history,csv,csv-asset,login
```

`shoot.mjs` は出力先ディレクトリの PNG を全消去してから選択系列を撮影する。部分撮影時は既存の成果物を退避するか、別の一時出力先を指定する。`--only` のケースは `screens.mjs` の `SCREENS` を参照。各系列は 5 viewport × light/dark。表や一覧の系列はスクロール後も保存する。管理者検索0件ケースは PC幅で検索欄へ入力し、スマホ幅では検索シートを開いて入力する。

撮影後は `Ctrl+C` で配信プロセスを止める。生成画像の 300KB 超過を確認し、必要な場合は縮小する。今回 1920px の高密度な一覧画像は Pillow の PNG 最適化後にも超過したため、段階的に縮小して上限に収めた。既存スクリプトの出力名は viewport 寸法を含むため、成果物では画面系列・状態・PC/mobile・テーマに整形し、基準外の幅だけ寸法を残した。

撮影対象の詳細と未撮影理由は [INDEX.md](shots/INDEX.md) に記録する。
