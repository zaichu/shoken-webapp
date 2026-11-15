# 画面設計状況(Phase 5 時点)

○=見本あり過不足なし(軽微な手直しは別記)、△=見本あり一部不足、×=未作成。担当は Phase 5 の分担(Codex/OpenCode)。HTML・PNG は実在確認済みのみ。数値は `docs/design/work/phase5-codex/data/fixtures.json`・両 `data-check.md` が正(評価額 ¥28,593,148、配当税引後 ¥3,163,438、明細18/3/2行、最近の配当 2026/12/24 ¥10,957・2026/12/10 ¥23,905)。「設計済み」を実態より多く書かない。未は未と書く。

## Codex 担当

| 画面・状態 | 状況 | 担当 | HTML | PC/スマホ画像 | 備考(残件) |
|---|---|---|---|---|---|
| ホーム(集計帯・配当・構成・損益上位・最近の配当) | ○ | Codex | `docs/design/work/phase5-codex/html/home.html` | `docs/design/work/phase5-codex/shots/home-pc.png`・`home-mobile.png`・`home-dark.png` | H-5(符号分断・月軸潰れ)・M-3(単月集中)・M-12(税引前省略)の手直し要 |
| 資産管理・すべて/国内株式/投資信託(一覧・詳細パネル) | ○ | Codex | `docs/design/work/phase5-codex/html/assets.html` | `docs/design/work/phase5-codex/shots/assets-pc.png`・`assets-mobile.png`・`assets-dark.png` | H-2(投信¥0を「—」)要修正。2件+1件・小計あり |
| 構成比・評価額基準 | ○ | Codex | `docs/design/work/phase5-codex/html/composition.html` | `docs/design/work/phase5-codex/shots/composition-pc.png`・`composition-mobile.png`・`composition-dark.png` | M-1(系列色)・M-5(タブ/積み未定義)要整理 |
| 構成比・年間配当基準 | ○ | Codex | `docs/design/work/phase5-codex/html/composition.html#dividend` | 同上 | 同上 |
| 構成比の区分切替(銘柄別/商品別/口座別) | × | — | なし | なし | 未デザイン。`COMPOSITION_GROUP_*` の意匠なし |
| 投資信託詳細 | ○ | Codex | `docs/design/work/phase5-codex/html/fund-detail.html` | `docs/design/work/phase5-codex/shots/fund-detail-pc.png`・`fund-detail-mobile.png`・`fund-detail-dark.png` | 実画面の更新待ち状態なし |
| 銘柄検索(結果あり・保有あり) | ○ | Codex | `docs/design/work/phase5-codex/html/stock-search.html` | `docs/design/work/phase5-codex/shots/stock-search-pc.png`・`stock-search-mobile.png`・`stock-search-dark.png` | M-8(外部リンク8件の並べ方未決。現行はダミー1件) |
| 銘柄検索(初期・該当なし・失敗・未ログイン) | △ | Codex | `docs/design/work/phase5-codex/html/states-codex.html`(静的カードのみ) | `docs/design/work/phase5-codex/shots/states-codex-pc.png`・`states-codex-mobile.png` | 検索画面自体の状態なし |
| 旧保有詳細URL→銘柄検索の置換 | ○ | Codex | `docs/design/work/phase5-codex/html/holding-route.html` | `docs/design/work/phase5-codex/shots/holding-route-pc.png`・`holding-route-mobile.png` | ダークは `holding-route-dark.png` 未確認(存在はする) |
| 配当・保有株詳細ダイアログ | ○ | Codex | `docs/design/work/phase5-codex/html/dialogs.html` | `docs/design/work/phase5-codex/shots/dialogs-pc.png`・`dialogs-mobile.png`・`dialogs-dark.png`・`dialog-dividend-pc.png`・`dialog-holding-pc.png` 他 | H-4(セマンティクスなし)・M-11(閉じる重複・符号分断)要修正 |
| 資産の検索・並べ替え UI | ○ | Codex | `docs/design/work/phase5-codex/html/assets.html`(静的例示) | `docs/design/work/phase5-codex/shots/assets-pc.png` | M-6(重なり・閉じる等なし)・M-10(チップ・件数なし) |
| 読込中・空・0件・取得失敗 | △ | Codex | `docs/design/work/phase5-codex/html/states-codex.html` | `docs/design/work/phase5-codex/shots/states-codex-pc.png`・`states-codex-mobile.png`・`states-codex-dark.png` | 状態ボードのみ。実画面分岐なし。M-2⑤⑥(文言)・M-4(EmptyState不適合) |
| 一部価格未取得・更新待ち・長い名称・複数口座 | △ | Codex | 同上 | 同上 | M-13(fixtures外名称混入)。割合32.0%/68.0%・13.1%/86.9%は検算一致 |

