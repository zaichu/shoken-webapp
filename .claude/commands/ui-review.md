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
