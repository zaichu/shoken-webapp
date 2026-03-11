# タスク名

資産管理 / 取引明細のレイアウト再配置

## 目的

`AssetBalance` と `Receipts` で、CSV 操作・検索オプション・集計情報・テーブルがすべて縦積みになっており、
主役であるテーブルが下に押し下げられて画面が窮屈に見える問題を解消する。

## 現状

- 資産管理 / 取引明細ともに、主な情報が `CSVファイル選択 -> 検索オプション -> 集計情報 -> テーブル` の順で縦に積まれている
- デスクトップ幅でもテーブルがファーストビューで見えづらい
- 補助情報がすべて上部に集まり、画面中央の視線誘導が弱い
- 画面全体として「管理画面」より「長いフォーム」に見えやすい

## 期待挙動

- デスクトップ幅では、テーブルを中央の主領域として先に視認できる
- CSV 操作・検索・集計は、上部に積み上げるのではなく、左右のサイド領域やコンパクトなツールバーへ再配置される
- `AssetBalance` と `Receipts` で同じレイアウト思想を使い、画面横断で統一感を出す
- モバイルでは従来どおり縦積みでもよいが、情報の優先順位が崩れない

## スコープ

- `AssetBalance` / `Receipts` のデスクトップ中心レイアウト見直し
- CSV 操作エリアの圧縮または再配置
- 検索オプションと集計情報の表示位置見直し
- テーブルを主役にするレイアウト構成への変更
- before / after スクショを見ながらの調整

## 非対象

- CSV import / preview / delete の挙動変更
- 検索ロジックや集計ロジックの仕様変更
- API 変更
- モバイル専用の大幅 redesign

## 対象ファイル

- `frontend/src/pages/AssetBalance.tsx`
- `frontend/src/pages/Receipts.tsx`
- `frontend/src/components/organisms/SearchCard/SearchCard.tsx`
- `frontend/src/components/organisms/AssetPortfolioSummary/AssetPortfolioSummary.tsx`
- `frontend/src/components/organisms/ReceiptTable/ReceiptTable.tsx`
- `frontend/src/styles/tailwind.css`

## 根拠情報

- ユーザー所感:
  - 画面が窮屈に感じる
  - いまは資産管理も取引明細も、CSVファイル選択 → 検索オプション → 集計情報 → テーブルの順に縦並び
  - テーブルを中央に置き、左右に情報を散らしたい
- 参照スクショ:
  - 取得日: 2026-03-11
  - 条件: `npm run ui:screenshot:auth` / Desktop Chrome / 1920x1080
  - `.playwright-mcp/assetbalance-initial.png`
  - `.playwright-mcp/assetbalance-search-security.png`
  - `.playwright-mcp/receipts-dividend-initial.png`
  - `.playwright-mcp/receipts-domestic-stock-initial.png`
  - `.playwright-mcp/receipts-mutualfund-initial.png`
- Codex コンセプトモック:
  - `output/design-concepts/receipts-workspace-concept.png`
  - `output/design-concepts/assetbalance-dashboard-concept.png`
  - 元 HTML:
    - `output/design-concepts/receipts-workspace-concept.html`
    - `output/design-concepts/assetbalance-dashboard-concept.html`
  - 共通 CSS:
    - `output/design-concepts/ui-concepts.css`

## デザイン思想

- 今回の 2 画面は「操作する画面」ではなく「結果を読む画面」として扱う
- desktop の first view では、ユーザーが最初に理解すべきものを左の main stage に置く
  - `Receipts`: 明細テーブル
  - `AssetBalance`: KPI と保有銘柄
- CSV / 検索 / 集計は「必要なときに見る道具」なので、右の utility rail に下げる
- 画面の格は `左 = stage`、`右 = tool tray` の役割分担で作る
- 現状のように「同じ重さのカードが上から順に並ぶ」見え方は避ける
- 情報設計の主従は `結果 > 状態 > 操作` にする

## 左 main / 右 rail にする理由

- 左上が視線の入口なので、主役を左に置くと画面の意味が最短で伝わる
- テーブルや保有カードは横幅に価値がある。広い main を左に確保した方が読みやすい
- 検索や CSV 操作は常時読む情報ではないため、右 rail に置いた方が主張が適切
- 左に操作、右に結果を置くと、左側だけが重くなりやすく、今回の「長いフォーム感」を強める
- 今回は「設定画面」ではなく「閲覧・確認画面」なので、左に結果、右に操作が合う
- 逆配置を採るのは、階層ナビ中心の画面や設定フロー中心の画面だけでよい

