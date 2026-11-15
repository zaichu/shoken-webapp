# Phase 6 相互レビュー: OpenCode 成果物

対象は Phase 4 の `finance-data.md`・`layout.md`・`finance.html` と、Phase 5 の `screens-status.md`・担当 HTML 9 枚・`shots/`・`data/build-opencode.mjs`・`data-check.md`。画面範囲は Phase 5 状況表の「担当分/担当外」を基準にし、画面デザインを作る OpenCode 担当と Codex 担当を分けて評価した。

## 指摘(重要度順)

### 1. [高] 共通トークンのある色・余白が画面 CSS に直接指定されている

- **何が:** 共通 CSS・HTML に、既存トークンで表せる色・余白の直接指定が残る。`site.css:5-13` の `#fff` は `--on-brand`、`components.css:1` の padding `12px`・`8px` は `--space-3`・`--space-2`、`settings-opencode.html:48-69` の margin `16px`・`4px`・`12px` は `--space-4`・`--space-1`・`--space-3` に対応する。Phase 4 の `finance.html:10-50,60` にも余白・文字サイズの直接値がある。数値寸法の検索結果だけでは、トークン化すべき値と例外は判定できない。
- **どこで:** `docs/design/work/phase5-opencode/html/site.css:5-13`、`components.css:1`、`settings-opencode.html:48-69`、`docs/design/work/phase4-opencode/finance.html:10-50,60`。
- **なぜ問題か:** Phase 5 の `site.css:1` が掲げる「色・余白・文字は tokens.css の変数だけを使う」という方針と一致せず、共有トークンを変更しても直接指定した箇所が追従しない。Phase 4 の見本も余白の変更が個別になる。
- **どう直すか:** 既存トークンのある色・余白・文字サイズを変数参照に置き換える。既存トークンで表せない値は必要性を確認し、例外として残す理由か、デザインシステム側での追加判断を記す。

### 2. [高] Phase 4 OpenCode 文書・見本と Phase 5 の共有トークンの対応が未整理

- **何が:** `cmp docs/design/work/phase4-codex/tokens.css docs/design/work/phase5-opencode/html/tokens.css` は終了コード0で、両ファイルはバイト単位で一致する。一方、Phase 4 OpenCode の `layout.md:133` と `tokens.css:1-4` は Codex トークン未作成として現行 CSS の抜粋を使い、後で突き合わせるとしている。抜粋の `--color-page` 等と共有トークンの `--page` 等は名前が異なり、ダークの正数色も `--color-positive: #53d28a` と `--positive: #7ce8ab` で異なる。Phase 5 は両方を根拠に挙げるが、この差分の整理がない(`screens-status.md:4`)。
- **どこで:** `docs/design/work/phase4-opencode/layout.md:133`、`tokens.css:1-7,59`、`finance-data.md:4`、`docs/design/work/phase4-codex/tokens.css:3,24`、`docs/design/work/phase5-opencode/html/tokens.css:3,24`。
- **なぜ問題か:** Phase 5 は Codex の共有トークンを使っているが、Phase 4 OpenCode の規則・見本との対応が未整理で、実装時に適用する名前・値が曖昧になる。
- **どう直すか:** Phase 5 が Phase 4 Codex の共有トークンを使っている事実を前提に、Phase 4 OpenCode 文書・見本との差分と対応を整理する。書体の金融表示ルール違反は、次の指摘で共有 CSS 側も含めて扱う。

### 3. [高] 共有 CSS と Phase 5 の金額表示が金融表示ルールに反して等幅書体を指定している

- **何が:** Phase 4 OpenCode の金融データ規則は、金額・数量・率を本文書体の `tabular-nums` でそろえ、`font-mono` は使わないとしている。`grep -nE -- '--font-mono|font-family|tabular-nums'` で、Phase 4 Codex と Phase 5 OpenCode の両 `tokens.css` に共通の `--font-mono` 定義(9行目)と `.numeric` の書体指定(34行目)を確認した。Phase 5 OpenCode の `site.css` も集計帯・表・スマホ行・詳細の金額に同じ書体を使う。OpenCode 独自の追加ではなく、共有 CSS 側にもある問題である。
- **どこで:** `docs/design/work/phase4-opencode/finance-data.md:18`、`docs/design/work/phase4-codex/tokens.css:9,34`、`docs/design/work/phase5-opencode/html/tokens.css:9,34`、`site.css:47,69,81,99,165`。
- **なぜ問題か:** `font-family: var(--font-mono)` は数字だけでなく対象要素内の書体も変えるため、トークンを共有していても金融表示ルールと矛盾する。
- **どう直すか:** 共有 CSS の `.numeric` と OpenCode の `site.css` の双方で、金額は本文書体のまま `tabular-nums` を使う。金額表示に等幅書体の指定が残らないことを検索で確かめる。

