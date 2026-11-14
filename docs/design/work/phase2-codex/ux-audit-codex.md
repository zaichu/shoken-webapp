# Phase 2 UI/UX 課題分析（Codex）

## 実際に見た画像

実画像を開いて確認したのは **31枚**（Codex 撮影 15枚、OpenCode 撮影 16枚）。画像パスは本書の各項目に記した。PC/スマホ、ライト/ダークを含む。撮影資料に記載された架空データだけを対象とした。

**未確認の画面・状態:** 構成比の PC/ダーク・他基準、投信詳細のスマホ/ライト、株保有詳細の PC、明細の条件別展開（PC/スマホ）、資産/明細の空・エラー全組み合わせ、検索0件（資産/明細）、CSV の行メニュー・削除確認・取込結果、ホームの配当詳細、アカウント削除確認、ローディング、設定の全セクション（アカウント/表示/データ/セキュリティ/端末）、実機ブラウザ。スクリーンショットがないことは撮影 INDEX の理由も参照した。

## 対象と資料の突合せ

Codex の `phase1-codex/screen-inventory.md` と OpenCode の `phase1-opencode/screens.md` を比較した。ルート・主要タブ・設定7セクション・管理者画面の把握は概ね一致する。Codex は構成比・保有詳細・空/失敗状態・条件UI・行詳細などを撮影できず、OpenCode はそれらの画像を含む一方、Codex は幅違い（1024/1440/1920）・一覧のスクロール後・管理者画面を多く含む。したがって、どちらか一方の撮影を全画面カバレッジとはみなさず、両資料の未撮影欄を合わせて未確認にした。

一覧上の相違として、Codex のルート棚卸しは「固定パス12件+旧URL5件+保有ID群2件+NotFound」と分類し、OpenCode はルート14個（旧URL互換6個を含む）と記載している。これは URL パターンと画面種別の数え方が異なる。`frontend/src/app.rs:84-123,354-426` のルート分岐では、旧URLは既存画面への別名、株保有IDは銘柄検索へ遷移する互換経路であり、独立画面として数えないのが妥当。投信詳細と構成比は別画面。OpenCode の棚卸し中の「構成比=取得額構成比」は、現行UIに評価額/年間配当基準がある記載（同資料の詳細部および `frontend/src/features/asset_balance/view/composition_page.rs`）と食い違うため、コード側の複数基準を採用した。Codex 撮影 INDEX は「構成比未撮影」とし、OpenCode に画像あり。

## 課題一覧

### 1. 金融データの時点が短い日付だけで示される（重要度: 高）

- **観点:** 金融データの表示精度、情報の見つけやすさ
- **根拠:** `phase1-codex/shots/home-pc-light.png` は上部に「データ: 2026/1/1 取り込み」、評価額には「価格基準日 10/20」と表示。`phase1-codex/shots/asset-all-1440x900-pc-light.png` の集計にも「価格基準日 10/20」。年を含む取込日と年なしの価格基準日が同時に見える。
- **課題:** 画面上で異なる種類の時点が隣り合い、価格がいつ時点かを年まで含めて素早く特定できない。資産判断に使う値なので、表示幅が狭くても時点の意味と年月日を取り違えない形が必要。
- **改善方針:** 取込時点と価格/基準価額の時点を別ラベルで明示し、価格基準日は年を含む日付に統一する。詳細な根拠や対象データがあれば注記行に集約する。

### 2. ホームのモバイル上部が他の集計画面と異なる構造（重要度: 中）

- **観点:** 情報の見つけやすさ、情報密度、画面間の一貫性、レスポンシブ
- **根拠:** `phase1-codex/shots/home-mobile-light.png` と `home-mobile-dark.png` では評価額/評価損益が上バー直下にあり、評価額だけの開閉ボタン、別カードの配当グラフ、別カードの資産内訳が続く。対して `phase1-codex/shots/receipts-dividend-mobile-dark.png` と `phase1-opencode/shots/asset-error-default-mobile-light.png` はページ名・全幅タブ・集計/状態が明確に縦の帯として続く。
- **課題:** ホームだけ要約値のまとまり方が違い、配当カード・内訳カードまでスクロールして主要情報と周辺情報を行き来する。暗色表示では配当カードが初期画面の大部分を占め、資産内訳の凡例が下部ナビに近い。
- **改善方針:** ホームも共通の集計帯の規則に合わせ、評価額・評価損益・配当サマリーの優先順と開閉対象を明確にする。グラフと内訳は要約の下に置き、モバイル初期画面で主要数値が見比べられるようにする。

