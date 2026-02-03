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

## 取得するスクリーンショット

受取金ページの各タブと検索結果を取得する（MCP版・E2E版共通）:

| ファイル名 | 内容 |
|-----------|------|
| `receipts-dividend.png` | 配当金タブの初期表示 |
| `receipts-dividend-search.png` | 配当金 - 銘柄検索結果（配当シミュレーション表示） |
| `receipts-domestic-stock.png` | 国内株式タブの初期表示 |
| `receipts-mutualfund.png` | 投資信託タブの初期表示 |

### 保存先
`.playwright-mcp/` ディレクトリに保存される。

## Implementation Steps

### 1. 環境チェック
1. 開発サーバー `http://localhost:8080` の起動確認
   - 起動していない場合: ユーザーに起動を促す
2. ToolSearch で `playwright` ツールをロード
   - `ToolSearch` に `playwright screenshot navigate` でツールを検索

### 2. スクリーンショット取得
1. `.playwright-mcp/` ディレクトリ内の既存画像を削除
   ```bash
   rm -f .playwright-mcp/*.png
   ```
2. `mcp__playwright__browser_navigate` で `http://localhost:8080/receipts` に遷移
3. 各タブについてスクリーンショットを取得:
   - 配当金タブ: `receipts-dividend.png`
   - 配当金タブで銘柄検索: `receipts-dividend-search.png`
   - 国内株式タブ: `receipts-domestic-stock.png`
   - 投資信託タブ: `receipts-mutualfund.png`
4. スクリーンショット取得時の設定:
   - `fullPage`: true（全ページキャプチャ）
   - `type`: png

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
mcp__playwright__browser_navigate: url="http://localhost:8080/receipts"

# スクリーンショット取得
mcp__playwright__browser_take_screenshot:
  type="png"
  filename="receipts-dividend.png"
  fullPage=true
```

## 注意事項
- 開発サーバーが起動していない場合は起動を促す
- スクリーンショット取得に失敗した画面はスキップして続行
- レポートは日本語で出力
- 認証が必要なページはユーザーにログインを依頼する

---

## E2E版（npm コマンド）

MCP版と同じスクリーンショットを npm コマンドでも取得できる。

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
shoken-webapp/
├── .playwright-mcp/              # スクショ保存先（共通）
└── frontend/
    ├── e2e/
    │   ├── ui-screenshots.spec.ts  # スクショ取得スクリプト
    │   └── save-auth.spec.ts       # ログイン状態保存スクリプト
    ├── .auth/                      # 認証情報（.gitignore対象）
    └── playwright.config.ts        # Playwright設定
```
