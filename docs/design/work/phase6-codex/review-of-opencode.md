# Phase 6 相互レビュー: OpenCode 成果物

対象は Phase 4 の `finance-data.md`・`layout.md`・`finance.html` と、Phase 5 の `screens-status.md`・担当 HTML 9 枚・`shots/`・`data/build-opencode.mjs`・`data-check.md`。画面範囲は Phase 5 状況表の「担当分/担当外」を基準にし、画面デザインを作る OpenCode 担当と Codex 担当を分けて評価した。

## 指摘(重要度順)

### 1. [高] Phase 5 の画面 CSS がデザインシステムのトークンだけを使っていない

- **何が:** 共通 CSS・HTML に色、余白、寸法の直接指定が多数残る。Phase 5 HTML/CSS と Phase 4 HTML を対象に `style=`・16進色・数値寸法を検索すると187箇所該当した。例として `site.css:5-13` の `#fff` と `rgba(...)`、`components.css:1` の `90px`・`12px`・`8px`、`settings-opencode.html:48-69` の inline margin/width/height、`finance.html:10-50` の多数の直接値がある。
- **どこで:** `docs/design/work/phase5-opencode/html/site.css:5-13`、`components.css:1`、`settings-opencode.html:48-69`、`docs/design/work/phase4-opencode/finance.html:10-50,60`。
- **なぜ問題か:** Phase 5 の根拠である Phase 4 Codex の `tokens.css`・`components.md` と Phase 4 の「新しいトークン名を作らない」という規則に反し、色・余白の変更が画面ごとに分岐する。フォーカス色・タップ寸法も一元管理できない。
- **どう直すか:** 直接値を既存トークンに置き換え、必要な値はデザインシステム側で合意してからトークンに追加する。検索結果を再確認し、例外値が残る場合は理由を記す。

### 2. [高] Phase 5 OpenCode のトークンが Phase 4 OpenCode の金融表示ルールと対応していない

- **何が:** Phase 5 OpenCode の `tokens.css` は Phase 4 Codex の `tokens.css` とバイト単位で一致する(`cmp` で確認)。一方、Phase 4 OpenCode の `finance-data.md` は色・余白を `frontend/style/input.css` の `@theme` に対応づけ、新しいトークン名を作らないとしている(§冒頭)。Phase 5 の CSS がその規則に沿う対応を示しているかは確認できない。
- **どこで:** `docs/design/work/phase4-opencode/finance-data.md` 冒頭、`docs/design/work/phase5-opencode/html/tokens.css:1-37`。
- **なぜ問題か:** Phase 4 OpenCode の金融データ規則と Phase 5 のトークン利用との対応が不明で、実装者がどの定義を参照するか判断できない。
- **どう直すか:** Phase 4 OpenCode の規則に対する Phase 5 のトークン対応を明記し、既存定義を使うことを確認する。

### 3. [高] 金融数値に `font-mono` を指定している

- **何が:** Phase 4 OpenCode の金融データ規則は、金額・数量・率を `tabular-nums` で等幅にし、`font-mono` は使わないとしている(`finance-data.md` §1.1)。Phase 5 の金額表示コンポーネントは `font-mono` を指定している。
- **どこで:** `docs/design/work/phase4-opencode/finance-data.md:16-19`、`docs/design/work/phase5-opencode/html/components.css:34,47,69,81`。
- **なぜ問題か:** `font-mono` は数字だけでなく本文書体も等幅にし、金融表示規則に反する。
- **どう直すか:** `font-mono` を外し、本文書体のまま `tabular-nums` を使う。

### 4. [高] 危険操作の確認ダイアログにキーボード操作の設計がなく、削除確定のフォーカスも不明

- **何が:** アカウント削除確認は `role="dialog" aria-modal="true"` とタイトル紐付けはあるが、フォーカス初期位置・フォーカス移動制御・Esc で閉じる操作がない。表示は `:target` だけで行い、削除・取消リンクはいずれも同じ画面へのリンクで、確認の確定挙動も見本から分からない。
- **どこで:** `docs/design/work/phase5-opencode/html/account-opencode.html:11-12,51-56`、CSV 削除確認は `docs/design/work/phase5-opencode/html/csv-opencode.html:92-132` の各 dialog。
- **なぜ問題か:** 破壊的操作をキーボード・読み上げ利用者が安全に確認できず、実装者に必要なダイアログ挙動が伝わらない。状況表でもキーボード/フォーカス未検証と記載 (`screens-status.md`「未確認」)。
- **どう直すか:** ダイアログを開いた時のフォーカス先、Tab の範囲、閉じた後の復帰先、Esc/取消、削除ボタンでのみ確定する挙動を設計に明記し、PC/スマホのキーボード確認画像または操作記録を添える。