### 3. 長い銘柄名が一覧で省略され、同名商品の識別が難しい（重要度: 中）

- **観点:** 情報の見つけやすさ、可読性、金融データの識別
- **根拠:** `phase1-codex/shots/asset-all-1440x900-pc-light.png` では長いファンド名が末尾「…」で省略されている。`phase1-codex/shots/asset-all-scrolled-mobile-light.png` と `phase1-codex/shots/receipts-fund-mobile-light.png` ではモバイルで折返し表示されるが、初期画面では1〜2件目が多くの縦幅を使う。
- **課題:** 同じ接頭辞の長い商品名は PC で区別しづらく、モバイルでは読むために一覧密度が下がる。名前が識別キーとなる場面では省略だけでは不十分。
- **改善方針:** PC は名称列に十分な幅を割り当て、完全名を行詳細またはフォーカス可能な補助表示で確認できるようにする。スマホは完全名の折返しを許容しつつ、行内の優先情報を固定して金額との視線移動を減らす。

### 4. 明細の情報密度が高く、一覧と詳細の役割が曖昧（重要度: 中）

- **観点:** 情報密度と可読性、操作の分かりやすさ、金融データの表示
- **根拠:** `phase1-codex/shots/receipts-stock-scrolled-1440x900-pc-light.png` の PC 表は日付、コード、銘柄、口座、数量、売却額、取得額、実現損益、税額、税引後を横一列に表示。対して `phase1-opencode/shots/receipts-detail-dialog-default-mobile-light.png` は同一種別の税引前・税額・税引後などを詳細ダイアログに配置し、一覧の行は少数情報に絞る。
- **課題:** PC では比較できる一方で列数が多く、一覧を読む際に目的の数値へ視線を移しにくい。モバイルではダイアログを開いて初めて詳細な金額内訳が見えるため、重要な値を一覧だけで比較する利用者には発見性が低い。
- **改善方針:** 一覧では主要値（税引後損益/受取額）を強調し、補助値は列のグルーピングと整列を明確にする。モバイルでは詳細を開く操作を行全体で一貫させ、詳細内の項目順を PC の列順に揃える。

### 5. 構成比チャートのラベルが狭い幅で切れ、凡例と一覧の対応が弱い（重要度: 中）

- **観点:** 情報の見つけやすさ、情報密度、レスポンシブ、金融データ表示精度
- **根拠:** `phase1-opencode/shots/asset-composition-default-mobile-light.png` では凡例の銘柄名が「…」で省略され、チャート色と順位表示はあるが長い名前を識別できない。下の順位一覧ではフルネームが複数行に折り返される。
- **課題:** チャート凡例で上位商品の識別が困難で、色だけを頼りに下の一覧と照合する必要がある。多数銘柄の小さな円図も単独では詳細比較しにくい。
- **改善方針:** 凡例は名称を一意に判別できる情報（短縮名+コード等）を残し、割合と順位一覧を同じ順序・同じ値表記で対応させる。全件の構成確認を優先する場合は、円図を補助的に扱う。

### 6. CSV 管理の更新状態が色と小さな時刻情報に依存する（重要度: 中）

- **観点:** 操作の分かりやすさ、アクセシビリティ、データ時点
- **根拠:** `phase1-codex/shots/csv-asset-1024x768-pc-dark.png` と `phase1-opencode/shots/csv-mutualfund-default-mobile-light.png` では「要更新・15日前(10/31 09:12)に更新」が橙色で表示される。0件の未取得は「まだ取得していません」と別表現。
- **課題:** 要更新の状態が色付きの長い文にまとまり、更新時刻（年なし）と次の操作が一読で分かれない。色覚や低コントラスト環境では状態の区別を色だけで補えない。
- **改善方針:** 状態名（要更新/未取得/更新済み）と最終取得時点を別項目として表示し、日付形式を全画面で統一する。各行のメニュー操作は状態と独立して明示する。

