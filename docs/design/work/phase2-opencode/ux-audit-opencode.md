# Phase 2 UX 分析 (OpenCode)

- 対象: `main` の全画面。Phase 1 の撮影画像を目視して分析した。
- 材料: OpenCode 撮影 (`docs/design/work/phase1-opencode/shots/`)、Codex 撮影 (`docs/design/work/phase1-codex/shots/`)、棚卸し (`docs/design/work/phase1-codex/screen-inventory.md` / `docs/design/work/phase1-opencode/screens.md`)、決まり (`docs/design/mobile-ui-rules.md`、`docs/ui-design.md`、`frontend/README.md`)、コード (読み取りのみ)。
- 重要度: 高=利用阻害・データ誤認・誤字の可能性 / 中=操作負荷・一貫性の欠け / 低=改善余地。

## 0. 実際に見た画像と見られなかった画面

### 見た画像: 42 枚 (すべて目視済み)

OpenCode 撮影 41 枚 + Codex 撮影 1 枚。PC 1280×800 / モバイル 390×844、ライト/ダークを含む。

- ホーム: `docs/design/work/phase1-opencode/shots/home-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/home-default-pc-dark.png`、`docs/design/work/phase1-opencode/shots/home-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/home-default-mobile-dark.png`、`docs/design/work/phase1-opencode/shots/home-loggedout-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/account-menu-default-pc-light.png`
- 資産管理: `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-all-default-pc-dark.png`、`docs/design/work/phase1-opencode/shots/asset-all-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/asset-all-default-mobile-dark.png`、`docs/design/work/phase1-opencode/shots/asset-search-ui-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-search-ui-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/asset-empty-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-error-default-pc-light.png`
- 取引明細: `docs/design/work/phase1-opencode/shots/receipts-dividend-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-dividend-default-pc-dark.png`、`docs/design/work/phase1-opencode/shots/receipts-dividend-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/receipts-dividend-default-mobile-dark.png`、`docs/design/work/phase1-opencode/shots/receipts-stock-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-fund-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-search-ui-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-detail-dialog-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/receipts-empty-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-error-default-pc-light.png`
- 銘柄検索: `docs/design/work/phase1-opencode/shots/search-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/search-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/search-empty-default-pc-light.png`
- 構成比・投信詳細: `docs/design/work/phase1-opencode/shots/asset-composition-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-holding-fund-default-pc-light.png`
- CSV: `docs/design/work/phase1-opencode/shots/csv-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/csv-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/csv-default-mobile-dark.png`
- 設定: `docs/design/work/phase1-opencode/shots/settings-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/settings-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/settings-login-history-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/settings-login-history-default-mobile-light.png`
- 管理者: `docs/design/work/phase1-opencode/shots/admin-logins-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/admin-logins-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/admin-logins-empty-default-pc-light.png`
- その他: `docs/design/work/phase1-opencode/shots/login-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/notfound-default-pc-light.png`
- Codex 撮影 (幅確認用): `docs/design/work/phase1-codex/shots/home-1920x1080-pc-light.png`

### 見られなかった画面・状態 (未確認)

- スクロール後の画像全般 (`*-scrolled.png`。帯の開閉後・下部・フッター周辺は未目視)
- ダークの多く (`asset-error`、`receipts-error`、`search`、`settings`、`admin`、`login`、`notfound`、`composition`、`holding-fund` 等のダーク)
- モバイルの多く (`asset-stock`/`asset-fund`、`asset-holding-fund`、`asset-composition`、`receipts-stock`/`receipts-fund`、`receipts-search-ui`、`search-empty`、`admin-empty`、`login`、`notfound`、`csv-asset`、`csv-mutualfund`)
- `asset-holding-stock-default-*` (国内株式→銘柄置換の遷移先の実表示)
- `csv-asset-default-*`、`csv-mutualfund-default-*`、`settings-data-default-*`
- PC 版の行詳細ダイアログ、期間・商品の条件パネル、銘柄検索のモバイル検索シート展開時
- CSV 削除確認モーダル・取込結果、CSV 行メニュー展開時、アカウント削除モーダル
- 読み込み中 (skeleton)、部分失敗・再取得エラー (`Alert`)、銘柄検索の取得失敗、未ログインでの共有 URL 表示
- Codex 撮影の 1024/1440 幅、Codex 撮影の残り約 260 枚 (系列は INDEX で把握、目視は上記 1 枚のみ)
- 実機ブラウザ・フォント差、320px 幅の実表示 (コードと a11y spec のみで確認)