### 4. [高] 危険操作の確認ダイアログにキーボード操作の設計がなく、削除確定のフォーカスも不明

- **何が:** アカウント削除確認と CSV 削除確認は `role="dialog" aria-modal="true"` とタイトル紐付けはあるが、フォーカス初期位置・フォーカス移動制御・Esc で閉じる操作がない。表示は `:target` だけで行い、削除・取消リンクはいずれも各一覧と同じ画面へのリンクで、確認の確定挙動も見本から分からない。
- **どこで:** `docs/design/work/phase5-opencode/html/account-opencode.html:11-12,51-56`、`csv-opencode.html:19-20,92-112`(国内株式・配当金・投資信託の削除確認)。`receipts-opencode.html:277-291` の `rcpt-d1` は配当の行詳細であり、この危険操作の指摘対象には含めない。
- **なぜ問題か:** 破壊的操作をキーボード・読み上げ利用者が安全に確認できず、実装者に必要なダイアログ挙動が伝わらない。状況表でもキーボード/フォーカス未検証と記載 (`screens-status.md`「未確認」)。
- **どう直すか:** ダイアログを開いた時のフォーカス先、Tab の範囲、閉じた後の復帰先、Esc/取消、削除ボタンでのみ確定する挙動を設計に明記し、PC/スマホのキーボード確認画像または操作記録を添える。

### 5. [中] インベントリで列挙された複数の状態が OpenCode 担当画面に反映されていない

- **何が:** Phase 1 は CSV の取得中・取込中、設定の読込/保存中/保存失敗、確認・失敗等を状態として挙げる。Phase 5 の CSV 画面には取込成功・一部スキップ・取得失敗・削除確認・急減確認が既にある。状態ボードには一般的な読込・空・再試行・部分失敗・未ログイン・権限なしと管理者検索0件がある。一方、CSV の取得中/取込中・取込全体の失敗・削除中/失敗、設定保存中/失敗、端末失効/全端末ログアウトの確認・失敗は見当たらない。設定の `settings-opencode.html:63` は CSV 出力成功の表示で、設定保存成功とは別である。
- **どこで:** `docs/design/work/phase1-codex/screen-inventory.md:13,19,30-31,57-58`、`docs/design/work/phase5-opencode/html/csv-opencode.html:45-46,65-67,92-138`、`states-opencode.html:31-87`、`settings-opencode.html:53-85`。
- **なぜ問題か:** Phase 1 が要求する既存機能・エラー時の振る舞いを実装者が画面見本から確認できず、主要な設定・CSV操作の失敗時 UX が未設計のまま残る。
- **どう直すか:** CSV 取得/取込/削除の処理中、取込全体/削除の失敗、設定保存中/失敗、端末失効/全端末ログアウトの確認と失敗を追加する。既存の成功・一部スキップ・取得失敗・削除/急減確認は再利用し、Phase 1 の状態項目ごとに根拠ショットを対応づける。

### 6. [中] 管理者検索の「＋ 場所」が操作可能なコントロールとして表現されていない

- **何が:** 条件パネルを開くように見える `＋ 場所` は `span` で、ボタンでもリンクでもない。表示テキストはあるが、フォーカス可能な操作要素としての役割やキーボード操作がない。
- **どこで:** `docs/design/work/phase5-opencode/html/admin-logins-opencode.html:29-34`。
- **なぜ問題か:** 検索条件追加という既存操作の入口が、見た目だけの静的ラベルになっている。操作可能かどうかを支援技術が判別できない。
- **どう直すか:** 実際に開くボタンとして表現し、開閉状態とパネルとの関連を aria で示す。PC/スマホのパネル状態もショットに含める。

### 7. [低] Phase 4 の金融データ例は Phase 5 の共通 fixtures と異なる数値セット

