# Phase 1 (OpenCode): 画面一覧

調査基準: `main` の `frontend/src`。ルーティングは `frontend/src/app.rs` の `route_for_path` (L84-106) と `protected_path` (L112-120) を正本とした。画面部品・状態は各 feature の view を確認した。

## 1. ルート一覧

| # | パス | Route | 画面 (実装) | 認証 | 備考 |
|---|---|---|---|---|---|
| 1 | `/` | Home | `HomePage` (`frontend/src/features/home/view.rs:77`) | 保護 | ログイン済み: ダッシュボード。未ログイン: `LoginPrompt` |
| 2 | `/search` | Search | `StockSearchPage` (`frontend/src/features/stock_search/view.rs:68`) | 公開 | `?code=` で銘柄詳細相当を表示 |
| 3 | `/receipts` | Receipts | `ReceiptsPage` (`frontend/src/features/receipts/view.rs:25`) | 保護 | `?kind=` なし。タブで配当/株式/投信 |
| 4 | `/receipts/conditions` | Receipts | 同上 | 保護 | 旧URL互換 (app.rs L89-90) |
| 5 | `/csv` | Csv | `CsvPage` (`frontend/src/features/csv_page.rs:119`) | 保護 | `?kind=` で初期種別 |
| 6 | `/receipts/csv`, `/receipts/csv/delete`, `/assetbalance/csv`, `/assetbalance/csv/delete` | Csv | 同上 | 保護 | 旧URL互換 (app.rs L92-96) |
| 7 | `/assetbalance` | AssetBalance | `AssetBalancePage` (`frontend/src/features/asset_balance/view.rs:261`) | 保護 | タブ: すべて/国内株式/投資信託 |
| 8 | `/assetbalance/composition` | AssetBalanceComposition | `AssetBalanceCompositionPage` | 保護 | 取得額構成比 |
| 9 | `/assetbalance/holdings/:id` | AssetBalanceHolding | `AssetBalanceHoldingPage` (`holding_detail_page.rs:15`) | 保護 | 銘柄画面(`/search?code=`)へ置換 |
| 10 | `/assetbalance/funds/:id` | AssetFundHolding | `AssetFundHoldingPage` (`fund_detail_page.rs:20`) | 保護 | 投資信託の詳細 |
| 11 | `/settings` | Settings | `SettingsPage` (`frontend/src/features/settings.rs:143`) | 保護 | `#id` でセクション直行 |
| 12 | `/admin/logins` | AdminLogins | `AdminLoginsPage` (`frontend/src/features/admin_logins.rs:202`) | 管理者のみ | 非管理者は 404 扱い (app.rs L361-363) |
| 13 | `/login` | Login | `LoginPage` (`frontend/src/features/login.rs:9`) | 公開 | 未ログイン専用。ログイン済みは `/` へ |
| 14 | `/404` および不明パス | NotFound | `NotFoundPage` (`frontend/src/features/not_found.rs:14`) | 公開 | 404 画面 |

合計: ルート 14 個 (うち旧URL互換 6 個を含む)。機能画面としては 11 種類。

## 2. サブ画面・タブ・詳細

| 親画面 | サブ | 根拠 |
|---|---|---|
| ホーム | ログイン済みダッシュボード / 未ログイン `LoginPrompt` | `home/view.rs` L120-128 |
| 資産管理 | タブ: すべて / 国内株式 / 投資信託 | `asset_balance/view.rs` (AssetKind::ALL) |
| 資産管理 | 構成比詳細 `/assetbalance/composition` | `app.rs` L98 |
| 資産管理 | 国内株式の保有詳細 → 銘柄検索へ置換 | `holding_detail_page.rs` L12-13 |
| 資産管理 | 投資信託の詳細 `/assetbalance/funds/:id` | `fund_detail_page.rs:20` |
| 取引明細 | タブ: 配当金 / 国内株式 / 投資信託 | `receipts` (ReceiptsTab) |
| 取引明細 | 行詳細 `DetailDialog` (モバイルカード) | `receipts/view/cards.rs` L479 |
| 銘柄検索 | 空状態 / 検索結果 / 保有セクション | `stock_search/view.rs` L380-333 |
| CSV管理 | 種別タブ? クエリ `kind=` で初期選択。5 種類の行 | `csv_page.rs` CsvKind::ALL |
| 設定 | セクション: アカウント/表示/データ/セキュリティ/登録端末/ログイン履歴/危険 | `settings.rs` L194-203 |
| 管理者 | ログイン一覧 (テーブル PC / リスト モバイル) | `admin_logins.rs` |

## 3. ダイアログ・シート・モーダル

| 名称 | 用途 | 根拠 |
|---|---|---|
| `DetailDialog` (`ui/detail_dialog.rs`) | 取引明細の行詳細 | `receipts/view/cards.rs:479` |
| `SearchDialog` (`ui/search_dialog.rs`) | 取引明細の検索・並べ替え (モバイル) | `receipts` view |
| `ConditionPanel` (`ui/search_controls.rs`) | 期間/口座/商品の条件追加 | `ui-design.md` §5 |
| `ConfirmDeleteModal` (`ui/confirm_modal.rs`) | CSV 削除確認 | `ui/confirm_modal.rs` |
| `AccountDeleteDialog` (`ui/site.rs`) | アカウント削除確認 | `site.rs:185` |
| `AccountMenu` (サイトヘッダー) | ログアウト・エクスポート等 | `site.rs:139` |
| `MobileSearchEntryShell` | 資産管理/管理者のモバイル検索入口 | `asset_balance`/`admin_logins` view |

## 4. 状態 (各画面で出現する)

- 読み込み中: `ListSkeleton` (`ui/state.rs`)。例: 資産管理 L? `list-skeleton`、取引明細。
- 取得失敗: `ListLoadError` (再読み込み付き)。例: a11y spec L? 
- 空: `EmptyState` (`ui/empty_state.rs`)。例: 資産管理「資産管理データがありません」。
- ログイン済み/未ログイン: `Show`/`LoginPrompt`。ホーム・資産管理保護ルートは未ログインで `/login` へ (app.rs L370-380)。
- ダーク/ライト: `setting_theme` / `colorScheme`。

## 5. 撮影カバレッジ

- 撮影 PNG は `shots/` に 162 枚 (PC 1280×800 とモバイル 390×844、ライト/ダーク)。
- 画面・状態の対応表は `shots/INDEX.md`。
- 撮れなかった画面/状態は `shots/INDEX.md` の「未撮影」に理由付きで記載。