### Phase 1 資料の突き合わせ結果

- Codex `docs/design/work/phase1-codex/shots/INDEX.md` は構成比・投信詳細・404 等を「撮影ケースなし・未撮影」とするが、OpenCode `docs/design/work/phase1-opencode/shots/` には `asset-composition-*`、`asset-holding-fund-*`、`notfound-*` が存在し、本分析で目視した。ルート自体はコードに存在する (`frontend/src/app.rs:98`、`frontend/src/app.rs:80-82`) ため、Codex 側の「未撮影」は撮影スクリプトのモック不足によるもので、画面が存在しないという意味ではない。OpenCode の記録が正。
- 画面数・ルート一覧は両資料で一致 (機能画面 11 種 + 旧 URL 互換)。`docs/design/work/phase1-codex/screen-inventory.md` の「画面・タブ・設定セクション 30」の数え方と `docs/design/work/phase1-opencode/screens.md` の「機能画面 11 種」は粒度の違いのみで矛盾なし。
- `docs/design/work/phase1-opencode/ux-audit-draft.md` は「画像入力非対応のため目視未実施」と明記していた。本分析はその未確認分 (目視) を埋めるもので、コード起点の指摘 (C-1〜C-9、D-1〜D-6) と重複する箇所は画像で再検証した。

## 1. 重要度「高」の課題

### H-1. ホームの配当プログレスバーが振り切れて比較として機能していない

- 重要度: 高
- 根拠 (画像): `docs/design/work/phase1-opencode/shots/home-default-pc-light.png` (モバイル `docs/design/work/phase1-opencode/shots/home-default-mobile-light.png` も同じ)。「受取 +¥3,163,438 / 予想 196,373」「1611%」でバーが全幅満杯。
- 内容: 実績が予想を大幅に上回るとバーが 100% に張り付き、月別内訳との関係も読み取れない。達成率バーとしての意味を失っている。
- 改善方針: 実績>予想の場合の表示を分ける (例: バーを上限で止めて「予想の 16.1 倍」と文言で出す、または目標達成型の表示に切り替える)。`frontend/src/features/home/view.rs` の配当バー分岐が対象。数式・文言の詳細は実装担当に委ねるが、1611% のような 4 桁% は出さない。

### H-2. (取り下げ) エラー画面の再試行ボタン表記 — 誤字なしを確認

- 取り下げ理由: `docs/design/work/phase1-opencode/shots/asset-error-default-pc-light.png` を原寸で確認し、ボタン表記は「再読み込み」で誤字なしのため本項を取り下げる。

## 2. 画面別・観点別の課題

### 2.1 ホーム `/`

| # | 観点 | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|---|
| HO-1 | 情報密度と可読性 | 月別棒グラフの 1〜10 月がほぼ 0 で潰れて判読不能。凡例「2026年/2025年」の区別が色のみ | 中 | `docs/design/work/phase1-opencode/shots/home-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/home-default-pc-dark.png` | 値 0 の月は棒を省略して注記にするか、凡例に形 (実線/破線) を足す |
| HO-2 | 情報密度と可読性 | モバイルで円グラフ凡例と口座別バー・口座バッジが縦に長く、下部ナビ直前まで迫る。`NISAつみたて` 等のバッジが行を広げる | 中 | `docs/design/work/phase1-opencode/shots/home-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/home-default-mobile-dark.png` | 凡例は件数つき 2 列維持のまま、口座別バーとバッジ群を折りたたみにする |
| HO-3 | 視覚的一貫性 | 達成率バー (紺) と月別棒 (紺/灰) と口座別バー (多色) で色の意味が画面内で不統一に見える | 低 | `docs/design/work/phase1-opencode/shots/home-default-pc-light.png` | 達成系は単色、内訳系は系列色と決めて凡例に書く |
| HO-4 | 金融データの表示精度 | 帯に「評価額 価格基準日 10/20」「データ: 2026/1/1取り込み」と時点表示あり。配当チャートの対象期間と帯の基準日の関係は画面から読み取れない | 低 | `docs/design/work/phase1-opencode/shots/home-default-pc-light.png` | チャート見出しに「2026/01〜2026/12」等の期間を添える (明細の帯と同様) |