### 5. [中] インベントリで列挙された複数の状態が OpenCode 担当画面に反映されていない

- **何が:** Phase 1 は CSV 同期/取込の待機・取得中・取込中・急減確認・完了/失敗、設定の読込/保存中/保存失敗、端末失効確認、検索後の条件0件等を個別状態として挙げる。Phase 5 状態ボードは一般的な読込・空・再試行・部分失敗・未ログイン・権限なしを載せるだけで、CSV の取込中/失敗、設定保存失敗、端末失効/全端末ログアウトの確認・失敗状態は見当たらない。設定画面は成功メッセージのみを表示している。
- **どこで:** `docs/design/work/phase1-codex/screen-inventory.md`「ページ内画面・操作状態」「共通状態の分岐」、`docs/design/work/phase5-opencode/html/states-opencode.html:31-87`、`settings-opencode.html:60-85`。
- **なぜ問題か:** Phase 1 が要求する既存機能・エラー時の振る舞いを実装者が画面見本から確認できず、主要な設定・CSV操作の失敗時 UX が未設計のまま残る。
- **どう直すか:** 状況ボードに CSV 取込/同期/削除の処理中・失敗・急減確認、設定保存失敗、端末失効/全端末ログアウトの確認と失敗を追加する。Phase 1 の状態項目ごとに根拠ショットを対応づける。

### 6. [中] 管理者検索の「＋ 場所」が操作可能なコントロールとして表現されていない

- **何が:** 条件パネルを開くように見える `＋ 場所` は `span` で、ボタンでもリンクでもない。画面上ではチップに見えるがキーボード操作もアクセシブル名もない。
- **どこで:** `docs/design/work/phase5-opencode/html/admin-logins-opencode.html:29-34`。
- **なぜ問題か:** 検索条件追加という既存操作の入口が、見た目だけの静的ラベルになっている。操作可能かどうかを支援技術が判別できない。
- **どう直すか:** 実際に開くボタンとして表現し、開閉状態とパネルとの関連を aria で示す。PC/スマホのパネル状態もショットに含める。

### 7. [低] Phase 4 の金融データ例は Phase 5 の共通 fixtures と異なる数値セット

- **何が:** Phase 4 の例は評価額合計 ¥3,345,000、年間配当 ¥300,000 (`finance-data.md` §1)、Phase 5 では評価額 ¥28,593,148、年配当税引前 ¥3,969,907 を使う。Phase 4 の文書は共通例として合計一致を明記しているため誤計算ではないが、Phase 5 データとの連続性はなく、仕様例と画面例の照合が難しい。
- **どこで:** `docs/design/work/phase4-opencode/finance-data.md` 冒頭・§1、`docs/design/work/phase5-opencode/screens-status.md` 冒頭。
- **なぜ問題か:** 同じ画面群の設計検討で、異なるフィクスチャを前提にした例が並び、数値の再利用や視覚比較時に誤差と仕様差を区別しにくい。
- **どう直すか:** ルール例として別データを維持する場合は「Phase 5 fixtures とは別の説明用データ」と明記する。共通値の比較を意図する節では fixtures の同じ値に揃える。

## 数値・レスポンシブ確認

- Phase 5 状況表の共通値と Codex Phase 5 の表示を照合した。評価額 ¥28,593,148、取得総額 ¥27,040,660、評価損益 +¥1,552,488 は一致。配当税引前 ¥3,969,907 − 税額 ¥806,469 = 税引後 ¥3,163,438、国内株式 +¥99,017、投資信託 +¥147,282、直近配当 ¥10,957 / ¥23,905 も OpenCode の明細・生成検算値と Codex 側で一致する。OpenCode の `account-opencode.html:42-44` は税引前配当を明示している。
- `data-check.md` の配当18行、月計12か月、株式3行、投信2行の検算は記載された式・集計値と一致。生成元が Codex fixtures であることも `build-opencode.mjs` の記述と状況表で確認した。
- PC/スマホ画像では、確認できる範囲で主要金額の切れ・重なり・横はみ出しは見られなかった。確認幅は資料記載どおり 1280px/390px。1024〜1279px、実機・実ブラウザの表示は未確認で、OpenCode 自身もその旨を記録している。
- 金額に符号が付く表示は符号と色を併用している。配色のコントラスト比は実測されておらず、画像目視だけでは数値保証できない。
- 用語は `labels.rs` の管理者一覧・設定内ログイン履歴の定数と照合し、意味の取り違えにつながる表記差は見つからなかった。HTML上の定数参照は静的見本のため行われていない。

