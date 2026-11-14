# Phase 5 Codex 画面・状態の状況

## 集計

- HTML: 8ページ。下表で「デザイン済み」としたものはリンク先HTMLに表示内容がある。
- PC/スマホ: 代表画像あり。ダークは記載のある画面のみ。
- 今回の依頼対象外画面は未作成であり、デザイン済みとは数えない。

## 画面・状態

| 画面・状態 | 状況 | HTML | PC / スマホ画像 | 備考 |
|---|---|---|---|---|
| ホーム | デザイン済み | [home.html](html/home.html) | [PC](shots/home-pc.png) / [スマホ](shots/home-mobile.png) / [ダーク](shots/home-dark.png) | 集計帯、配当、資産構成、最近の配当 |
| 資産管理・すべて | デザイン済み | [assets.html](html/assets.html) | [PC](shots/assets-pc.png) / [スマホ](shots/assets-mobile.png) / [ダーク](shots/assets-dark.png) | 一覧・詳細パネル |
| 資産管理・国内株式 | デザイン済み | [assets.html#stocks](html/assets.html#stocks) | [PC](shots/assets-pc.png) / [スマホ](shots/assets-mobile.png) | 2件の行と小計を表示 |
| 資産管理・投資信託 | デザイン済み | [assets.html#funds](html/assets.html#funds) | [PC](shots/assets-pc.png) / [スマホ](shots/assets-mobile.png) | 1件の行と小計を表示 |
| 構成比・評価額基準 | デザイン済み | [composition.html](html/composition.html) | [PC](shots/composition-pc.png) / [スマホ](shots/composition-mobile.png) / [ダーク](shots/composition-dark.png) | クラス別の評価額・割合 |
| 構成比・年間配当基準 | デザイン済み | [composition.html#dividend](html/composition.html#dividend) | [PC](shots/composition-pc.png) / [スマホ](shots/composition-mobile.png) | 年間配当の内訳を表示 |
| 投資信託詳細 | デザイン済み | [fund-detail.html](html/fund-detail.html) | [PC](shots/fund-detail-pc.png) / [スマホ](shots/fund-detail-mobile.png) / [ダーク](shots/fund-detail-dark.png) | 保有口数等 |
| 銘柄検索 | デザイン済み | [stock-search.html](html/stock-search.html) | [PC](shots/stock-search-pc.png) / [スマホ](shots/stock-search-mobile.png) / [ダーク](shots/stock-search-dark.png) | 結果表示 |
| 旧保有詳細URLから銘柄検索への置換 | デザイン済み | [holding-route.html](html/holding-route.html) | [PC](shots/holding-route-pc.png) / [スマホ](shots/holding-route-mobile.png) / [ダーク](shots/holding-route-dark.png) | 9901の保有情報 |
| 配当・保有株詳細ダイアログ | デザイン済み | [dialogs.html](html/dialogs.html) | [PC](shots/dialogs-pc.png) / [スマホ](shots/dialogs-mobile.png) / [ダーク](shots/dialogs-dark.png) | ダイアログ状態 |
| 読込・空・0件・エラー・部分欠測・更新待ち・長い名称・大きな値・複数口座・銘柄検索状態 | デザイン済み | [states-codex.html](html/states-codex.html) | [PC](shots/states-codex-pc.png) / [スマホ](shots/states-codex-mobile.png) / [ダーク](shots/states-codex-dark.png) | 静的な状態例 |
| 取引明細、CSV管理、設定、管理者ログイン、ログイン、404、その他の確認ダイアログ | 未作成 | — | — | 今回の担当範囲外 |

## 数値根拠

- 全体評価額 ¥28,593,148 = 国内株式 ¥9,149,807 + 投資信託 ¥19,443,341。
- 画面案の評価額・取得総額・損益等の検算は [data-check.md](data-check.md)。
- 画像は架空データを使った既存撮影で、今回の修正反映後に撮り直す。