## UIレビュー所見（2026-03-11 / Codex）

- 高:
  - `Receipts` は CSV 操作だけが左レールへ移動したが、`ReceiptTemplate` 内の `SearchCard` と `ReceiptHeader` はまだテーブルの上に残っている。結果として `タブ -> 検索バー -> 集計バー -> テーブル` の縦積みが継続しており、主役のテーブルがファーストビューで弱い
  - `Receipts` は濃い見出しバーが `グローバルヘッダー / タブ / 検索 / 集計` と連続し、画面上部に視線が滞留する。情報の優先順位より「帯が何本もある」印象が先に来る
  - `AssetBalance` は 2 カラム化自体はされているが、main 側が `絞り込みバナー -> KPI -> 構成比` を 1 枚の大きいカード内に抱え込んでおり、主領域が強く見えない。特に絞り込み後は 1 銘柄カードに対して右側の余白が大きく、密度配分が悪い
  - `AssetBalance` / `Receipts` の両方で、補助 UI が dark header 付きカードとして強く主張しており、main より aside の見出しの方が目立つ瞬間がある
- 中:
  - `SearchCard` の `compact` は `lg:grid-cols-4` を外すだけで、sidebar 幅でも `sm:grid-cols-2` が残る。狭いレール内でドロップダウンが 2 列化しやすく、操作密度が崩れる
  - `Receipts` の empty state は main が広く空く一方で、左の CSV パネルだけが孤立して見える。データ有無でレイアウトの重心が揺れている
  - `AssetBalance` の「絞り込み中」表示は full-width banner になっており、main の先頭で縦方向の高さを余計に消費している
  - CSV 操作、検索、集計がすべて同系統のカード見た目で、役割差より箱の数が先に認識される

## 採用するレイアウト案

- desktop は 2 画面とも `main を左 / aside を右` に固定する
- desktop の基本骨格は `minmax(0, 1fr) + 280px〜320px aside` の 2 カラムにそろえる
- main は最初に見せたいコンテンツだけを置く
  - `Receipts`: テーブル
  - `AssetBalance`: KPI と保有銘柄一覧
- aside は補助操作専用にする
  - CSV 操作
  - 検索オプション
  - 集計情報
- desktop では「上部に full-width の dark header 付き card を積む」構造をやめる
- モバイルでは現状どおり縦積みでよいが、desktop だけは DOM を維持しつつ見た目の優先順位を明確に変える

## ビジュアル方針

- 濃いネイビーはグローバルヘッダーやアクティブタブなど、意味の強い場所に限定する
- `CardHeader` の濃色バーを連続させず、余白・ラベル・面のトーン差で区切る
- main は明るく広い paper 面、rail は少しトーンを落とした utility 面として差をつける
- 数値の強さは背景色の多用ではなく、タイポグラフィと余白で出す
- `Receipts` のタブは小さいリンクではなく、モード切り替えとして見える segmented control 寄りにする
- 適用中フィルタは form 内ではなく結果の近くに chip / status として出す
- 「絞り込み中」「件数」などの状態表示は横長の banner ではなく、compact な pill / inline status で表現する

## ページ別の設計

### `Receipts`

- タブの直下に `main + aside` を置く
- main にはテーブル card だけを置く。検索カードと集計カードは main 先頭に残さない
- aside は上から `CSV操作 -> 検索オプション -> 集計情報` の順で縦積みする
- 検索は desktop では常時展開でもよいが、sidebar 専用の 1 列レイアウトにする
- 集計情報は full-width の 3 カラム帯ではなく、aside 内の縦積み KPI もしくは 1 列 card に落とす
- 1365px 前後でもテーブルヘッダと 4〜6 行程度が自然に視界へ入ることを優先する

### `AssetBalance`

- `CSV操作 + 検索` は aside へまとめる
- main には `KPI strip` と `保有銘柄カード群` を置く
- 「絞り込み中」は banner ではなく、main 冒頭の小さい chip / inline status に縮約する
- KPI は現状の 4 枚横並びを維持してよいが、main の独立した帯として見せ、構成比カード群と 1 枚の大箱にまとめすぎない
- 保有銘柄カード群は `sm:1列 / xl:2〜3列` を基本にし、1件表示時に大きい空白だけが残る見え方を避ける