## 良い点

- Phase 5 状況表で担当分と担当外を明示し、担当外のホーム・資産管理・構成比・銘柄検索等を理由付きで切り分けている。
- 金融データの検算表が数量×単価、税引後、月小計、年計まで追え、生成データを Codex fixtures と共有している。
- 明細の PC 表とスマホ行、検索条件の PC ポップオーバー、ダーク代表、CSV削除/急減確認など主要な別状態が画像で用意されている。
- 設定の7セクション、管理者履歴、ログイン/404、アカウント削除、一般的な読込/空/失敗状態まで画面の形が揃っている。
- skip link、`focus-visible`、検索ランドマーク、見出しとの紐付け、ダイアログの aria 属性など、HTML 上のアクセシビリティ配慮が複数箇所にある。

## 全画面の網羅確認

○ = OpenCode 担当画面として見本あり、△ = 画面ありだが状態/要件の一部不足、× = OpenCode 担当で未作成。担当外は状況表に基づき「担当外」と記載。

| Phase 1 の画面/状態 | 判定 | 確認根拠 |
|---|:---:|---|
| ホーム `/` | 担当外 | Phase 5 OpenCode 状況表「担当外」(Codex 担当) |
| 銘柄検索 `/search` | 担当外 | 同上 |
| 取引明細 `/receipts` 配当・国内株式・投資信託 | ○ | `receipts-opencode.html`、配当/株式/投信 PC・SP shots |
| 取引明細 条件パネル・行詳細 | △ | 条件 PC、配当詳細 PC dark あり。SP 条件・全行の詳細挙動は画像で未確認 |
| 旧条件 URL `/receipts/conditions` | 担当外 | URL 解決のみで独立画面なし (`screen-inventory.md`) |
| CSV 管理 `/csv` 5種類 | ○ | `csv-opencode.html`、PC/SP shots |
| CSV 取得・取込・削除・急減確認 | △ | 削除確認・急減確認あり。同期/取込の各処理中・失敗状態の画像なし |
| 旧 CSV URL 群 | 担当外 | 統合画面への旧 URL alias (`screen-inventory.md`) |
| 資産管理 `/assetbalance` すべて・国内株式・投資信託 | 担当外 | Phase 5 OpenCode 状況表「担当外」 |
| 構成比 `/assetbalance/composition` | 担当外 | 同上 |
| 旧保有株詳細 URL `/assetbalance/holdings/:id` | 担当外 | 同上 |
| 投信詳細 `/assetbalance/funds/:id` | 担当外 | 同上 |
| 設定 `/settings` 7セクション | △ | `settings-opencode.html` と PC/SP/dark shots あり。保存失敗・確認状態が不足 |
| 管理者ログイン履歴 `/admin/logins` | △ | HTML/PC/SP/dark/0件状態あり。条件追加が非操作要素、AD-1コピーは担当外扱い |
| ログイン `/login` | ○ | HTML、PC/SP/dark shots |
| 404・未知パス | ○ | HTML、PC/SP/dark shots |
| アカウントメニュー | △ | PC/SP shots あり。メニュー開閉のキーボード動作・ARIA連動は未提示 |
| アカウント削除確認 | △ | PC dark shot と dialog HTML あり。フォーカス/確定操作が未設計 |
| CSV行メニュー・削除確認 | ○ | CSV PC dark shot および削除確認リンク |
| ホーム配当詳細ダイアログ | 担当外 | Phase 1 のホーム画面内状態。Codex 担当 |
| 保有株詳細ダイアログ | 担当外 | 資産管理の画面内状態。Codex 担当 |
| 検索・条件の共通パネル | △ | 明細条件はあり。管理者「＋ 場所」は操作要素でなく、他担当画面は担当外 |
| 認証/権限/読込/空/通信失敗/部分失敗 | △ | 状態ボードあり。CSV/設定/端末操作の具体的な状態が不足 |
| 端数値・長い名前・複数口座・欠測/価格更新待ち | ○ | `states-opencode.html:76-86`、PC/SP/dark shots |

## 画像確認記録

Phase 4 `shots/` の3枚と Phase 5 `shots/` の30枚、合計33枚を画像として開いて確認した。Phase 5 は全画像をコンタクトシートにして表示し、Phase 4 は各画像を表示して確認した。個別の実機表示やブラウザ操作は行っていない。
