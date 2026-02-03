---
description: "開発サーバーに対してUIレビューを自動実行し、スクリーンショットとAI分析レポートを生成します。"
---
# UI Review

受取金ページのUIレビューを実施し、スクリーンショットと改善提案を作成する。  
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
| `receipts-dividend-initial.png` | 配当金タブの初期表示 |
| `receipts-domestic-stock-initial.png` | 国内株式タブの初期表示 |
| `receipts-mutualfund-initial.png` | 投資信託タブの初期表示 |
| `receipts-dividend-search-year.png` | 配当金 - 西暦検索結果 |
| `receipts-dividend-search-security.png` | 配当金 - 銘柄検索結果 |
| `receipts-domestic-stock-search-year.png` | 国内株式 - 西暦検索結果 |
| `receipts-domestic-stock-search-account.png` | 国内株式 - 口座検索結果 |
| `receipts-mutualfund-search-year.png` | 投資信託 - 西暦検索結果 |
| `receipts-mutualfund-search-fund.png` | 投資信託 - ファンド検索結果 |

## 実施手順（共通）

### 1) 事前確認
- `http://localhost:8080` が利用可能であること（本レビューは **8080固定**）
- **ログインは必須**。必ず認証状態を準備する
- このアプリのレビューは **PC表示前提**（モバイル評価は対象外）

### 2) 既存スクショを削除
```bash
rm -f .playwright-mcp/*.png
```

### 3) スクショを取得（MCPかE2Eのどちらか）

#### A. MCPで取得する場合
- 先にログインを完了してから開始する
- `http://localhost:8080/receipts` に遷移
- 各タブ/検索状態を操作し、上表と同名で `png` 保存
- 設定は `fullPage: true`

#### B. E2Eで取得する場合
```bash
cd frontend

# ログイン状態を保存（必須）
npm run ui:save-auth

# 保存した認証情報を使ってスクショ取得
npm run ui:screenshot:auth
```

※ `npm run ui:screenshot` は認証情報を読み込まないため、本レビューでは使用しない。

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