良好だった点: 帯の 3 項目 (評価額・評価損益・年間予想配当) は PC/モバイル・ライト/ダークで崩れなし。ダークの緑/赤は目視で判読可。未ログインは案内カードのみで操作不能箇所なし (`docs/design/work/phase1-opencode/shots/home-loggedout-default-pc-light.png`)。

### 2.2 資産管理 `/assetbalance`

| # | 観点 | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|---|
| AS-1 | 情報の見つけやすさ | 長いファンド名が `…` で切れる。PC 表で「架空グローバル先端素材ソリューション…」「架空・世界革新テクノロジー…購入・換…」の 2 件を確認。ホバーしないと全文不明 | 中 | `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png` | `title` または 2 行表示にする。`mobile-ui-rules.md` は hover 前提 UI を禁じている (`ui-design.md:15`) |
| AS-2 | 情報の見つけやすさ | 帯の項目数・順序がタブで変わる (すべて: 評価額/評価損益/前日比/取得総額、国内株式: +年間配当・利回りで 5 項目)。タブ切替で帯の形が変わり、定点観測しにくい | 中 | `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-search-ui-default-pc-light.png` (国内株式タブ) | 項目の順序を固定し、タブ固有項目は末尾に足す |
| AS-3 | レスポンシブ | モバイル行で長い銘柄名が 2〜3 行に折り返し、行高がばらつく (例: 「架空人生設計・長期安全感・資産形成ファンド」)。読めるため実害は小さい | 低 | `docs/design/work/phase1-opencode/shots/asset-all-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/asset-all-default-mobile-dark.png` | 現状維持可。揃えるなら銘柄名を 2 行までに制限 |
| AS-4 | 金融データの表示精度 | 金額列は右寄せ・桁区切り・符号+色が PC/モバイル・ダークで統一。1 株配当は小数 1 桁で揃っている (¥72.5/¥71.0/¥0.0)。欠測は `—` | — (良好) | `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-all-default-pc-dark.png`、`docs/design/work/phase1-opencode/shots/asset-search-ui-default-pc-light.png` | 維持 |

### 2.3 取引明細 `/receipts`

| # | 観点 | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|---|
| RE-1 | 情報の見つけやすさ | モバイルは税額・税引前が一覧に出ず、`DetailDialog` を開く必要がある。PC は全列表示。同じ情報に辿り着く手数が違う | 中 | `docs/design/work/phase1-opencode/shots/receipts-dividend-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/receipts-detail-dialog-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/receipts-dividend-default-pc-light.png` | モバイル行の 2 段目に税引後だけでなく税額の目安を足すか、現状のまま (密度とのトレードオフを Phase 3 で判断) |
| RE-2 | 操作の分かりやすさ | 口座の条件ポップオーバーが表の列見出しに重なる。閉じれば問題ないが、開いている間は下の行を読めない | 低 | `docs/design/work/phase1-opencode/shots/receipts-search-ui-default-pc-light.png` | 条件パネルを開いたら表側を非活性化する旨が分かるよう、 backdrop を濃くする |
| RE-3 | 金融データの表示精度 | グループ小計行 (例: 「2026年12月・3件 +¥2,023,380 −¥411,048 +¥1,612,332」) と帯の集計の関係が一目で分からない | 低 | `docs/design/work/phase1-opencode/shots/receipts-dividend-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-stock-default-pc-light.png` | 小計行のラベルに「(3 件の合計)」等を添えるか現状維持 |
| RE-4 | 視覚的一貫性 | 空状態で残る「条件を足す」ボタンが「+ 期間」のみ (口座・商品は出ない)。空データ時に条件追加の意味が薄い | 低 | `docs/design/work/phase1-opencode/shots/receipts-empty-default-pc-light.png` | 空状態では条件行ごと出さない |

良好だった点: 帯ラベルのタブ別切替 (配当=受取配当金/株式・投信=実現損益) は用語が統一されている。`DetailDialog` (モバイル) は数量・税額・税引後が 1 画面に収まり「閉じる」が明確 (`docs/design/work/phase1-opencode/shots/receipts-detail-dialog-default-mobile-light.png`)。

### 2.4 銘柄検索 `/search`

