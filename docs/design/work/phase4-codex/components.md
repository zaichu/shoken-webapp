# 共通部品

本書は Phase 4 のデザイン提案。ラベルと金額・日付表記は `frontend/src/ui/labels.rs`、`frontend/src/ui/format.rs` を正本とする。根拠: `docs/design/mobile-ui-rules.md:14-76`、`docs/ui-design.md:1-75`、`frontend/README.md` の共通部品表、`frontend/src/ui/`。

| 部品 | 見た目・使い方 | 既存部品との対応 | 通常 / ホバー / フォーカス / 無効 / エラー |
|---|---|---|---|
| PC 上ナビ | 紺の帯に主要画面、選択位置は橙の線 | `PageFrame`, `BottomNav`, `ui/site.rs` | 通常=白文字 / hover=明るい面 / focus=3pxリング / 無効=項目を出さない / エラー=該当なし |
| スマホ下部ナビ | 4項目を均等配置、各タップ領域44px以上 | `BottomNav` (`ui/site.rs`) | 通常=補助文字 / hover=浮き面 / focus=リング / 無効=項目を出さない / エラー=該当なし |
| アカウントメニュー | 右端のアバターから開くメニュー | `Avatar`, `Menu` (`ui/avatar.rs`, `ui/button.rs`) | 通常=面 / hover=沈み面 / focus=リング / 無効=薄くして操作不可 / エラー=操作結果を文で提示 |
| ボタン | 主操作=塗り、副操作=枠、静かな操作=Ghost、破壊操作=DangerSolid | `Button`, `LinkButton`, `ButtonVariant` (`ui/button.rs`) | 通常=variant色 / hover=明度変化 / focus=リング / 無効=opacityとdisabled / エラー=エラー文と関連付け |
| 入力・選択 | ラベルを常時表示。エラーは入力直下に文で表示 | `Select`, `SearchTextInput`, `SearchBox` (`ui/select.rs`, `ui/search_controls.rs`) | 通常=罫線 / hover=濃い罫線 / focus=リング / 無効=沈み面 / エラー=エラー色罫線+文 |
| タブ | 画面種別は全幅下線タブ、押した設定はSegmented | `TabButton`, `UnderlineTabs`, `Segmented` (`ui/underline_tabs.rs`, `ui/segmented.rs`) | 通常=本文色 / hover=沈み面 / focus=リング / 無効=薄く残す / エラー=該当なし |
| 検索入口 | スマホでは検索・並べ替えの1ボタン、条件と件数を直下に表示 | `SearchEntry`, `SearchDialog`, `SearchCount` (`ui/search_entry.rs`, `ui/search_dialog.rs`, `ui/search_controls.rs`) | 通常=枠ボタン / hover=沈み面 / focus=リング / 無効=条件がない場合も入口を維持 / エラー=入力近くに案内 |
| 要点カード | 主数値1つと補助値。ラベルと時点を省略しない | `SummaryBand`, `SummaryItem`, `Card` (`ui/summary.rs`, `ui/card.rs`) | 通常=面 / hover=操作可能な場合だけ影 / focus=リング / 無効=操作なし / エラー=該当値に状態文 |
| 一覧カード | スマホではカードで囲まず行を連ねる。詳細はPC右パネル | `ListRow`, `ListRowLink`, `ListRowButton`, `DetailDialog` (`ui/list_row.rs`, `ui/detail_dialog.rs`) | 通常=区切り線 / hover=沈み面 / focus=リング / 無効=静的行 / エラー=一覧エラー部品を使う |
| PC テーブル | 数値列右揃え、見出しと値の右端をそろえる。必要な画面で右詳細パネル | `.asset-table`, `.receipt-table` (`frontend/style/input.css`) | 通常=罫線 / hover=行背景 / focus=行操作にリング / 無効=操作を無効表示 / エラー=表の直上に文 |
| スマホ行 | 1段目に名称・主額、2段目に日付・補助情報 | `ListRow*` (`ui/list_row.rs`) | 通常=区切り線 / hover=沈み面 / focus=リング / 無効=静的行 / エラー=一覧エラー部品 |
| ダイアログ / シート | PC は中央、スマホは下から。タイトル、内容、操作の順 | `DialogShell`, `DetailDialog`, `ConfirmDeleteModal` (`ui/dialog.rs`, `ui/detail_dialog.rs`, `ui/confirm_modal.rs`) | 通常=面と影 / hover=操作ボタンのみ / focus=リング / 無効=操作を残して無効化 / エラー=内容の先頭に文 |
| アイコン | 線画を標準とし、単独操作にはアクセシブル名を付ける | `Avatar`, `SecurityCodeLink`, ボタン内 SVG (`ui/avatar.rs`, `ui/security_link.rs`) | 通常=本文色 / hover=強調色 / focus=親のリング / 無効=薄色 / エラー=色だけで状態を示さない |
| 空状態 | アイコン・見出し・説明・次の操作 | `EmptyState` (`ui/empty_state.rs`) | 通常=案内面 / hover=操作のみ / focus=操作にリング / 無効=主操作を無効表示 / エラー=失敗時は別部品 |
| 読み込み中 | 行の形を保つスケルトン、読み込み中であることを支援技術に伝える | `ListSkeleton` (`ui/state.rs`) | 通常=中立面 / hover=なし / focus=なし / 無効=なし / エラー=失敗後に置換 |
| エラー | 理由と再読み込み操作を同じ領域に表示 | `ListLoadError`, `Alert` (`ui/state.rs`) | 通常=エラー面と文章 / hover=再試行のみ / focus=リング / 無効=再試行中は無効 / エラー=追加失敗は文を更新 |

ボタン・入力・行操作の最小タップ領域を44px確保する。通常文字のコントラスト4.5:1以上を保つ。ホバーだけで情報や操作を出さず、キーボードフォーカスを常に視認可能にする。増減表示は `AmountKind::Delta` に寄せ、色のみの意味伝達を避ける。データ時点は「取込日」と「価格基準日」を別ラベルで年まで表示する。月次金額は離散値の棒グラフにする。金額・内訳のサンプル値も合計一致を保つ。

実装根拠: `frontend/src/ui/amount.rs`, `format.rs`, `labels.rs`, `button.rs`, `list_row.rs`, `summary.rs`, `page_frame.rs`, `site.rs`, `search_entry.rs`, `detail_dialog.rs`, `empty_state.rs`, `state.rs`。画面の問題根拠: `docs/design/work/phase2-codex/ux-audit-codex.md` の「画面をまたぐ課題」「観点別の確認メモ」。