## コンポーネント別の改修指示

- `frontend/src/pages/Receipts.tsx`
  - ページ全体で desktop 用の `main + aside` 骨格を作る
  - `ReceiptTemplate` 側に検索・集計を持たせたままにせず、tab 本体から sidebar へ逃がせるようにする
- `frontend/src/components/templates/ReceiptTemplate.tsx`
  - `header` と `SearchCard` を常に table の上へ積む前提をやめる
  - `mainContent` / `sidebarContent` のような 2 スロット構成に寄せるか、desktop 時だけ順序を切り替えられる構造へ変える
- `frontend/src/components/organisms/SearchCard/SearchCard.tsx`
  - sidebar 用に「最大 1 列」のレイアウト指定を入れる
  - `compact` は gap だけでなく columns と action label も抑制する
  - desktop sidebar では `sm:grid-cols-2` を残さない
- `frontend/src/components/molecules/ReceiptHeader/ReceiptHeader.tsx`
  - desktop aside 用の compact variant を持たせ、横 3 枚の帯 UI をそのまま main 上に置かない
- `frontend/src/components/organisms/AssetPortfolioSummary/AssetPortfolioSummary.tsx`
  - 「絞り込み中」banner を縦に積まない
  - KPI と保有銘柄一覧の視覚的な主従を整理し、main の見出し面を強くする
- `frontend/src/pages/AssetBalance.tsx`
  - 現状の left aside を right aside に寄せ、`SearchCard` の compact 崩れを解消する

## 明確に避ける案

- `SearchCard` を閉じたままにするだけで解決したことにしない
- `Receipts` の main に `検索 -> 集計 -> テーブル` を残したまま、CSV 操作だけ移す案は不採用
- `AssetBalance` の main を 1 枚の大カードにしたまま、余白だけ増やす案は不採用
- sidebar 幅を狭いまま維持して、内部だけ無理に 2 列配置する案は不採用
- `左に操作 / 右に結果` の構成へ戻す案は不採用
- dark header 付き card を色違いで並べ直すだけの案は不採用

## 制約

- 既存の画面機能は維持する
- 既存の React/Tailwind 構成に合わせる
- モバイル表示は崩さない
- 画面ごとの差は残してよいが、情報配置の考え方は統一する

## 受け入れ条件

- [ ] `AssetBalance` でデスクトップ表示時に主コンテンツが主領域としてファーストビュー近くにある
- [ ] `Receipts` でデスクトップ表示時にテーブルが主領域としてファーストビュー近くにある
- [ ] CSV 操作・検索・集計が「すべて上に縦積み」の状態ではなくなっている
- [ ] `AssetBalance` と `Receipts` の情報配置に共通した設計意図がある
- [ ] モバイル幅でも破綻しない
- [ ] 最新スクショで before / after の差が説明できる
- [ ] 1365px 前後の desktop スクショで、`Receipts` はテーブルヘッダが自然に視界へ入る
- [ ] 1365px 前後の desktop スクショで、`AssetBalance` は main / aside の役割分担が見て分かる
- [ ] `Receipts` の desktop では検索カードと集計カードがテーブルの上に full-width で残っていない
- [ ] `SearchCard` の sidebar 表示で、desktop 幅でも検索 UI が 1 列で読める
- [ ] `Receipts` のタブがモード切り替えとして十分に目立ち、その直下に main/rail 本体が始まる
- [ ] 画面全体で dark header の帯が連続せず、`結果 > 操作` の視線順序が作れている

## 実行してほしい確認

- `cd frontend && npm test -- AssetBalance`
- `cd frontend && npm test -- Receipts`
- `cd frontend && npm run ui:screenshot:auth`

## Claude への依頼メモ

- スクショを見た上で、まずレイアウト案を 1 つに絞ってから実装する
- 今回は `main left / aside right` の 2 カラム案を採用し、別案には広げない
- `output/design-concepts/*.png` の方向性を優先し、迷ったらそちらに寄せる
- 「カードを足す」「余白を増やす」「初期状態で閉じる」だけで終わらせず、情報の置き場所自体を変える
- `Receipts` は `ReceiptTemplate` の責務を見直してでも、検索・集計を table 上から退かす
- `AssetBalance` は aside 化だけで終わらせず、main の見え方も整理する
- after スクショでは、初見で先に目に入るのが補助 UI ではなく main になっていることを説明できるようにする
- もし既存コンポーネント構造が邪魔なら、見た目の責務に合わせて page 側レイアウトを再編してよい