| # | 観点 | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|---|
| SE-1 | 情報の見つけやすさ | 未ログインで共有 URL を開いた場合の保有セクションの扱いは画像なし (撮影はログイン済みのみ)。コード上は非表示とのこと (`docs/design/work/phase1-opencode/ux-audit-draft.md` §2.2) | 中 | 未確認 (`docs/design/work/phase1-opencode/shots/search-default-pc-light.png` はログイン済み) | 「ログインすると保有情報が見られます」の案内を出す (draft の提案を支持) |
| SE-2 | 情報密度と可読性 | 空状態の説明文が 3 行 (コード例・会社名例・証券サイト言及)。`ui-design.md:14` の「見ればわかる案内は書かない」にやや冗長 | 低 | `docs/design/work/phase1-opencode/shots/search-empty-default-pc-light.png` | 2 行に縮める (例とリンク先は残す) |

良好だった点: 結果画面の保有タイル→現在値・数量・配当の 4 列グリッドは桁・単位が揃っている。0 配当は「¥0」「0.00%」と明確 (`docs/design/work/phase1-opencode/shots/search-default-pc-light.png`)。モバイルは検索欄が下部操作帯に固定され、結果をスクロールしても操作が残る (`docs/design/work/phase1-opencode/shots/search-default-mobile-light.png`)。

### 2.5 構成比 `/assetbalance/composition`

| # | 観点 | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|---|
| CO-1 | 情報の見つけやすさ | 「ほか 67 件」の内訳への導線が一覧の下 (要スクロール) か不明。スクロール後画像を見ていないため未確定 | 中 | `docs/design/work/phase1-opencode/shots/asset-composition-default-pc-light.png` (初期位置のみ目視) | スクロール後画像の目視で確定させる。なければ Phase 3 で撮影追加 |
| CO-2 | 視覚的一貫性 | PC 右上の「構成比を見る」「CSV管理」2 ボタンと件数行の入口の関係 (上バー重複) については §3 X-2 に記載 | 低 | `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`、`frontend/src/features/asset_balance/view.rs:341-342` | §3 X-2 参照 |

良好だった点: 基準切替 (銘柄別/商品別/口座別、評価額/年間配当) が `Segmented` で統一。ドーナツ+凡例+順位一覧の対応は色と名称で取れる。

### 2.6 投信詳細 `/assetbalance/funds/:id`

- 目視 (`docs/design/work/phase1-opencode/shots/asset-holding-fund-default-pc-light.png`): タイル 10 項目 (評価額/評価損益/基準価額/前日比/基準日/口数/平均取得額/取得総額/トータルリターン)。欠測は `—` (トータルリターン)。前日比 −¥95/−¥260 は赤字で明確。良好。
- 課題なし (重要度つきの指摘なし)。モバイル・ダークは未確認。

### 2.7 CSV 管理 `/csv`

| # | 観点 | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|---|
| CSV-1 | 情報の見つけやすさ | 状態文が冗長で意味が取りにくい。「要更新・15日前(10/31 09:12)に更新」は「要更新なのに更新日時?」と読める | 中 | `docs/design/work/phase1-opencode/shots/csv-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/csv-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/csv-default-mobile-dark.png` | 「15日前のデータがあります (10/31取得)」等の形に整理。`ui-design.md:136` の「事実だけ書く」に沿う |
| CSV-2 | 操作の分かりやすさ | 行メニュー「⋯」の中身・削除確認モーダル・取込結果は未撮影のため未確認 | — (未確認) | `docs/design/work/phase1-opencode/shots/INDEX.md` 未撮影欄 | Phase 3 で操作後状態の撮影を追加 |

良好だった点: モバイル 2 段表示は件数・メニューの位置が固定で崩れなし。0 件行「まだ取得していません」は事実のみの表示で決まり通り。

### 2.8 設定 `/settings`

