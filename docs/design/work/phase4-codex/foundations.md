# 基礎デザイン

## 採用する骨格

Phase 3 の採用方針に従い、C の上部ナビゲーションと要点の集計カードを全体の骨格にする。スマホは B の1列構成・大きな数字・下部ナビゲーションを優先し、PC の資産管理と取引明細には A の一覧と右詳細パネルを採用する。縦ナビは作らない。根拠: [採用決定](../../../../../.requests/design-decision-p3.md)、[C の見本](../../../phase3-codex/shots/concept-c-home-pc-light.png)、[B のスマホ見本](../../../../../../shoken-webapp-1890-p3-opencode/docs/design/work/phase3-opencode/shots/B-home-sp.png)、[A の資産見本](../../../phase3-codex/shots/concept-a-asset-pc.png)。

## 色

紺 `--brand` は上部ナビと集計帯、橙 `--brand-accent` は選択位置や強調の印に用いる。本文は明るい面に濃い文字を置く。色に意味がある増減は符号を併記し、状態は文字ラベルとアイコン形状も併用する。成功・注意・エラー・情報には専用色と薄い背景を組み合わせる。チャート色は系列に固定割当てし、テーマが変わっても同じ系列を表す。

コントラストは WCAG 相対輝度計算による。表の比率は通常文字の基準 4.5:1 を満たす。

| テーマ | 文字色 / 背景色 | 比率 | 用途 |
|---|---|---:|---|
| 明 | `#111827` / `#ffffff` | 17.74:1 | 本文 / 面 |
| 明 | `#5f6b7a` / `#ffffff` | 5.43:1 | 補助文字 / 面 |
| 明 | `#0f2a4a` / `#ffffff` | 14.48:1 | 紺文字 / 面 |
| 明 | `#1d4ed8` / `#ffffff` | 6.70:1 | 強調リンク / 面 |
| 明 | `#0b7a4b` / `#ffffff` | 5.39:1 | 増加 / 面 |
| 明 | `#c0362c` / `#ffffff` | 5.52:1 | 減少 / 面 |
| 明 | `#92400e` / `#fff8e6` | 6.69:1 | 注意 / 注意面 |
| 明 | `#ffffff` / `#0f2a4a` | 14.48:1 | 紺ナビ / 紺面 |
| 明 | `#f2b33d` / `#0f2a4a` | 7.78:1 | 橙アクセント / 紺面 |
| 暗 | `#f2f5fa` / `#121b2c` | 15.77:1 | 本文 / 面 |
| 暗 | `#c2cddf` / `#121b2c` | 10.74:1 | 補助文字 / 面 |
| 暗 | `#a9c4fa` / `#121b2c` | 9.82:1 | 強調リンク / 面 |
| 暗 | `#7ce8ab` / `#121b2c` | 11.48:1 | 増加 / 面 |
| 暗 | `#f5a49b` / `#121b2c` | 8.75:1 | 減少 / 面 |
| 暗 | `#f2c066` / `#292112` | 9.47:1 | 注意 / 注意面 |

紺面の上の白文字は 14.48:1。フォーカスリングは明テーマ `#1d4ed8`、暗テーマ `#a9c4fa` を使い、3px 幅で表示する。罫線やグラフ記号など非テキストの識別では 3:1 を確保する。

## 文字と数値

日本語書体は Noto Sans JP を先頭に置き、未導入環境ではシステム日本語書体へフォールバックする。本文 16px、補助 14px、小文字 12px、見出しは 18/20/24/30px の段階を使い、太さは 400/500/700、行間は 1.25/1.5/1.7 とする。金額・数量・率は `Amount` と `format.rs` の書式を正本とし、右揃え、桁区切り、全桁、等幅数字を使う。表示単位の選択は `format.rs` の既存規則に従い、独自に省略しない。損益等は符号と色、必要に応じた語句を併用する。

## 余白・角丸・影

4px 基準の `--space-*` を使う。スマホ左右余白は 16px。角丸は小 4px、標準 8px、大 12px、ピル形とする。影はカードと重ね合わせ面に限定し、境界線を消す用途にしない。

## グリッドと幅

1024px を `lg` 境界とする。スマホは1列で本文左右16px、PC 本文は最大1440px、12列・段間24px。1024px 未満は1列へ切り替える。1280px 以上の資産一覧・取引明細は右側詳細パネルを加え、本文一覧の読みやすさを損なわない。

## ナビゲーション

PC は紺の上部ナビに主要4画面を配置し、アカウントメニューを右端に置く。スマホは主要4画面の下部ナビを常時表示する。アカウントメニューは上部右端から開くが、フォーカス移動と閉じる操作を持たせる。既存対応: `ui/site.rs` `BottomNav`、`ui/page_frame.rs` `PageFrame`、`ui/avatar.rs`。フォーカス・メニューの全操作は見本ページ上の静的例示に留まる。

## 現行トークンとの対応

出典: `frontend/style/input.css:5-154,220-300`。

| 現行名 | 新名 |
|---|---|
| `page` | `--page` |
| `surface` | `--surface` |
| `surface-sunken` | `--surface-sunken` |
| `surface-raised` | `--surface-raised` |
| `border` / `border-strong` | `--border` / `--border-strong` |
| `text-deep` / `text-muted` | `--text` / `--text-muted` |
| `brand` / `brand-deep` | `--brand` / `--brand-deep` |
| `gold` | `--brand-accent` / `--gold` |
| `accent-text` | `--accent` |
| `positive` / `negative` | `--positive` / `--negative` |
| `info` / `warn-ink` | `--info` / `--warning` |
| `negative-soft` / `positive-soft` | `--negative-soft` / `--positive-soft` |
| `band` | `--warning-soft` (注意面用途に限定) |
| `font-sans` | `--font-sans` |
| `text-eyebrow` / `text-cell` / `text-summary-*` | `--text-xs` / `--text-xs` / `--text-3xl` |
| `space-*` | 対応する `--space-*`。段間専用トークンは個別に維持 |
| `radius-note` / `rounded-lg` / `rounded-xl` | `--radius-lg` / `--radius-md` / `--radius-lg` |
| `shadow-card` / `shadow-elevation-2` | `--shadow-card` / `--shadow-overlay` |
| `container-page` | `--content-max` |
| 新設 | `--font-mono`, `--weight-*`, `--leading-*`, `--space-1`〜`--space-16`, `--radius-sm`, `--radius-pill`, `--breakpoint-lg`, `--grid-columns`, `--grid-gap`, `--mobile-gutter`, `--success`, `--error`, `--on-brand`, `--on-warning`, `--focus` |

本ファイルは Phase 4 の提案トークンで、本番 CSS を変更しない。現行トークンの詳細定義は `frontend/style/input.css:5-154,220-300`、画面骨格は `docs/design/mobile-ui-rules.md:1-76` を参照。
