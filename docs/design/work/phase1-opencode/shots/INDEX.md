# 撮影一覧 (Phase 1 OpenCode)

ファイル名規則: `<画面キー>-<状態>-<pc|mobile>-<light|dark>.png`。スクロール後は末尾 `-scrolled.png`。

## 撮影条件

- PC: 1280×800、モバイル: 390×844
- ライト/ダーク両方
- データは `frontend/scripts/ui-shots/data.mjs` と `screens.mjs` の架空モック。実データ・本番 API は不使用
- 配信: `trunk build --release` → `prepare-dist.mjs` → `serve-dist.mjs` (localhost)
- 各 PNG は量子化済み (256 色)。最大 71KB、平均 31KB、合計 5.1MB

## 撮影済み (166 枚: 基本 122 + スクロール後 44)

| 画面キー | 状態 | 説明 | ファイル例 |
|---|---|---|---|
| home | default | ホーム (ログイン済み) | `home-default-pc-light.png` |
| home | loggedout | ホーム (未ログイン) | `home-loggedout-default-pc-light.png` |
| search | default | 銘柄検索 (結果あり, `?code=9901`) | `search-default-pc-light.png` |
| search | empty | 銘柄検索 (未検索) | `search-empty-default-pc-light.png` |
| receipts-dividend/stock/fund | default | 取引明細 3 タブ | `receipts-stock-default-pc-light.png` |
| receipts | empty | 取引明細 (空) | `receipts-empty-default-pc-light.png` |
| receipts | error | 取引明細 (取得失敗) | `receipts-error-default-pc-light.png` |
| receipts | detail-dialog | 取引明細 行詳細ダイアログ (主にモバイル) | `receipts-detail-dialog-default-mobile-light.png` |
| receipts | search-ui | 取引明細 検索・条件 UI 展開 | `receipts-search-ui-default-pc-light.png` |
| asset-all/stock/fund | default | 資産管理 3 タブ | `asset-stock-default-pc-light.png` |
| asset | empty | 資産管理 (空) | `asset-empty-default-pc-light.png` |
| asset | error | 資産管理 (取得失敗) | `asset-error-default-pc-light.png` |
| asset | composition | 構成比詳細 | `asset-composition-default-pc-light.png` |
| asset | holding-stock | 国内株式 保有詳細 (銘柄画面へ置換) | `asset-holding-stock-default-pc-light.png` |
| asset | holding-fund | 投資信託 詳細 | `asset-holding-fund-default-pc-light.png` |
| asset | search-ui | 資産管理 検索 UI 展開 | `asset-search-ui-default-pc-light.png` |
| csv | default | CSV管理 (取引明細種別) | `csv-default-pc-light.png` |
| csv-asset | default | CSV管理 (資産管理種別) | `csv-asset-default-pc-light.png` |
| csv-mutualfund | default | CSV管理 (投資信託, 0件) | `csv-mutualfund-default-pc-light.png` |
| settings | default | 設定 | `settings-default-pc-light.png` |
| settings | data | 設定 データセクション直行 | `settings-data-default-pc-light.png` |
| settings-login-history | default | 設定 ログイン履歴と登録端末 | `settings-login-history-default-pc-light.png` |
| admin-logins | default | 管理者 ログイン一覧 | `admin-logins-default-pc-light.png` |
| admin-logins | empty | 管理者 ログイン一覧 (検索0件) | `admin-logins-empty-default-pc-light.png` |
| account-menu | default | アカウントメニュー展開 | `account-menu-default-pc-light.png` |
| login | default | ログイン (未ログイン) | `login-default-pc-light.png` |
| notfound | default | 404 | `notfound-default-pc-light.png` |

## 撮れなかった画面・状態 (理由)

| 画面/状態 | 理由 |
|---|---|
| 読み込み中 (skeleton) | モックが即時応答するため、人為的に遅延させた取得の再現は今回の撮影スクリプトでは行っていない。a11y spec (`frontend/e2e/accessibility/a11y.spec.ts`) で同等状態を検証している |
| 資産管理/取引明細の取得失敗以外の全タブ失敗 | dividend 失敗のみ撮影。他タブの失敗は API モック差し替えで可能だが、同一 UI (`ListLoadError`) のため省略 |
| 条件パネル (期間/口座/商品) の個別展開 | PC の条件パネルは `receipts-search-ui` で口座ダイアログのみ撮影。期間・商品の個別小窓は操作が複雑で未撮影 |
| CSV 削除確認モーダル (`ConfirmDeleteModal`) | 削除メニュー操作が必要。UI コンポーネントは `ui/confirm_modal.rs` で共通。未撮影 |
| アカウント削除モーダル | 設定の危険操作。未撮影 |
| 銘柄検索の取得失敗 | モックで `stock` を空/結果のみ。エラー応答の再現は未実施 |
| モバイルの PC 専用 UI (構成比リンク上バー) | `max-lg:hidden` のためモバイルでは出ない仕様 |
| 実機ブラウザでの表示 | 本番ブラウザ・フォント差は未確認 (CI コンテナ推奨) |

## 未確認

- 画像の視覚的な確認 (レイアウト崩れ等) は本モデルが画像入力非対応のため未実施。構造・数値はコードと a11y spec で確認した。
- 撮影画像のファイルサイズは 300KB 以下を確認済み。
