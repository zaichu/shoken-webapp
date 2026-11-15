# 画面インベントリ(最終・重複なし)

対象: `frontend/src/app.rs` のルート分岐と `frontend/src/features/`・`frontend/src/ui/`。用語は `frontend/src/ui/labels.rs` を正とする。旧 URL は別画面に数えない。クエリ・ハッシュは別画面に数えない。Phase 1 の2資料(`docs/design/work/phase1-codex/screen-inventory.md`、`docs/design/work/phase1-opencode/screens.md`)の数え方の差(30 対 11種)は粒度の違いであり、ここでは下記に統一する。矛盾点は Phase 6 と `labels.rs` に従い解消した(構成比の基準は評価額/年間配当、`docs/design/work/phase1-opencode/screens.md` の「取得額構成比」の記載は不採用。株保有 ID は独立画面でなく銘柄検索への置換)。

## 1. ルート(機能画面 12 + 旧 URL 別名)

| # | パス | 画面 | 認証 | 備考・到達 |
|---|---|---|---|---|
| 1 | `/` | ホーム(`HOME_TITLE`「ホーム」) | 保護。未ログインは `LoginPrompt` | 下部/上部ナビ。概要帯・配当カード・構成・損益上位・最近の配当 |
| 2 | `/search`(`?code=`) | 銘柄検索(`STOCK_SEARCH_TITLE`「銘柄検索」) | 公開 | 検索欄・結果・保有情報・外部リンク8件(`STOCK_LINK_*`) |
| 3 | `/receipts`(`?kind=`) | 取引明細(`RECEIPTS_TITLE`「取引明細」。タブ: 配当金/国内株式/投資信託) | 保護 | タブ切替・検索・期間/口座/商品条件・並べ替え・行詳細 |
| 4 | `/csv`(`?kind=`) | CSV管理(`CSV_MANAGE_TITLE`「CSV管理」。5種類: 資産2+明細3) | 保護 | 取得カード・5行の表・行メニュー・取込/削除確認 |
| 5 | `/assetbalance` | 資産管理(`ASSET_BALANCE_TITLE`「資産管理」。タブ: すべて/国内株式/投資信託) | 保護 | 検索・ソート・詳細・構成比入口 |
| 6 | `/assetbalance/composition` | 構成比(基準: 評価額/年間配当。区分: 銘柄別/商品別/口座別) | 保護 | 資産管理の「構成比を見る」から。円図+凡例+順位一覧 |
| 7 | `/assetbalance/holdings/:id` | 旧保有株詳細 URL(独立画面なし。`/search?code=` に履歴置換) | 保護 | 資産行・旧リンクから。読込骨格・失敗/再試行のみ |
| 8 | `/assetbalance/funds/:id` | 保有ファンド詳細(`ASSET_FUND_DETAIL_TITLE`) | 保護 | 投信行から。価格・基準日・数量・損益・口座 |
| 9 | `/settings`(`#account|display|data|security|devices|login-history|danger`) | 設定(`SETTINGS_TITLE`「設定」。7セクション) | 保護 | 左目次(PC)/縦積み(スマホ)。テーマ・ソート・初期タブ・金額単位・出力・通知・端末・履歴・危険操作 |
| 10 | `/admin/logins` | ログイン一覧(管理者)(`ADMIN_LOGINS`) | 管理者のみ。非管理者は NotFound、未ログインは `/login` | メール・場所検索・条件パネル・期間・ソート |
| 11 | `/login` | ログイン(`LOGIN_TITLE`「ログイン」) | 公開。ログイン済みは `/` へ | 中央カードのみ |
| 12 | `/404`・未知パス | NotFound(`NOT_FOUND_TITLE`) | 公開。下部ナビなし | ホーム/主要機能リンク。未知パスは `/404` に置換 |

旧 URL 別名(独立画面なし。対応する現行画面を開く): `/receipts/conditions`→取引明細、`/receipts/csv`・`/receipts/csv/delete`・`/assetbalance/csv`・`/assetbalance/csv/delete`→CSV管理。削除操作は確認なしに実行しない。`/settings/<unknown>`・`/admin/<unknown>` はルート解決上 NotFound だが認証前は保護判定で `/login` に置換される。