- **何が:** Phase 4 の例は評価額合計 ¥3,345,000、年間配当(税引後) ¥300,000、Phase 5 では評価額 ¥28,593,148、年間配当(税引後) ¥3,163,438(税引前 ¥3,969,907 − 税額 ¥806,469)を使う。Phase 4 の文書は本書内の共通例として合計一致を明記しているため誤計算ではないが、Phase 5 データとの連続性はなく、仕様例と画面例の照合が難しい。
- **どこで:** `docs/design/work/phase4-opencode/finance-data.md:6-12`、`docs/design/work/phase5-opencode/screens-status.md:6`。
- **なぜ問題か:** 同じ画面群の設計検討で、異なるフィクスチャを前提にした例が並び、数値の再利用や視覚比較時に誤差と仕様差を区別しにくい。
- **どう直すか:** ルール例として別データを維持する場合は「Phase 5 fixtures とは別の説明用データ」と明記する。共通値の比較を意図する節では fixtures の同じ値に揃える。

## 数値・レスポンシブ確認

- Phase 5 状況表の共通値と Codex Phase 5 の表示を照合した。評価額 ¥28,593,148、取得総額 ¥27,040,660、評価損益 +¥1,552,488 は一致。配当税引前 ¥3,969,907 − 税額 ¥806,469 = 税引後 ¥3,163,438、国内株式 +¥99,017、投資信託 +¥147,282、直近配当 ¥10,957 / ¥23,905 も OpenCode の明細・生成検算値と Codex 側で一致する。OpenCode の `account-opencode.html:44` は「年間予想配当(税引前)」として同じ ¥3,969,907 を表示している。数値の一致と、予想配当/明細の年間合計の意味が一致するかは分けて扱う。
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
| CSV 取得・取込・削除・急減確認 | △ | 取込成功・一部スキップ・取得失敗・削除/急減確認あり。取得/取込/削除の処理中、取込全体/削除の失敗が不足 |
| 旧 CSV URL 群 | 担当外 | 統合画面への旧 URL alias (`screen-inventory.md`) |
| 資産管理 `/assetbalance` すべて・国内株式・投資信託 | 担当外 | Phase 5 OpenCode 状況表「担当外」 |
| 構成比 `/assetbalance/composition` | 担当外 | 同上 |
| 旧保有株詳細 URL `/assetbalance/holdings/:id` | 担当外 | 同上 |
| 投信詳細 `/assetbalance/funds/:id` | 担当外 | 同上 |
| 設定 `/settings` 7セクション | △ | `settings-opencode.html` と PC/SP/dark shots あり。保存失敗・確認状態が不足 |
| 管理者ログイン履歴 `/admin/logins` | △ | HTML/PC/SP/dark/0件状態あり。条件追加が非操作要素、AD-1コピーは担当外扱い |
| ログイン `/login` | ○ | HTML、PC/SP/dark shots |
| 404・未知パス | ○ | HTML、PC/SP/dark shots |
| アカウントメニュー | △ | PC/SP shots と `account-opencode.html:27` の `aria-expanded="true"` はあり。開閉時の属性更新・キーボード動作は未提示 |
| アカウント削除確認 | △ | PC dark shot と dialog HTML あり。フォーカス/確定操作が未設計 |
| CSV行メニュー・削除確認 | △ | CSV PC dark shot と `csv-opencode.html:92-112` の削除確認あり。指摘4のフォーカス/確定操作が未設計 |
| ホーム配当詳細ダイアログ | 担当外 | Phase 1 のホーム画面内状態。Codex 担当 |
| 保有株詳細ダイアログ | 担当外 | 資産管理の画面内状態。Codex 担当 |
| 検索・条件の共通パネル | △ | 明細条件はあり。管理者「＋ 場所」は操作要素でなく、他担当画面は担当外 |
| 認証/権限/読込/空/通信失敗/部分失敗 | △ | 状態ボードあり。CSV/設定/端末操作の具体的な状態が不足 |
| 端数値・長い名前・複数口座・欠測/価格更新待ち | ○ | `states-opencode.html:76-86`、PC/SP/dark shots |

## 画像確認記録

Phase 4 `shots/` の3枚と Phase 5 `shots/` の30枚、合計33枚を画像として開いて確認した。Phase 5 は全画像をコンタクトシートにして表示し、Phase 4 は各画像を表示して確認した。個別の実機表示やブラウザ操作は行っていない。
