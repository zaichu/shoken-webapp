# 数字と記号の書体(Issue #2042 の設計資料)

金額・率の「数字と記号の字形」だけを欧文書体に替える案を比べるための資料。日本語(円・株・口・万・千など)は Noto Sans JP のまま。
字間・単位の大きさ・配置・余白・色・項目・文字の大きさ・太さ・行の高さは変えない。候補の間の違いは `font-family` だけにする。

- 対象文字(16 字): `0-9` `,` `.` `¥` `%` `+` `−`(U+2212)
- 候補(5 書体): Inter・Roboto・Manrope・Figtree・Barlow。IBM Plex Sans は不採用(2.)
- 適用範囲: 撮影のときだけ、画面の `Amount` 部品の中に当てる。通常の表示は今のまま
- 状態: 撮影用の試作まで。書体の選定、本実装、字間や単位の大きさの調整、merge はまだ行っていない(7.)

## 1. 今回固定する値と、書体を選んだ後に決める値

### 今回固定する(比較の間は動かさない)

現行の値を写したもの。正本は `frontend/style/input.css`・`frontend/src/ui/format.rs`・`docs/ui-design.md`「用語と書式」。

| 項目 | 今回の値 | 正本 |
| --- | --- | --- |
| 日本語と単位(円・株・口・万・千)の書体 | Noto Sans JP(wght 400–700) | `docs/design/fonts.md` |
| 数字と記号 16 字の書体 | 候補ごとに替える(比較する唯一の差) | この資料 |
| 文字の大きさ | 元の `font-size` を継承する。集計帯の主 PC 32px / スマホ 30px、従 24px / 16px、本文 15px、ラベル 13px、表のセル 12px | `input.css` `@theme` |
| 太さ | 元の `font-weight` を継承する(400 か 700)。候補の書体も同じ太さで描く | `input.css` |
| ¥・%・円・株・口の大きさ | 数字と同じ(比 100%) | 現状 |
| ¥・%・円・株・口の太さ | 数字と同じ(その場の 400 か 700) | 現状 |
| 字間 | `normal`(追加 0em) | `input.css` `body` |
| 符号と数字・¥ の間 | 追加 0em(書体の字幅のまま) | 現状 |
| 行の高さ | 本文 1.5(単位なし)を継承する | `input.css` `body` |
| 数字の幅 | 等幅(`tabular-nums`。表は `tabular-nums lining-nums`) | `ui/amount.rs`・`.col-num` |
| 金額の書式 | `¥1,234`。負は `-¥1,234`、正の差分は `+¥1,234`。候補の撮影では負号の文字だけ `−`(U+2212)で描く(4.) | `shared/src/format.rs`・`ui/format.rs` |
| 小数の桁 | 1株・1口あたりは列ごとに 0〜2 桁でそろえる。利回りは小数 2 桁、損益率は小数 1 桁+符号、達成率は整数%で符号なし | `ui/format.rs`・`docs/ui-design.md`(#2007) |
| 色 | 損益の正は緑、負は赤、0 と欠測は色なし。残高・価格は正でも通常色 | `ui/amount.rs` |
| 欠測 | `—` | `ui/labels.rs` |

### 書体を選んだ後に決める(今回は適用しない)

数値は検討の起点となる例で、今回の画像には入れていない。

| 項目 | 例 | 決め方 |
| --- | --- | --- |
| ¥・% の大きさ | 数字比 100%(今のまま)/ 90% / 85% | 選んだ書体で、¥ が数字より大きく見えるかを見る |
| 円・株・口(Noto)の大きさと太さ | 100%・数字と同じ太さ(今のまま)/ 85%・太字の金額でも 400 | 欧文の数字と並んだときの高さと黒さの差を見る |
| 大きな金額の字間 | 0em(今のまま)/ -0.01em / -0.02em(30px 以上の主の値だけ) | 選んだ書体の詰まり方を見る。Stripe の販促ページは 48px で -0.02em(3.) |
| 符号と数字の間 | 0em(今のまま)/ 0.05em | − が数字に近すぎるかを見る |
| `font-size-adjust` / `size-adjust` | 指定なし(今のまま)/ `cap-height` を基準にした値 | 数字の高さを Noto にそろえるかを決める |

#### 寸法の調整(font-size-adjust 等)を選定の後に回す理由

- ユーザーの指定は、元の大きさのまま字形を比べること。大きさまで合わせると、字形の差と大きさの差が混ざり、比べられなくなる
- 技術的には今でも使える。subset でも OS/2 の `sxHeight`・`sCapHeight` などの値は残るため、小文字の字形が無くても `font-size-adjust` は効く。ただし数字の高さは x ハイトでは決まらない。合わせるなら `cap-height` を基準にする
- `0` の上端の高さの差は 3〜8% だった(Google Fonts 配信版での予備測定。2.)。書体を 1 つに絞ってから、その書体の値で決めれば足りる
- 対応状況: `font-size-adjust` は Baseline 2024(Chrome・Edge 127、Firefox 118、Safari 17)

## 2. 候補書体

### 候補の決定

| 書体 | 採否 | 理由 |
| --- | --- | --- |
| Inter | 採用 | GSUB に `tnum` がある |
| Roboto | 採用 | 既定の数字がもともと等幅で、加えて GSUB に `tnum` もある(両方を確かめた) |
| Manrope | 採用 | GSUB に `tnum` がある |
| Figtree | 採用 | GSUB に `tnum` がある |
| Barlow | 採用 | 静的ファイル `Barlow-Regular.ttf`・`Barlow-Bold.ttf` の GSUB に `tnum` がある |
| IBM Plex Sans | 不採用 | 現行の可変版 `ofl/ibmplexsans/IBMPlexSans[wdth,wght].ttf` の GSUB・GPOS に `tnum` が無い。既定の数字は等幅だが、「tnum を持つ書体」という条件を満たさない(統合担当が fontTools で確認) |
| Roboto Flex | 候補外 | Roboto と同系統のため Roboto を採った。記号の比率は違う(下の予備測定) |

### subset での検証(実ファイル)

`frontend/scripts/ui-shots/build-numerals-fonts.py` が作り、同じスクリプトで検証した結果。
検証の内容は 3 つ。
- cmap が 16 字ちょうどであること
- GSUB に `tnum` があること
- `tnum` を当てた 0〜9 の送り幅が 1 つにそろうこと(400 と 700 のそれぞれ)

| 候補 | ファイル | バイト数 | 内部名 | 軸 | cmap | tnum | 数字の送り幅(UPM 単位) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Inter | `inter.woff2` | 4,108 | Shoken Numerals A | wght 400–700、opsz 14 固定 | 16 字 | あり | 400: 1328 / 700: 1324(UPM 2048) |
| Roboto | `roboto.woff2` | 6,592 | Shoken Numerals B | wght 400–700、wdth 100 固定 | 16 字 | あり | 400: 1151 / 700: 1175(UPM 2048) |
| Manrope | `manrope.woff2` | 3,380 | Shoken Numerals C | wght 400–700 | 16 字 | あり | 400: 1240 / 700: 1240(UPM 2000) |
| Figtree | `figtree.woff2` | 3,024 | Shoken Numerals D | wght 400–700 | 16 字 | あり | 400: 623 / 700: 632(UPM 1000) |
| Barlow | `barlow-400.woff2`・`barlow-700.woff2` | 5,500 + 5,532 | Shoken Numerals E | 静的 2 本 | 16 字 | あり | 400: 527 / 700: 544(UPM 1000) |

合計 28,136 バイト(目安の 100KB 以内)。撮影では、選んだ候補の 1 書体だけを読み込む。

### Google Fonts 配信版での予備測定(第 1 段階)

Google Fonts の配信版を Chromium で 100px に描いて測った値。subset の検証とは別のもので、選定の目安として残す。
記号の列は太さ 400 の値で、同じ書体の tnum の数字 1 字の幅に対する比。行幅比は `-¥1,234,567` を tnum で組んだ幅を、Noto の同じ太さで 1.00 とした値。
¥・% の線の太さは数値では測っていない(見本の目視だけ)。

| 書体 | 数字の幅(em) 400 / 700 | 行幅比 400 / 700 | 0 の高さ(px/100) | ¥ | % | + | − | , | + の高さ※ | − の高さ※ |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Noto Sans JP(現状) | 0.555 / 0.590 | 1.00 / 1.00 | 76 / 77 | 1.00 | 1.66 | 1.00 | 1.00 | 0.50 | 0.83 | 0.54 |
| Inter | 0.648 / 0.647 | **1.17** / 1.09 | 74 | 0.85 | 1.52 | 1.00 | 1.00 | 0.42 | 0.70 | 0.45 |
| Roboto | 0.562 / 0.574 | 0.96 / 0.95 | 73 | 0.93 | 1.30 | 1.01 | 1.02 | 0.35 | 0.82 | 0.55 |
| Manrope | 0.620 / 0.620 | 1.08 / 1.03 | 74 | 0.90 | 1.46 | **0.93** | **0.87** | 0.43 | 0.81 | 0.54 |
| Figtree | 0.623 / 0.632 | 1.08 / 1.04 | 71 | 0.87 | 1.27 | 1.00 | 1.00 | 0.36 | 0.73 | 0.48 |
| Barlow | 0.527 / 0.544 | 0.99 / 0.95 | 71 | 1.31 | 1.61 | 0.95 | 0.95 | 0.50 | 0.76 | 0.49 |
| (参考)IBM Plex Sans | 0.600 / 0.600 | 1.08 / 1.02 | 72 | 1.01 | 1.54 | 1.00 | 1.00 | 0.45 | 0.76 | 0.47 |
| (参考)Roboto Flex | 0.565 / 0.565 | 1.00 / 0.95 | 73 | 0.88 | 1.60 | 0.88 | 0.88 | 0.50 | 0.78 | 0.52 |

※ 記号の上端の高さを、0 の上端の高さで割った値。

- 幅: Inter は数字が Noto より約 17% 広い。Manrope・Figtree は約 8% 広い。大きさを継承するため、390px 幅の `nowrap` の値は横に伸びる
- 符号: Manrope だけ、+ と − の幅が違う。符号の付いた値を縦に並べると、桁が少しずれる
- 負号: Noto でも、`-` を `−` にすると行が約 0.2em 広がる(534.3 → 555.1)

## 3. 実在の金融アプリの観察

観察した事実と提案は分けて書く。他社の非公開の書体名や px の値は推測で書かない。

| 対象 | 見たもの | 観察した事実 |
| --- | --- | --- |
| マネーフォワード ME | 公式ヘルプのホーム画面の画像(目視のみ)。https://support.me.moneyforward.com/hc/ja/articles/24226189541401 | 総資産 `¥1,518,776` は大きく、太字ではない。¥ は数字と同じ大きさと太さに見える。負は `¥-176,202` の並びで、符号を ¥ の後に置く(画像なので、符号の文字コードまでは分からない)。支出は赤、収入は青、収支は黒 |
| 楽天証券 iSPEED | 公式ヘルプ。https://www.rakuten-sec.co.jp/smartphone/ispeed/help/iphone/account/summary.html | 掲載の画像は 120×174px で、字形を判読できない。観察できず |
| Stripe | Dashboard の資料。https://docs.stripe.com/dashboard/basics | 金額の写った画面画像が無い。観察できず |
| Stripe(販促ページ) | https://stripe.com/jp/billing の算出スタイル(Dashboard ではない) | `font-family` の指定は `sohne-var, "SF Pro Display", sans-serif`、`ss01` 付き。大きな数値は 48px・太さ 300・字間 -0.96px(-0.02em)。表の中の率は 13px・`tabular-nums`・字間 -0.39px(-0.03em) |
| Robinhood | ログアウト状態の Web の銘柄ページ。https://robinhood.com/us/en/stocks/AAPL/ の算出スタイル | `font-family` の指定は `Capsule Sans Text`。値動きは `+$0.36 (+0.10%)`・`-$0.67 (-0.19%)` の形で、負号は U+002D(DOM で確認)。13px・太さ 700、字間 -0.1px、`font-variant-numeric: normal` |
| Apple 株価 | — | 見ていない |

観察からの提案(事実ではない):
- 通貨記号は数字と同じ大きさと太さのまま使う例がある(MF ME・Robinhood)。そのため ¥ の縮小は、選定後の任意項目にした
- 大きな数値の字間を詰めるとしても、30px 以上の主の値に限る。上限の目安は -0.02em

## 4. 試作の実装

### 変更したファイル

| ファイル | 内容 |
| --- | --- |
| `frontend/src/ui/amount.rs` | `Amount` が出す要素に、識別用の属性 `data-amount=""` を足した。見た目・文字列・色の判定は変えていない |
| `frontend/fonts/numerals/*.woff2` | 5 候補の subset(6 ファイル) |
| `frontend/fonts/numerals/OFL-<候補>.txt` | 各書体の OFL 1.1 の本文(元リポジトリの `OFL.txt`) |
| `frontend/fonts/numerals/sources.json` | 出典。元ファイルの URL(google/fonts のコミット SHA で固定)・sha256・固定した軸・出力名・ライセンス |
| `frontend/scripts/ui-shots/build-numerals-fonts.py` | subset の再現と検証。`python3 frontend/scripts/ui-shots/build-numerals-fonts.py`(fonttools・brotli が必要)。`--verify-only` で検証だけを行う |
| `frontend/scripts/ui-shots/numerals.mjs` | 候補の定義、撮影時の適用、読み込みの確認、比較用の状態の記録、字形見本 |
| `frontend/scripts/ui-shots/shoot.mjs`・`frontend/scripts/ui-shots.sh` | `--numerals <候補|all>` を足した。指定しなければ従来どおり |

`style/input.css`・API・DB・`shared`・各画面の構造は変えていない。

### 出典

元ファイルは `https://raw.githubusercontent.com/google/fonts/5bd87dfc525c8a0d1d8529d3684716e00c3fc59a/` 以下にある。

| 候補 | 元ファイル |
| --- | --- |
| Inter | `ofl/inter/Inter[opsz,wght].ttf` |
| Roboto | `ofl/roboto/Roboto[wdth,wght].ttf` |
| Manrope | `ofl/manrope/Manrope[wght].ttf` |
| Figtree | `ofl/figtree/Figtree[wght].ttf` |
| Barlow | `ofl/barlow/Barlow-Regular.ttf`・`Barlow-Bold.ttf` |

5 書体とも SIL OFL 1.1。subset は OFL の Modified Version に当たるため、内部の name(ID 1・3・4・6)を `Shoken Numerals A`〜`E` に替えた。Reserved Font Name を名乗らないためで、ID 16・17・21・22・25 も消している。著作権表示(ID 0)とライセンス(ID 13・14)は残した。元の書体名は、この資料・`sources.json`・字形見本の画像に書いてある。

### 撮影時の適用

- 読み込み: `FontFace` で候補 1 書体だけを登録する(`unicode-range: U+0025, U+002B-002C, U+002E, U+0030-0039, U+00A5, U+2212`、`font-display: block`)。配信は同じオリジンの `/fonts/numerals/` から行い、Google Fonts や CDN には通信しない
- 当て方: 構築可能なスタイルシートで `[data-amount] { font-family: "<候補>", var(--font-sans); }` を足す。配信の CSP が `style-src 'self'` なので、`<style>` 要素は使わない
- 当てない文字: 範囲外の文字(円・株・口・万・`(`・`)`・空白・`—` など)は、後ろの `var(--font-sans)` で Noto Sans JP に落ちる
- 負号: `[data-amount]` の中の文字ノードの `-` だけを `−`(U+2212)に置き換え、`MutationObserver` で描き直しにも追従する。U+002D は cmap にも `unicode-range` にも入れない。色の判定は Rust 側で元の文字列から済ませているため変わらない
- 読み込みの確認: 撮る前に次の 2 点を確かめ、満たさなければ撮影を失敗にする
  - `document.fonts` の候補の face がすべて `loaded` であること
  - CDP の `CSS.getPlatformFontsForNode` で、`Amount` の描画に候補の書体が実際に使われていること

## 5. 撮影と比較の手順

```bash
env -u NO_COLOR RUSTC_WRAPPER=sccache bash frontend/scripts/ui-shots.sh i2042-fonts after --pr 2042 \
  --only home,asset-all,asset-stock,asset-fund,receipts-dividend,receipts-stock,receipts-fund \
  --viewports 390,1280 --numerals all
```

- 画像の置き場: screenshots ブランチの `pr2042/after/`。baseline は統合担当が撮った `pr2042/before/` の 52 枚(main `5d498269`)で、変えていない
- 画像名(候補名を前に付ける)
  - 画面: `<候補>-<画面>-<幅>x<高さ>-<light|dark>[-scrolled].png`
  - 字形見本: `<候補>-specimen-<light|dark>.png`
  - 拡大: `<候補>-zoom-<home-overview|asset-stock-table>-<幅>x<高さ>-<light|dark>.png`(`deviceScaleFactor` 2 で実画面から切り出す)
- 1 候補あたりの枚数: 62 枚
  - 画面: 28 枚
  - スクロール後: 24 枚(既存どおり、スマホのスクロール後も撮る。home にはスクロールが無いため 6 画面分)
  - 字形見本: 2 枚
  - 拡大: 8 枚(home の集計帯と、資産管理・国内株式の表の先頭 6 行 × 2 幅 × 2 配色)
- baseline との比較: 同じ実行の中で、候補を当てない同じ状態を開いて `Amount` の値を記録し、候補と比べる。結果は `frontend/test-results/shots/numerals-report.json` に出す(コミットしない)
  - 変えてはいけない値(文言・色・大きさ・太さ・字間・行の高さ・`font-variant-numeric`)に差があれば、撮影を失敗にする
  - `data-testid` の付いた枠の位置と幅の差は、件数を数えるだけにする(字幅で動いてよい)

### 意図した差分と、変えてはいけないもの

| 区分 | 内容 |
| --- | --- |
| 意図した差(書体の違いで避けられない) | 数字と記号の字形、字幅による値の横幅、`nowrap` の値の伸び、表の列幅の自動調整による列位置のずれ、負号の字形(`−`) |
| 変えてはいけない(比較で確かめる) | 文言、数値、正・負・0 の色、文字の大きさ、太さ、字間、行の高さ、日本語と単位の書体、余白、項目 |

## 6. 撮影結果

- 撮影日: 2026-11-19。`env -u NO_COLOR bash frontend/scripts/ui-shots.sh i2042-fonts after --pr 2042 --only home,asset-all,asset-stock,asset-fund,receipts-dividend,receipts-stock,receipts-fund --viewports 390,1280 --numerals all`
- 画像: screenshots ブランチの `pr2042/after/` に 310 枚(5 候補 × 62 枚)。SHA `01c57ded577a4c209ca61e157aa09a7e5dc79e57`
- baseline: `pr2042/before/` の 52 枚(main `5d498269`、統合担当が撮影)。変えていない
- 比較の自動検証(`numerals-report.json`、コミットしない): 140 状態(5 候補 × 7 画面 × 2 幅 × 2 配色)で、変えてはいけない値(文言・色・文字の大きさ・太さ・字間・行の高さ・`font-variant-numeric`)の差は 0 件。位置が動いた枠は件数のみ記録(字幅の違いによる想定内の動き)
- 目視: 字形見本 10 枚と各候補の home・zoom を確認。数字と記号 16 字だけが候補書体になり、日本語(円・株・口・万・千)は Noto のまま。色・配置・余白・項目の違いは無い
- URL の一覧は Issue #2042 のコメントに書いた

## 7. まだ行っていないこと

- 書体の選定(ユーザーが画像を見て選ぶ)
- 本実装:
  - `input.css` への `@font-face` と `Amount` の書体の指定
  - 負号を文字列として U+2212 にするかどうか(色の判定と多くのテストが U+002D を前提にしている)
- 字間・単位の大きさ・`font-size-adjust` の調整(1. の後段の表)
- `Amount` を通らない数字(株数・口数、日付、`ReceiptStatCard` の素の文字列など)は Noto のまま。本採用するなら、そろえ方を決める
- merge

## 8. 残るリスク

- 混在: `Amount` を通らない数字は Noto のまま。比較画像には、2 つの書体の数字が同じ画面に並ぶ
- 幅: Inter は数字が約 17% 広い。390px で値が伸びるため、選ぶなら大きさや折り返しの調整が要るかもしれない
- 負号: 撮影では DOM の文字ノードを置き換えているだけ。本実装で同じ見た目にするには、文字列か書体側で別の手当てが要る
- ライセンス: subset は改名して OFL 本文を同梱した。本番で配信する前に、統合担当がもう一度確認する