## 2. タブ・セクション(ルート内の切替表示)

- 取引明細 3タブ: 配当金/国内株式/投資信託(帯の主役は配当=`RECEIVED_DIVIDEND_AFTER_TAX`、株・投信=`REALIZED_AFTER_TAX`)。
- 資産管理 3タブ: すべて/国内株式/投資信託(帯項目はタブで末尾追加。順序固定)。
- 構成比 2基準: 評価額/年間配当(`COMPOSITION_BASIS_LABEL`)。区分切替 3種(`COMPOSITION_GROUP_HOLDING` 銘柄別/`COMPOSITION_GROUP_PRODUCT` 商品別/`COMPOSITION_GROUP_ACCOUNT` 口座別)はコードに存在するが Phase 5 時点でデザインなし(未)。
- 設定 7セクション: アカウント/表示/データ/セキュリティ/登録端末/ログイン履歴/危険操作。
- 保有銘柄詳細(`/search?code=` 内の保有セクション、`STOCK_HOLDING_SECTION`)は別ルートでなく検索結果の一部。

## 3. ダイアログ・パネル類(9種)

| 種別 | 用途 | PC/スマホ |
|---|---|---|
| 保有株詳細ダイアログ | 国内株式一覧からの詳細表示 | PC中央/スマホシート |
| 取引明細の行詳細 | 税引前・税額・税引後の内訳(配当18行・株式3行・投信2行) | PC右パネル相当/スマホシート |
| ホーム配当詳細ダイアログ | 配当金・税額・税引後・口座・日付 | PC中央/スマホシート |
| 検索・条件追加パネル | 期間(全期間/年/年月/日/範囲)・口座・商品。期間は下書き確定、他は即時反映 | PCポップオーバー/スマホ下部シート |
| CSV行メニュー | 種別ごとの再取得/ファイル取込/削除 | PC表内メニュー/スマホ行メニュー |
| 削除確認 | CSV 6件(明細3+資産個別2+一括1)+アカウント1件の計7件。対象・件数・取消不能を明示(`ConfirmDeleteModal` 系) | 中央ダイアログ |
| サイト共通アカウントメニュー | 設定・CSV出力・ログイン履歴/管理者・ログアウト・アカウント削除 | PCヘッダー/スマホアバターメニュー |
| アカウント削除確認 | パスワード/確認入力後に確定 | モーダル |
| 管理者検索パネル | メール・場所・条件・期間・ソート | PCポップオーバー/スマホ検索ダイアログ |

## 4. 状態(8群。直積でなく分岐の種類)

認証/権限(未ログイン→`/login`、管理者以外は NotFound)、アプリ起動(セッション読込スピナー)、データ一覧(読込スケルトン/正常/成功0件/取得失敗・再試行/前回データ保持の再取得エラー。明細 `Loading/Ready/Failed`)、ホームのカード単位独立(骨格/空/値あり/失敗を混在可)、検索/条件(未入力案内/入力中/処理中/結果/該当なし/エラー/絞込件数/チップ有無/下書き・確定・並べ替え)、資産数値/価格(正・負・ゼロ/欠測 `MISSING`「—」/更新待ち `PENDING_UPDATE`/基準価額・配当推定なし/基準額0で算出不可/行上限)、CSV処理(待機/取得中/取込中/急減確認/取込完了/スキップ/失敗)、設定/アカウント操作(読込中/保存中/保存失敗/保存済み/通知/端末失効・全端末ログアウト確認と失敗)、幅/テーマ(1024px=`lg` で表・カード、ポップオーバー・シート切替。ライト/ダーク)。

根拠コード: `frontend/src/app.rs:84-123,354-426`、`frontend/src/urls.rs:1-14`、`frontend/src/features/asset_balance/view/`、`frontend/src/features/receipts/`、`frontend/src/features/stock_search/`、`frontend/src/features/csv_page.rs`、`frontend/src/features/settings.rs`・`settings/login_history.rs`、`frontend/src/features/admin_logins.rs`、`frontend/src/features/home/view.rs`、`frontend/src/ui/`。
