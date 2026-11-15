# 実装計画(順番・規模・対応・リスク)

本番コードは変えない(Phase 7 時点)。下記は設計を実装に移す際の順番・規模・既存部品対応・リスクの計画。数値・文言の正本は `docs/design/work/phase5-codex/data/fixtures.json`・`frontend/src/ui/labels.rs`。1 PR は 10ファイル前後に分ける。

## 順番(スマホ先行→PC詳細パネル→ダーク)

1. **PR-1 基盤**: tokens・書体・焦点・タップ領域。Phase 4 Codex `tokens.css` を正にし、Phase 4 OpenCode 差分(名前・ダーク正数色)を対応表で吸収。`.numeric` を `font-mono` から本文書体+`tabular-nums` に統一(高-3)。帯上損失色トークン追加(light/dark 4.5:1検算)。
2. **PR-2 共通部品の置換**: `EmptyState`・`ListSkeleton`・`ListLoadError`・`SearchCount`・チップ・`Segmented`・`UnderlineTabs` の適用。M-4・M-10 を解消。
3. **PR-3 ホーム(スマホ)**: 集計帯・配当カード・月別棒・内訳・損益上位・最近の配当。H-5(符号nowrap・月軸12px+間引き)・H-1(実績>予想の分岐)・M-3(単月集中方針)・M-12(税額扱い)を反映。
4. **PR-4 資産管理(スマホ→PC表)**: 3タブ・帯順序固定・検索・ソート・構成比入口一本化。H-2(¥0の符号・色なし)・M-6(backdrop+閉じる+クリア+適用後状態)・M-10 を反映。
5. **PR-5 構成比・投信詳細**: 2基準・凡例(金額+割合)・順位一覧。M-1(系列色テーマ不変)・M-5(切替/積みの二択)・L-4(凡例色不整合)を反映。区分切替(銘柄別/商品別/口座別)は新規設計が必要(×のため別途)。
6. **PR-6 取引明細(3タブ・条件・行詳細)**: 18/3/2行・12か月小計・年合計の検算を維持。ナビ誤遷移(H-1)の正リンクに差替。条件適用後チップ・件数、詳細の項目順=列順を反映。
7. **PR-7 CSV管理・設定・管理者・ログイン/404**: 5種類・行メニュー・削除確認7件・急減確認、設定7セクション・簡易目次、メール全文表示の方針維持。M-2(用語置換)・高-1(トークン直書き除去)・高-2(対応表)を反映。
8. **PR-8 ダイアログ・アクセシビリティ一括**: 全ダイアログに `role="dialog"`・`aria-modal`・`aria-labelledby`、閉じる一元化、フォーカストラップ・Esc・復帰先を実装(H-4・高-4)。外部リンク8件の並べ方(M-8)、閉じる重複・符号分断(M-11)を反映。axe に管理者ルート追加(AD-3)。
9. **PR-9 残件・検証**: 1024〜1279px 収まり、コントラスト実測、キーボード・読み上げ確認、fixtures 外名称の除去(M-13)、11px 等の直書き除去(M-7)。

## 規模目安

- PR-1: S(トークン・CSS 数ファイル)。PR-2: M(部品横断だが機械置換中心)。PR-3〜7: 各 M(画面単位。HTML 見本1枚+対応する view・CSS)。PR-8: M(ダイアログ横断)。PR-9: S(検証・微修正)。工数見積は S/M のみとし、時間見積は実装担当の計測に委ねる(Phase 6 M-9 の不足を繰越)。

## 既存部品との対応(抜粋)

| 設計の部品・クラス | 既存(`frontend/src/ui/`・`frontend/style/input.css`) | 扱い |
|---|---|---|
| 要点カード・集計帯 | `summary.rs` `SummaryBand`・`SummaryItem`(`Primary`・`Secondary`・`Neutral`・`Gain`・`Loss`) | 流用(損失色トークンのみ新設) |
| PCテーブル | `.asset-table`・`.receipt-table`(`input.css`) | 流用(右揃え・nowrap維持) |
| スマホ行・グループ行 | `list_row.rs` `ListRow*`・`ListGroupHeader`、`.list-group-header` | 流用 |
| タブ | `underline_tabs.rs`・`segmented.rs`・`page_skeleton.rs` `SkeletonTabs` | 流用 |
| 検索入口・件数・チップ | `search_entry.rs`・`search_dialog.rs`・`search_controls.rs`(`SearchCount`) | 流用(適用後表示を追加) |
| ダイアログ/シート | `dialog.rs` `DialogShell`・`detail_dialog.rs` `DetailDialog`・`confirm_modal.rs` | 拡張(aria・フォーカス管理を追加) |
| 空・読込・失敗 | `empty_state.rs`・`state.rs`(`ListSkeleton`・`ListLoadError`・`Alert`) | 流用 |
| 金額表示 | `amount.rs`(`AmountKind::Delta`・`Plain`)・`format.rs`・`labels.rs` | 流用(書体のみ修正) |
| グラフ | `.home-chart-*`・`.home-donut`・`.comp-*`(`input.css:1416-1523`) | 流用(スケール方針を追加) |
| 骨格・ナビ | `page_frame.rs`・`site.rs`(`BottomNav`・`Avatar`・`Menu`)・`page_skeleton.rs` | 流用(縦ナビは作らない) |

## リスク

- トークン名の二重管理(Phase 4 両案の差)。対策: Codex 名を正にし対応表を PR-1 に同梱する。
- 損失色・系列色のコントラスト未実測。対策: PR-1・PR-5 で light/dark とも 4.5:1 を検算し `foundations.md` の表に追記する。
- ダイアログのキーボード・フォーカス未検証。対策: PR-8 で実装注記(初期位置・トラップ・復帰・Esc)を必須化し画像または操作記録を添える。
- 1024〜1279px の表収まり未確認。対策: PR-9 で 1280px 以外の幅を撮影し直す。
- ナビ誤遷移(H-1)のまま写す危険。対策: PR-6 で正リンク(`receipts` 側)に差し替えるまで `href="#"`+「仮」注記にする。
- 区分切替・外部リンク8件・月別スケールは設計未決。対策: 未決のまま実装しない。要否を Issue に残す。
