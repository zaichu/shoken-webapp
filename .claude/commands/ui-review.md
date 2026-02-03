---
description: "開発サーバーに対してUIレビューを自動実行し、スクリーンショットとAI分析レポートを生成します。"
---
# UI Review
Claude Code で完結するUIレビューシステム（Playwright MCP版）

## Usage
```
/ui-review                    # 完全自動レビュー
/ui-review --screenshots-only # スクリーンショットのみ
/ui-review --review-only      # 既存画像の分析のみ
```

## 対象画面
以下の画面をレビュー対象とする:
- `/` - ホーム画面
- `/search` - 銘柄検索
- `/assetbalance` - 保有銘柄
- `/receipts` - 受取金
- `/404` - エラーページ

## Implementation Steps

### 1. 環境チェック
1. 開発サーバー `http://localhost:8080` の起動確認
   - 起動していない場合: ユーザーに起動を促す
2. ToolSearch で `playwright` ツールをロード
   - `ToolSearch` に `playwright screenshot navigate` でツールを検索

### 2. スクリーンショット取得
1. `mcp__playwright__browser_navigate` で対象URLに遷移
2. 各対象画面について:
   - `mcp__playwright__browser_navigate` でURLに遷移（例: `http://localhost:8080/search`）
   - `mcp__playwright__browser_take_screenshot` でキャプチャ
     - `filename`: ページ名.png（例: `home.png`, `search.png`）
     - `fullPage`: true（全ページキャプチャ）
     - `type`: png
   - スクリーンショットは `.playwright-mcp/` ディレクトリに自動保存される

### 3. UI分析
Read ツールで各スクリーンショットを読み込み、以下の観点で分析:
- **視認性**: テキストサイズ、コントラスト、情報の階層
- **操作性**: ボタン配置、クリック領域、ナビゲーション
- **一貫性**: デザインパターン、カラースキーム、スペーシング
- **レスポンシブ**: モバイル対応、ブレークポイント
- **アクセシビリティ**: フォーカス状態、ラベル、セマンティクス

### 4. レポート生成
分析結果を以下の形式で出力:

```markdown
## UIレビューレポート

### 総合評価
- スコア: X/10
- 概要: ...

### 優れている点
- ...

### 改善提案
| 優先度 | 画面 | 問題 | 改善案 |
|--------|------|------|--------|
| 高 | ... | ... | ... |

### 推奨アクション
1. ...
2. ...
```

## Playwright MCP ツール使用例

```
# ツールのロード
ToolSearch: query="playwright screenshot navigate"

# ページ遷移
mcp__playwright__browser_navigate: url="http://localhost:8080"

# スクリーンショット取得
mcp__playwright__browser_take_screenshot:
  type="png"
  filename="home.png"
  fullPage=true
```

## 注意事項
- 開発サーバーが起動していない場合は起動を促す
- スクリーンショット取得に失敗した画面はスキップして続行
- レポートは日本語で出力
- 認証が必要なページはユーザーにログインを依頼する

---

## 受取金ページ詳細スクリーンショット（npm コマンド版）

上記の `/ui-review` コマンドとは別に、受取金ページの詳細なスクリーンショットを npm コマンドで取得できます。

### 違い
| 項目 | /ui-review (MCP版) | npm run (E2E版) |
|------|-------------------|-----------------|
| 実行方法 | Claude Code から `/ui-review` | ターミナルで `npm run ui:screenshot` |
| 保存先 | `.playwright-mcp/` | `frontend/screenshots/` |
| 対象 | 全ページの概要 | 受取金ページの詳細（タブ・検索） |
| 認証 | 手動ログイン依頼 | storageState で保持可能 |

### 保存先
`frontend/screenshots/` ディレクトリに保存されます（コミット可能）。

### 取得するスクリーンショット
| ファイル名 | 内容 |
|-----------|------|
| `receipts-dividend-initial.png` | 配当金タブの初期表示 |
| `receipts-domestic-stock-initial.png` | 国内株式タブの初期表示 |
| `receipts-mutualfund-initial.png` | 投資信託タブの初期表示 |
| `receipts-dividend-search-year.png` | 配当金 - 西暦検索結果 |
| `receipts-dividend-search-security.png` | 配当金 - 銘柄検索結果 |
| `receipts-domestic-stock-search-year.png` | 国内株式 - 西暦検索結果 |
| `receipts-domestic-stock-search-account.png` | 国内株式 - 口座検索結果 |
| `receipts-mutualfund-search-year.png` | 投資信託 - 西暦検索結果 |
| `receipts-mutualfund-search-fund.png` | 投資信託 - ファンド検索結果 |

### 実行コマンド

```bash
cd frontend

# 1. 開発サーバーを起動（別ターミナル）
npm run dev

# 2. スクリーンショット取得（ログイン不要の場合）
npm run ui:screenshot

# 3. ログインが必要な場合
#    3-1. ログイン状態を保存
npm run ui:save-auth
#    ブラウザが開くので手動でログイン後、ターミナルでEnterを押す

#    3-2. 保存した認証情報を使ってスクショ取得
npm run ui:screenshot:auth
```

### ファイル構成
```
frontend/
├── e2e/
│   ├── ui-screenshots.spec.ts  # スクショ取得スクリプト
│   └── save-auth.spec.ts       # ログイン状態保存スクリプト
├── screenshots/                # スクショ保存先（コミット可能）
├── .auth/                      # 認証情報（.gitignore対象）
└── playwright.config.ts        # Playwright設定
```