## OpenCode 担当

| 画面・状態 | 状況 | 担当 | HTML | PC/スマホ画像 | 備考(残件) |
|---|---|---|---|---|---|
| 取引明細・配当金タブ(一覧・年/月グループ・集計帯) | ○ | OpenCode | `docs/design/work/phase5-opencode/html/receipts-opencode.html` | `docs/design/work/phase5-opencode/shots/receipts-dividend-pc-light.png`・`receipts-dividend-sp-light.png` | 18行・12か月・年合計一致 |
| 取引明細・条件パネル/行詳細 | △ | OpenCode | 同上 | `docs/design/work/phase5-opencode/shots/receipts-dividend-cond-pc-light.png`・`receipts-dividend-dialog-pc-dark.png` | SP条件・全行詳細の挙動は画像で未確認 |
| 取引明細・国内株式/投資信託タブ | ○ | OpenCode | 同上 | `docs/design/work/phase5-opencode/shots/receipts-stock-pc-light.png`・`receipts-stock-sp-light.png`・`receipts-fund-pc-light.png`・`receipts-fund-sp-light.png` | 3行(+¥99,017)・2行(+¥147,282)一致 |
| CSV管理(5種類・取得カード・取込結果・部分失敗) | ○ | OpenCode | `docs/design/work/phase5-opencode/html/csv-opencode.html` | `docs/design/work/phase5-opencode/shots/csv-pc-light.png`・`csv-sp-light.png` | 件数は行数から生成 |
| CSV管理(行メニュー・削除確認6件・急減確認) | △ | OpenCode | 同上 | `docs/design/work/phase5-opencode/shots/csv-delete-pc-dark.png`・`csv-plunge-pc-light.png` | 取得/取込/削除の処理中・取込全体/削除の失敗なし。フォーカス未設計 |
| 設定(7セクション) | △ | OpenCode | `docs/design/work/phase5-opencode/html/settings-opencode.html` | `docs/design/work/phase5-opencode/shots/settings-pc-light.png`・`settings-sp-light.png`・`settings-pc-dark.png` | 保存失敗・確認状態なし |
| 管理者ログイン履歴(表・検索・条件・並べ替え) | △ | OpenCode | `docs/design/work/phase5-opencode/html/admin-logins-opencode.html` | `docs/design/work/phase5-opencode/shots/admin-logins-pc-light.png`・`admin-logins-sp-light.png`・`admin-logins-pc-dark.png` | 「+場所」が非操作要素。0件は状態ボードに掲載 |
| ログイン | ○ | OpenCode | `docs/design/work/phase5-opencode/html/login-opencode.html` | `docs/design/work/phase5-opencode/shots/login-pc-light.png`・`login-sp-light.png`・`login-pc-dark.png` | 中央カードのみ |
| 404 | ○ | OpenCode | `docs/design/work/phase5-opencode/html/notfound-opencode.html` | `docs/design/work/phase5-opencode/shots/notfound-pc-light.png`・`notfound-sp-light.png`・`notfound-pc-dark.png` | 見出し+ホーム導線 |
| アカウントメニュー/削除確認 | △ | OpenCode | `docs/design/work/phase5-opencode/html/account-opencode.html` | `docs/design/work/phase5-opencode/shots/account-pc-light.png`・`account-sp-light.png`・`account-delete-pc-dark.png` | 開閉の属性更新・キーボード動作・フォーカス未提示 |
| 状態ボード(読込・空・失敗・部分失敗・未ログイン・権限なし・端数データ) | △ | OpenCode | `docs/design/work/phase5-opencode/html/states-opencode.html` | `docs/design/work/phase5-opencode/shots/states-pc-light.png`・`states-sp-light.png`・`states-pc-dark.png` | CSV/設定/端末操作の具体的状態なし。コントラスト・キーボード未検証 |

## 未確認(両担当共通)

- 実ブラウザ・実機表示、1024〜1279px の表の収まり、コントラスト実測、キーボード・フォーカス・読み上げ順、44px 未満タップの有無。
- 1024px・1440px・1920px の全幅網羅、スクロール後・シート展開時の全状態。