### 7. 設定画面の長いブラウザ名がテーブルとカードで異なる見え方（重要度: 低）

- **観点:** 可読性、レスポンシブ、一貫性
- **根拠:** `phase1-codex/shots/settings-login-history-scrolled-1440x900-pc-light.png` ではブラウザ/OS名が末尾省略、`phase1-opencode/shots/settings-login-history-default-mobile-dark.png` では複数行に折返した後に省略される。
- **課題:** 端末を見分ける情報が PC とスマホで異なる長さだけ見え、同じ履歴の比較がしづらい。
- **改善方針:** 表示名を短いブラウザ/OS名に正規化し、完全な user agent が必要なら詳細操作で参照できるようにする。

### 8. 画面ごとに空・エラー表示後の次の行動が異なる（重要度: 低）

- **観点:** 操作の分かりやすさ、画面遷移、視覚的一貫性
- **根拠:** `phase1-opencode/shots/receipts-empty-default-mobile-light.png` は CSV 取込ボタン付き空状態、`phase1-opencode/shots/asset-empty-default-mobile-light.png` は説明と CSV 導線付き。`phase1-opencode/shots/receipts-error-default-pc-dark.png` と `asset-error-default-mobile-light.png` は再読込ボタン付きだが、パネル/枠や背景が異なる。
- **課題:** 同じ一覧画面でも空状態の誘導文とエラー枠の見え方に差があり、失敗から再試行する場所が画面ごとに変わる。
- **改善方針:** 空は「登録/取込」の次の行動、失敗は「再読み込み」を共通部品・共通位置で提示し、説明文だけ画面固有にする。

## 画面をまたぐ課題

### 用語・時点・数値表現

- ホームには「評価額 価格基準日 10/20」「年間予想配当(税引前)」があり、資産管理には「評価額 価格基準日 10/20」、明細には「受け取った配当金(税引後)」がある（`home-pc-light.png`, `asset-all-1440x900-pc-light.png`, `receipts-dividend-mobile-dark.png`）。税引後と予想配当を近接表示する際は、実績/予想と税引前/税引後をラベルで常に区別する。表示例の意味は `docs/ui-design.md:20-47` の用語・書式ルールに合わせる。
- 価格基準日、取込日、CSV 最終取得日が画像上ではそれぞれ異なる粒度（年なし/年あり/相対日数+時刻）。全画面で「何の時点か」と年月日を揃える。
- ライト/ダークの切替でホームの構成比グラフ色が異なって見える（ライト `home-mobile-light.png` は青/濃紺、ダーク `home-mobile-dark.png` は橙/青）。色を意味として使うならテーマを越えて同じ資産種別へ同じ色を割り当て、色以外のラベルも残す。

### 共通部品・操作・密度

- 検索UIは資産一覧では本文内、明細では画面下部の入口、管理者一覧では本文入力/条件追加と異なる。用途に応じた例外は妥当だが、入口名・条件適用後の件数表示・クリア操作は共通化する（`asset-search-ui-default-pc-light.png`, `receipts-search-ui-default-mobile-dark.png`, `admin-logins-empty-1920x1080-pc-light.png`）。
- 空/失敗表示、検索件数、長い文字列の省略位置を共通部品に寄せる。設計上は共通 `EmptyState`/`ListLoadError` が定義されている（`docs/design/mobile-ui-rules.md:45-56`）。
- アカウントメニューはスマホで画面右上に重なって開き、背後のチャート情報を覆う（`account-menu-default-mobile-light.png`）。開いた後に外側クリック/フォーカス移動/閉じる操作が可能かは静止画では未確認。重なりが少ない位置とキーボードフォーカスを確認する。
- スマホの管理者ログイン一覧は長いメールアドレスを複数行に折り返す（`admin-logins-1024x768-scrolled-pc-dark.png` は PC 画像であり、スマホ版のこの端のデータは未確認）。管理者画面のスマホ端データは撮影 INDEX 上に存在するか要照合し、この監査では断定しない。

## 観点別の確認メモ