## Codex 実施内容（2026-03-12 時点）

- UI 再設計の一次実装は完了
  - `Receipts` / `AssetBalance` を `main left / utility rail right` の desktop レイアウトへ寄せた
  - `Receipts` の集計表示位置を main 上部へ移し、`AssetBalance` のサマリー表現と近い言語へ揃えた
  - `配当金 / 国内株式 / 投資信託` タブは、重すぎる segmented 表現から戻しつつ、モード切り替えとして見える見た目に調整した
  - 右 rail の幅と padding を広げ、窮屈さを軽減した
  - `PortfolioPieChart` 周辺は、`銘柄コード + 企業名` の横並び、`取得総額 / 取得単価 / 数量` の横一列 strip、説明文の削除など、保有カードをコンパクト寄りに再設計した
- CSV 保存後の結果表示を統一
  - `CsvSaveResultNotice` を新設し、`Receipts` / `AssetBalance` の保存完了通知をシンプルな notice に寄せた
  - 長い補助文言や冗長な説明文を削除し、CSV アップロード面の copy を短くした
- ヘッダーと本文コンテナ幅を共通化
  - `証券Web` ヘッダー、本文、フッターの横幅基準を統一した
- 資産管理 CSV の誤検知を修正
  - ユーザー提供サンプルで発生していた `保有数量［株］` エラーは、実データではなく `特定口座合計` 行を明細として読んでいたことが原因
  - backend 側で「全空行」と「口座合計行」を読み飛ばすよう修正した
  - 対応ファイル:
    - `backend/src/services/csv_import.rs`
    - `backend/src/services/asset_balance.rs`
- この内容は commit / push 済み
  - branch: `feature/ui-layout-rebalance-receipts-assetbalance`
  - commit: `a8513b2`

## 確認結果

- 通過
  - `cd backend && cargo fmt --check`
  - `cd backend && cargo test`
- 既知の未解消
  - `cd frontend && npm run ui:screenshot:csv-crud`
  - 4件とも失敗
  - 現在の失敗は CSV 内容ではなく、Playwright が `#csv-file-input` を見つけられていないこと
  - つまり E2E が現行 UI 構造に追随できていない

## Claude に残す作業

- E2E を現行 UI に追随させる
  - `frontend/e2e/csv-crud.spec.ts` のアップロード操作が古い selector 前提で止まっているので修正する
  - `#csv-file-input` 依存をやめ、現行コンポーネントに対して安定した selector を使う
  - 必要なら upload trigger 側に `data-testid` を追加して、UI 変更に強い E2E にする
- `npm run ui:screenshot:csv-crud` を通す
  - 対象 4 シナリオ:
    - 国内株式
    - 配当金
    - 資産管理
    - 投資信託
- 資産管理 CSV 修正の実画面確認
  - ユーザー提供サンプルで、`特定口座合計` 行を含んでも保存時にエラーにならないことを確認する
  - preview / save の両方で余計な行エラーが出ないことを確認する
- UI polish の残件があれば続ける
  - まずは E2E を直してから判断する
  - いまの優先順位は `見た目の追加改善` より `現行 UI とテストの整合` を優先する

## Claude への補足

- `.claude/skills/pr-review/SKILL.md` は今回の作業と無関係な dirty change なので触らない
- `node_modules/` と `output/` は untracked のままでよい
- 直近の push には UI 改修一式と backend の CSV 誤検知修正が入っているので、Claude はこの続きから始めてよい

## 進捗

- [x] 調査
- [x] UI 再設計の一次実装
- [x] 資産管理 CSV 誤検知の修正
- [x] backend テスト
- [ ] csv-crud E2E の追随
- [ ] PR 作成
- [ ] レビュー対応

## レビュー指摘

- 高:
  - `Receipts` は CSV 操作だけが aside に移動しており、`ReceiptTemplate` 内の検索カードと集計ヘッダーは引き続きテーブルの上に縦積みされる。desktop でもテーブルがファーストビュー近くに来ないため、受け入れ条件の「主領域として先に視認できる」を満たしていない
- 中:
  - `AssetBalance` で `SearchCard` を aside に移したが、`SearchCard` 自体は sidebar 向けの 1 列最適化が入っていないため、1024px 以上でも狭い補助カラム内で検索 UI が詰まりやすい
