# フロントエンドUIレビュー

## 概要

主要ページ（ホーム・銘柄検索・資産管理・取引明細）のUIレビューを実施し、スクリーンショットと改善提案を作成する。  
**MCPでもE2Eでも、やることと成果物は同じ。**

## Usage
```
/ui-review                    # 完全自動レビュー
/ui-review --screenshots-only # スクリーンショットのみ
/ui-review --review-only      # 既存画像の分析のみ
```

## 完了条件
1. `.playwright-mcp/` に対象スクリーンショットが揃っている  
2. UIレビューレポート（日本語）が出ている  
3. 改善提案に優先度（高/中/低）が付いている

## 保存先・成果物（共通）
保存先: `.playwright-mcp/`

| ファイル名 | 内容 |
|---|---|
| `home-initial.png` | ホームページの初期表示 |
| `search-nintendo-result.png` | 銘柄検索で「任天堂」を検索した結果 |
| `assetbalance-initial.png` | 資産管理ページの初期表示 |
| `assetbalance-search-security.png` | 資産管理ページの銘柄検索結果 |
| `receipts-dividend-initial.png` | 配当金タブの初期表示 |
| `receipts-domestic-stock-initial.png` | 国内株式タブの初期表示 |
| `receipts-mutualfund-initial.png` | 投資信託タブの初期表示 |
| `receipts-dividend-search-year.png` | 配当金 - 西暦検索結果 |
| `receipts-dividend-search-security.png` | 配当金 - 銘柄検索結果 |
| `receipts-domestic-stock-search-year.png` | 国内株式 - 西暦検索結果 |
| `receipts-domestic-stock-search-account.png` | 国内株式 - 口座検索結果 |
| `receipts-mutualfund-search-year.png` | 投資信託 - 西暦検索結果 |
| `receipts-mutualfund-search-fund.png` | 投資信託 - ファンド検索結果 |

## 追加であると安心なスクショ（任意）
| ファイル名 | 内容 |
|---|---|
| `login-initial.png` | ログインページの初期表示（**未ログイン時**） |
| `notfound-404.png` | 404ページ（関連リンク表示） |
| `header-user-menu.png` | ヘッダーのユーザーメニュー展開（**ログイン時**） |
| `header-delete-account-modal.png` | アカウント削除の確認モーダル（**ログイン時**） |
| `search-empty.png` | 銘柄検索の初期状態（EmptyState表示） |
| `search-invalid-code-param.png` | `?code=` が不正な場合の警告表示 |
| `assetbalance-empty.png` | 資産管理が0件のEmptyState（**ログイン時**） |
| `assetbalance-filter-empty.png` | 絞り込み0件のEmptyState（解除リンク表示） |

## 実施手順（共通）

### 1) 事前確認
- 本レビューは **8080固定**（`http://127.0.0.1:8080` を使用）
- **起動順は必ず backend → frontend**（先にバックエンドを起動し、`/health` が応答してからフロントエンドを起動）
- **ログインは必須**。認証情報は `frontend/.auth/storage-state.json` を使用する
  - Playwright の `storageState` オプションにこのパスを指定する
  - 初回保存 / 期限切れ時は `npm run ui:save-auth` で再取得する
- このアプリのレビューは **PC表示前提**（モバイル評価は対象外）
- スクショは **FHD（1920x1080）** を基準に取得する
- 開発サーバー運用は以下を厳守する
  - 8080が既に起動中なら **再利用**（新規起動しない）
  - 新規起動時は `--strictPort` を必須化（8081/8082への自動フォールバック禁止）
  - 新規起動したプロセスは `trap` で必ず停止する（残留防止）

### 2) 既存スクショを削除
```bash
rm -f .playwright-mcp/*.png
```

### 3) スクショを取得（MCPかE2Eのどちらか）

#### A. MCPで取得する場合
- 先にログインを完了してから開始する
- `http://localhost:8080/` に遷移して `home-initial.png` を保存
- `http://localhost:8080/search` で `任天堂` を検索し、`search-nintendo-result.png` を保存
- `http://localhost:8080/assetbalance` に遷移し、初期表示を `assetbalance-initial.png` として保存
- 資産管理の検索オプション（銘柄）を1つ選択し、`assetbalance-search-security.png` を保存
- `http://localhost:8080/receipts` に遷移し、各タブ/検索状態を操作して取引明細の9枚を保存
- 設定は `fullPage: true`

#### B. E2Eで取得する場合
```bash
set -euo pipefail
cd /path/to/shoken-webapp

# ログイン状態を保存（初回のみ/期限切れ時）
# cd frontend && npm run ui:save-auth

BACK_STARTED=0
if curl -sSf http://127.0.0.1:3001/health >/dev/null 2>&1; then
  echo "reuse existing backend on :3001"
else
  (cd backend && make run >/tmp/shoken-backend.log 2>&1) &
  BACK_PID=$!
  BACK_STARTED=1
fi

for i in $(seq 1 120); do
  if curl -sSf http://127.0.0.1:3001/health >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
  if [ "$i" -eq 120 ]; then
    echo "backend did not start" >&2
    tail -n 80 /tmp/shoken-backend.log >&2 || true
    exit 1
  fi
done

FRONT_STARTED=0
if curl -sSf http://127.0.0.1:8080/ >/dev/null 2>&1; then
  echo "reuse existing frontend on :8080"
else
  (
    cd frontend
    VITE_SHOKEN_WEBAPI_API_URL=http://127.0.0.1:3001 \
      npm run dev -- --host 127.0.0.1 --port 8080 --strictPort >/tmp/shoken-frontend.log 2>&1
  ) &
  FRONT_PID=$!
  FRONT_STARTED=1
fi

for i in $(seq 1 120); do
  if curl -sSf http://127.0.0.1:8080/ >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
  if [ "$i" -eq 120 ]; then
    echo "frontend did not start" >&2
    tail -n 80 /tmp/shoken-frontend.log >&2 || true
    exit 1
  fi
done

cleanup() {
  if [ "$FRONT_STARTED" -eq 1 ]; then
    kill "$FRONT_PID" >/dev/null 2>&1 || true
  fi
  if [ "$BACK_STARTED" -eq 1 ]; then
    kill "$BACK_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

# 保存した認証情報を使ってスクショ取得
(cd frontend && npm run ui:screenshot:auth)
```

※ `npm run ui:screenshot` は認証情報を読み込まないため、本レビューでは使用しない。  
※ `npm run dev` を単独でバックグラウンド起動して放置しないこと（プロセス残留の原因）。

### 4) UIレビュー（画像分析）
以下の観点で評価する:
- 視認性（文字サイズ、コントラスト、情報の階層）
- 操作性（クリック領域、状態の分かりやすさ、導線）
- 一貫性（配色、余白、コンポーネント挙動）
- データ表示の妥当性（ラベル誤り、不自然な空白、誤解を生む表現）
- アクセシビリティ（フォーカス、ラベル、可読性）

### 5) レポート出力
```markdown
## UIレビューレポート

### 総合評価
- スコア: X/10
- 概要: ...

### 優れている点
- ...

### 改善提案
| 優先度 | 画面 | 問題 | 改善案 |
|---|---|---|---|
| 高 | ... | ... | ... |

### 推奨アクション
1. ...
2. ...
```

## 運用ルール
- スクショ取得に一部失敗しても、取得できた分でレビューを継続
- 失敗ファイルは「未取得」と明記
- レポートは必ず日本語で出力
- 作業終了時に `8080/8081/8082` の不要プロセスが残っていないことを確認する