- **情報の見つけやすさ:** ホームと資産/明細では主要値の位置が異なる。構成比凡例と長い名前は省略される。
- **操作の分かりやすさ:** 明細の検索シート、CSV 行メニュー、詳細ダイアログを確認。CSV の行メニュー・削除確認は未確認。
- **情報密度と可読性:** PC 資産表/明細表とスマホ行リストを確認。明細 PC の列数、ホームのスマホカード高さ、長い名称の省略が課題。
- **画面遷移:** 銘柄検索の保有情報と「資産管理に戻る」、投信詳細の戻るリンク、404 のホーム導線を確認。旧URLの遷移動作は画像ではなく棚卸しコードを参照。
- **視覚的一貫性:** 下部ナビ、タブ、帯、状態枠は概ね共通だが、ホームの集計構造・空/失敗の枠・テーマ間チャート色に差がある。
- **金融データ表示精度:** 円・数量・割合・損益符号、税引前/税額/税引後、日付表示を画像で確認。基準日/取込日が年なしで並ぶ点を高優先で改善候補とした。数値の実際の正確さは画像だけでは検証できない。
- **レスポンシブ:** 390px、1024px、1440px、1920pxの画像を確認。全ての画面を全幅・全テーマで確認したわけではない。横スクロールの有無は画像のみでは断定できない。
- **アクセシビリティ:** 画像で見えるラベル付きボタンと色・フォーカス枠のみ観察。キーボード順、フォーカス可視性の全状態、コントラスト比、実タップ領域、スクリーンリーダー名は画像から確認できず未確認。
- **実装と保守の容易さ:** 画像から実装の重複や保守性は判定できない。設計ルールは共通部品を定めているため、改善時は `docs/design/mobile-ui-rules.md` と `docs/ui-design.md` の共通部品へ集約する。

## 画像パス一覧（実際に開いて確認した31枚）

### Codex 撮影（15枚）

- `phase1-codex/shots/home-pc-light.png`
- `phase1-codex/shots/home-mobile-light.png`
- `phase1-codex/shots/home-mobile-dark.png`
- `phase1-codex/shots/asset-all-1440x900-pc-light.png`
- `phase1-codex/shots/asset-all-scrolled-mobile-light.png`
- `phase1-codex/shots/asset-fund-scrolled-mobile-dark.png`
- `phase1-codex/shots/receipts-stock-scrolled-1440x900-pc-light.png`
- `phase1-codex/shots/receipts-dividend-mobile-dark.png`
- `phase1-codex/shots/receipts-fund-mobile-light.png`
- `phase1-codex/shots/search-mobile-light.png`
- `phase1-codex/shots/csv-asset-1024x768-pc-dark.png`
- `phase1-codex/shots/csv-scrolled-mobile-light.png`
- `phase1-codex/shots/settings-login-history-scrolled-1440x900-pc-light.png`
- `phase1-codex/shots/admin-logins-1024x768-scrolled-pc-dark.png`
- `phase1-codex/shots/admin-logins-empty-1920x1080-pc-light.png`

### OpenCode 撮影（16枚）

- `phase1-opencode/shots/asset-composition-default-mobile-light.png`
- `phase1-opencode/shots/asset-holding-fund-default-pc-dark.png`
- `phase1-opencode/shots/asset-holding-stock-default-mobile-dark.png`
- `phase1-opencode/shots/asset-empty-default-mobile-light.png`
- `phase1-opencode/shots/asset-error-default-mobile-light.png`
- `phase1-opencode/shots/asset-search-ui-default-pc-light.png`
- `phase1-opencode/shots/receipts-empty-default-mobile-light.png`
- `phase1-opencode/shots/receipts-error-default-pc-dark.png`
- `phase1-opencode/shots/receipts-search-ui-default-mobile-dark.png`
- `phase1-opencode/shots/receipts-detail-dialog-default-mobile-light.png`
- `phase1-opencode/shots/csv-mutualfund-default-mobile-light.png`
- `phase1-opencode/shots/settings-login-history-default-mobile-dark.png`
- `phase1-opencode/shots/account-menu-default-mobile-light.png`
- `phase1-opencode/shots/login-default-mobile-dark.png`
- `phase1-opencode/shots/search-empty-default-mobile-dark.png`

ファイルパスはそれぞれ `/home/zaichu/project/.worktrees/shoken-webapp-1890-codex/docs/design/work/phase1-codex/` と `/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/docs/design/work/phase1-opencode/` を基点とする。
