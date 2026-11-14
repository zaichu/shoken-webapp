# Phase 4 Codex 成果物

- [基礎デザイン](foundations.md) と [共通部品](components.md)
- [トークン CSS](tokens.css)
- [基礎見本](html/foundations-preview.html) と [部品見本](html/components-preview.html)。PC表示で幅を390pxにすると、390pxのiframeにスマホ幅の見本を表示する。
- 見本本体: [基礎](html/foundations.html)、[部品](html/components.html)。CSSは `../tokens.css` を読み込む。
- PNG: `shots/` に8枚。未作成項目なし。

## 確認状況

- Playwright の PNG 撮影: 作成済み。PC/スマホ × ライト/ダークを `shots/` に保存。各 PNG は150KB以下。
- 実機スクリーンリーダー、実キーボード操作、実ブラウザのアクセシビリティ計測: 未確認。
- コントラスト: `python3 scripts/contrast.py` で両テーマの使用中22文字色/背景色ペアを再計算し、全て4.5:1以上であることを確認。