| # | 観点 | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|---|
| ST-1 | 情報の見つけやすさ | モバイルは左 TOC なしの縦積み。7 セクション全体像を掴むには最下部までスクロールが必要 | 中 | `docs/design/work/phase1-opencode/shots/settings-default-mobile-light.png` (`settings.rs:170` の `hidden lg:block`) | セクション先頭に簡易目次 (ページ内リンク) を足す |
| ST-2 | 情報密度と可読性 | ログイン履歴の生 User-Agent 表示が中途半端に切れて折り返す。PC・モバイル両方で確認。利用者に UA 文字列は無意味 | 中 | `docs/design/work/phase1-opencode/shots/settings-login-history-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/settings-login-history-default-mobile-light.png` | ブラウザ/OS 名に正規化して表示し、全文は詳細に隠す |
| ST-3 | 画面遷移 | `#data` 等のフラグメント直行・危険操作・アカウント削除の実表示は初期位置画像になく未確認 | — (未確認) | `docs/design/work/phase1-opencode/shots/settings-default-pc-light.png` (初期位置のみ) | スクロール後・`#danger` 直行の撮影で確定 |

良好だった点: PC の 2 カラム (TOC+フォーム) は崩れなし。テーマ・金額単位の `Segmented`、並び順・初期タブの `Select` はラベルと対応が明確。

### 2.9 管理者ログイン一覧 `/admin/logins`

| # | 観点 | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|---|
| AD-1 | 情報密度と可読性 | 長いメールが PC 表で 3 行に折り返し、行高がばらつく (`long.email.for.administrator.login.history.display.check.0@example.com`) | 中 | `docs/design/work/phase1-opencode/shots/admin-logins-default-pc-light.png` | メール列は省略+ `title`/コピーにする |
| AD-2 | 情報密度と可読性 | モバイルは 1 件が 3 行分 (メール 2 行+日時・場所+ブラウザ/OS) で 50 件表示が長大。`50件 / 64件` のペーシングはある | 低 | `docs/design/work/phase1-opencode/shots/admin-logins-default-mobile-light.png` | 現状維持可。短縮するならメール 1 行省略+詳細シート |
| AD-3 | アクセシビリティ | 管理者画面は axe スキャン対象外の模様 (`a11y.spec.ts` に admin の `page.goto` なし。本分析の grep で 30 件の `page.goto` 中 admin なし) | 中 | `frontend/e2e/accessibility/a11y.spec.ts` | admin ルートを a11y spec に追加 (draft C? の提案を支持) |
| AD-4 | 情報の見つけやすさ | 下部ナビに管理者項目なし (管理者の到達導線はコード上も要確認) | 低 | `docs/design/work/phase1-opencode/shots/admin-logins-default-pc-light.png` (ヘッダーナビに管理者なし) | 設定画面等からの導線追加を検討 |

良好だった点: 検索 0 件 (`docs/design/work/phase1-opencode/shots/admin-logins-empty-default-pc-light.png`) は検索語の保持+クリア導線あり。結果バッジ (成功/新しい端末/場所の変化/失敗) は色と文言で区別できる。

### 2.10 ログイン・404

- ログイン (`docs/design/work/phase1-opencode/shots/login-default-pc-light.png`): 中央カードのみで迷いがない。PC ヘッダーに機能ナビが残るが、保護ルート経由で戻るため実害は小さい。低。
- 404 (`docs/design/work/phase1-opencode/shots/notfound-default-pc-light.png`): 見出し+説明+「ホームに戻る」+関連 3 リンク。良好。モバイル・ダークは未確認。

## 3. 画面をまたぐ課題 (横断)

| # | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|
| X-1 | (取り下げ) エラー文言の要再確認 (H-2) — 誤字なしを確認 | — (取り下げ) | `docs/design/work/phase1-opencode/shots/asset-error-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-error-default-pc-light.png`、`frontend/src/ui/labels.rs:1075`、`frontend/src/ui/state.rs:104` | H-2 の取り下げ (原寸確認で「再読み込み」と確定) に伴い対応不要 |
| X-2 | 構成比への入口が PC では右上ボタンと件数行 (?) の 2 か所に見える。コード上は `page-secondary-link-sm max-lg:hidden` (`view.rs:341`) で上バーにも配置。決まり (`mobile-ui-rules.md:47` 件数行の右 1 か所) とのずれ | 低 | `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`、`frontend/src/features/asset_balance/view.rs:341-342` | PC 上バーの構成比リンクを残すなら決まり側に例外追記、統一するなら件数行に寄せる |
| X-3 | 空状態の型は統一 (アイコン→見出し→説明→操作) だが説明文の長さが画面でばらつく (検索空=3 行、資産空=1 行、明細空=1 行、404=見出しのみ+リンク) | 低 | `docs/design/work/phase1-opencode/shots/search-empty-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-empty-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-empty-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/notfound-default-pc-light.png` | 空状態の説明は 1〜2 行に揃える |
| X-4 | 数値表現は統一 (右寄せ・桁区切り・符号+色・欠測 `—`)。PC/モバイル・ダークで崩れなし。1 株配当の小数桁も揃っている | — (良好) | `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-all-default-pc-dark.png`、`docs/design/work/phase1-opencode/shots/receipts-dividend-default-pc-dark.png`、`docs/design/work/phase1-opencode/shots/asset-search-ui-default-pc-light.png` | 維持。`unit_price_decimals` の運用を継続 |
| X-5 | タブ件数バッジ・口座バッジ (`AccountBadge`)・結果バッジの色分けは全画面で統一。ダークでも判読可 | — (良好) | `docs/design/work/phase1-opencode/shots/asset-all-default-pc-dark.png`、`docs/design/work/phase1-opencode/shots/receipts-dividend-default-pc-dark.png`、`docs/design/work/phase1-opencode/shots/admin-logins-default-pc-light.png` | 維持 |
| X-6 | 設定の TOC なし問題 (ST-1) と管理者導線なし (AD-4) は「セクションへの到達しにくさ」という同根。目次・導線の共通パターンがない | 中 | `docs/design/work/phase1-opencode/shots/settings-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/admin-logins-default-pc-light.png` | 長い画面用の目次パターンを決まりに足す |
| X-7 | 文書間のボタンの記述差 (draft C-1/D-1: `ui-design.md` 3 種 vs README 20 種以上) はコード側 (`frontend/src/ui/button.rs` に多数の variant) を見る限り README が現状に近い。画像上のボタン (Primary/Ghost/メニュー) は違和感なし | 低 | `frontend/src/ui/button.rs`、`docs/design/work/phase1-opencode/shots/account-menu-default-pc-light.png` | `ui-design.md` のボタン節を現状に合わせて更新 |

## 4. 未確認事項 (Phase 3 への送り)

1. (取り下げ済み) H-2 は `docs/design/work/phase1-opencode/shots/asset-error-default-pc-light.png` の原寸確認で「再読み込み」と確定したため対応不要。
2. CO-1 (構成比「ほか 67 件」の導線) — スクロール後画像の目視が必要。
3. 削除確認・取込結果・アカウント削除・行メニュー等の操作後状態 — 撮影ケースの追加が必要 (Codex INDEX も同様に未撮影と記録)。
4. 期間・商品の条件パネル、銘柄検索の失敗・未ログイン共有 URL、部分失敗 (`Alert`)、読み込み中 skeleton の実表示。
5. モバイル・ダークの残り (404、ログイン、投信詳細、構成比、検索 UI、条件シート等)。
6. 1024/1440 幅での表の収まり (Codex 撮影あり・未目視)。PC 表の列数が多い画面 (明細 11 列) の狭幅での挙動。
7. 実機ブラウザ・フォント差、コントラスト実測値 (axe は通過しているが目視は本分析の範囲)。
8. CSV 状態文の秒単位の更新タイミング・「要更新」の判定基準 (仕様の裏付けが必要)。

## 5. 対応表 (Phase 1 資料との関係)

- 本分析で目視により確定したもの: draft C-4 (PC/モバイルの情報差→RE-1 に具体化)、C-5 (設定 TOC→ST-1 に具体化)、C-6/D-2 (構成比入口→X-2 に具体化、画像で重複を確認)、C-8 (管理者導線→AD-4)。
- draft のまま残るもの (本分析で画像なし): C-2 (CSV 文言の例外規定。CSV-1 で具体例を追加)、C-3 (検索配置の例外。画像で現状は妥当と判断)、C-7 (ダーク目視。本分析で主要画面は目視し問題なし)、C-9 (チャート 6 系列以上の色。本撮影データは 2 系列のみで未検証)。
- 新規 (本分析で発見): H-1 (1611% バー)、H-2 (再試行ボタン表記→原寸確認で誤字なしとして取り下げ)、AS-1 (ファンド名省略)、SE-2 (検索空の説明長)、CSV-1 (状態文の冗長)、ST-2 (UA 生表示)、AD-1 (メール折り返し)、RE-4 (空時の条件行)。X-1 も H-2 に伴い取り下げ。
