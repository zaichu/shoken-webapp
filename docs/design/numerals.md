# 数字と記号の書体(Issue #2042 の設計資料)

金額・率の「数字と記号の字形」だけ Inter に替えた記録。日本語(円・株・口・万・千など)は Noto Sans JP のまま。
字間・単位の大きさ・配置・余白・色・項目・文字の大きさ・太さ・行の高さは変えない。
ユーザーが試作の 5 候補(Inter・Roboto・Manrope・Figtree・Barlow)の画像を見て Inter を選んだ(11/19)。
試作の撮影方法(`--numerals`・`data-amount`)は本実装に持ち込んでいない。

- 対象文字(16 字): `0-9` `,` `.` `¥` `%` `+` `−`(U+2212)
- 適用範囲: `amount::Amount` 部品(`.numerals`)・集計帯(値を `amount::Amount` で出すため同じ指定になる)・表のセル(`.col-num`)の中だけ
- 状態: 本実装済み

## 1. 決めた値

現行の値を写したもの。正本は `frontend/style/input.css`・`frontend/src/ui/format.rs`・`docs/ui-design.md`「用語と書式」。

| 項目 | 決めた値 | 正本・理由 |
| --- | --- | --- |
| 日本語と単位(円・株・口・万・千)の書体 | Noto Sans JP(wght 400–700) | `docs/design/fonts.md`。変えない |
| 数字と記号 16 字の書体 | Inter の subset(`Shoken Numerals`、可変 wght 400–700、opsz 14 固定) | 試作の比較で選定。この資料 §2 |
| 数字と記号の当て方 | `@font-face` に `unicode-range`(16 字)を付け、`font-family: 'Shoken Numerals', var(--font-sans)` で出す。範囲外の文字は Noto に落ちる | `frontend/style/input.css`。Google Fonts や CDN には通信しない |
| 文字の大きさ | 元の `font-size` を継承する。集計帯の主 PC 32px / スマホ 30px、従 24px / 16px、本文 15px、ラベル 13px、表のセル 12px | `input.css` `@theme`。変えない |
| 太さ | 元の `font-weight` を継承する(400 か 700)。Inter も同じ太さで出す | `input.css`。変えない |
| ¥・%・円・株・口の大きさ | 数字と同じ(比 100%) | 現状維持。Inter の ¥ は数字より横幅が狭く(§2 の予備測定で幅比 0.85)、大きく見えないため縮小しない |
| ¥・%・円・株・口の太さ | 数字と同じ(その場の 400 か 700) | 現状維持。§3 の観察(MF ME・Robinhood)も同じ大きさ・太さで使う例がある |
| 字間 | `normal`(追加 0em) | 現状維持。変更前後の撮影で 390px の主の値の折り返し・はみ出しが無いことを確かめた(§5) |
| 符号と数字・¥ の間 | 追加 0em(書体の字幅のまま) | 現状維持 |
| 行の高さ | 本文 1.5(単位なし)を継承する | `input.css` `body`。変えない |
| 数字の幅 | 等幅(`tabular-nums`。表は `tabular-nums lining-nums`) | `ui/amount.rs`・`.col-num`。subset の `tnum` で 400・700 とも 1 つにそろうことを検証済み(§4) |
| マイナスの字形 | `−`(U+2212)。`amount::Amount` の表示層で符号位置の `-` だけ置き換える。色の判定・CSV の書き出しなどデータ側の文字は `-`(U+002D)のまま | `amount::display_text`。日付の中の `-` は触らない |
| 金額の書式 | `¥1,234`。負は表示 `−¥1,234`、正の差分は `+¥1,234` | `shared/src/format.rs`・`ui/format.rs`。データ側は `-` のまま |
| 小数の桁 | 1株・1口あたりは列ごとに 0〜2 桁でそろえる。利回りは小数 2 桁、損益率は小数 1 桁+符号、達成率は整数%で符号なし | `ui/format.rs`・`docs/ui-design.md`(#2007)。変えない |
| 色 | 損益の正は緑、負は赤、0 と欠測は色なし。残高・価格は正でも通常色 | `ui/amount.rs`。変えない |
| 欠測 | `—` | `ui/labels.rs`。変えない |
| `font-size-adjust` / `size-adjust` | 指定なし | 数字の高さを Noto にそろえる必要が無いことを変更前後の撮影で確かめた(§5)。要らなければ足さない |

### 寸法の調整を入れなかった理由

- 試作では大きさまで合わせると字形の差と混ざるため、元の大きさのまま比べた(下の §2「寸法の調整を後に回す理由」も参照)
- 本実装の変更前後の撮影で、¥ が大きく見える・大きな金額がはみ出す・数字の高さがそろわない、のいずれも起きなかったため、¥・% の縮小・字詰め・`font-size-adjust` は入れていない
- 要るとなったら、この表に比・em 値を書いてから入れる

## 2. 候補書体(試作の記録)

ユーザーが画像を見て Inter を選んだ。選定の根拠として残す。

### 候補の決定

| 書体 | 採否 | 理由 |
| --- | --- | --- |
| Inter | 採用 | GSUB に `tnum` がある |
| Roboto | 不採用 | 既定の数字がもともと等幅で、加えて GSUB に `tnum` もある(両方を確かめた) |
| Manrope | 不採用 | GSUB に `tnum` がある。ただし + と − の幅が違い、符号付きの値の桁がずれる |
| Figtree | 不採用 | GSUB に `tnum` がある |
| Barlow | 不採用 | 静的ファイル `Barlow-Regular.ttf`・`Barlow-Bold.ttf` の GSUB に `tnum` がある |
| IBM Plex Sans | 候補外 | 現行の可変版の GSUB・GPOS に `tnum` が無い。「tnum を持つ書体」という条件を満たさない |

### subset での検証(実ファイル)

試作の `build-numerals-fonts.py` が作り、同じスクリプトで検証した結果。
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

本実装のファイルは Inter だけを作り直したもの(`frontend/fonts/numerals/inter-400-700.woff2`、4,128 バイト、内部名 `Shoken Numerals`)。
`400: 1328 / 700: 1324` と試作と同じ値になることを `frontend/scripts/build-numerals-fonts.py` で確かめている。

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

- 幅: Inter は数字が Noto より約 17% 広い。大きさを継承するため、390px 幅の `nowrap` の値は横に伸びる。変更前後の撮影で折り返し・はみ出しが無いことを確かめた(§5)
- 符号: Manrope だけ、+ と − の幅が違う。符号の付いた値を縦に並べると、桁が少しずれる
- 負号: Noto でも、`-` を `−` にすると行が約 0.2em 広がる(534.3 → 555.1)

#### 寸法の調整(`font-size-adjust` 等)を選定の後に回した理由(試作時)

- 指定は元の大きさのまま字形を比べることだった。大きさまで合わせると、字形の差と大きさの差が混ざり、比べられなくなる
- 技術的には使える。subset でも OS/2 の `sxHeight`・`sCapHeight` などの値は残るため、小文字の字形が無くても `font-size-adjust` は効く。ただし数字の高さは x ハイトでは決まらない。合わせるなら `cap-height` を基準にする
- `0` の上端の高さの差は 3〜8% だった(Google Fonts 配信版での予備測定)

## 3. 実在の金融アプリの観察

観察した事実と提案は分けて書く。他社の非公開の書体名や px の値は推測で書かない。

| 対象 | 見たもの | 観察した事実 |
| --- | --- | --- |
| マネーフォワード ME | 公式ヘルプのホーム画面の画像(目視のみ)。https://support.me.moneyforward.com/hc/ja/articles/24226189541401 | 総資産 `¥1,518,776` は大きく、太字ではない。¥ は数字と同じ大きさと太さに見える。負は `¥-176,202` の並びで、符号を ¥ の後に置く(画像なので、符号の文字コードまでは分からない)。支出は赤、収入は青、収支は黒 |
| 楽天証券 iSPEED | 公式ヘルプ。https://www.rakuten-sec.co.jp/smartphone/ishelp/iphone/account/summary.html | 掲載の画像は 120×174px で、字形を判読できない。観察できず |
| Stripe | Dashboard の資料。https://docs.stripe.com/dashboard/basics | 金額の写った画面画像が無い。観察できず |
| Stripe(販促ページ) | https://stripe.com/jp/billing の算出スタイル(Dashboard ではない) | `font-family` の指定は `sohne-var, "SF Pro Display", sans-serif`、`ss01` 付き。大きな数値は 48px・太さ 300・字間 -0.96px(-0.02em)。表の中の率は 13px・`tabular-nums`・字間 -0.39px(-0.03em) |
| Robinhood | ログアウト状態の Web の銘柄ページ。https://robinhood.com/us/en/stocks/AAPL/ の算出スタイル | `font-family` の指定は `Capsule Sans Text`。値動きは `+$0.36 (+0.10%)`・`-$0.67 (-0.19%)` の形で、負号は U+002D(DOM で確認)。13px・太さ 700、字間 -0.1px、`font-variant-numeric: normal` |
| Apple 株価 | — | 見ていない |

観察からの提案(事実ではない):
- 通貨記号は数字と同じ大きさと太さのまま使う例がある(MF ME・Robinhood)。そのため ¥ の縮小は任意項目にした。本実装では縮小していない(§1)
- 大きな数値の字間を詰めるとしても、30px 以上の主の値に限る。上限の目安は -0.02em。本実装では詰めていない(§1)

## 4. 本実装

### 変更したファイル

| ファイル | 内容 |
| --- | --- |
| `frontend/fonts/numerals/inter-400-700.woff2` | Inter の subset(16 字、可変 wght 400–700、opsz 14 固定)。内部名は `Shoken Numerals`(Reserved Font Name を名乗らないため。ID 16・17・21・22・25 は消し、著作権表示とライセンスは残した) |
| `frontend/fonts/numerals/OFL-inter.txt` | SIL OFL 1.1 の本文(元リポジトリの `OFL.txt`。subset は Modified Version のため同梱) |
| `frontend/fonts/numerals/sources.json` | 出典。元ファイルの URL(google/fonts のコミット SHA で固定)・sha256・固定した軸・出力名・ライセンス |
| `frontend/scripts/build-numerals-fonts.py` | subset の再現と検証。`python3 frontend/scripts/build-numerals-fonts.py`(fonttools・brotli が必要)。`--verify-only` で検証だけを行う。試作のスクリプトから Inter だけを残し、内部名と出力名を本番用にした |
| `frontend/style/input.css` | `@font-face`(`Shoken Numerals`、16 字の `unicode-range`、可変 400–700)と `--font-numerals`、`amount::Amount` 用の `.numerals`、数値列の `.col-num` への書体指定 |
| `frontend/src/ui/amount.rs` | `amount::Amount` の出す要素に `numerals` クラスを足した。表示する `-` のうち符号位置のものだけ `−`(U+2212)にする(`amount::display_text`)。色の判定は元の文字列のまま、CSV などデータ側の文字は変えない |
| `frontend/src/ui/amount/tests.rs` | `amount::display_text` の単体テスト(符号の `-` は置き換え、日付の中の `-` は残す)。色の判定の期待は変えていない |
| `frontend/e2e/receipts/mobile-card.spec.ts`・`frontend/e2e/receipts/mobile.spec.ts`・`frontend/e2e/asset-balance/fund.spec.ts` | 画面に出るマイナスの期待だけ `−` に合わせた(判定の弱めはしていない) |

`style/input.css` の色・配置・余白・項目・文字の大きさのトークン、API・DB・`shared`・各画面の構造は変えていない。`features/` にクラスは足していない。
試作の撮影用の仕組み(`--numerals`・`data-amount`・候補の `@font-face` 切替え・字形見本)は持ち込んでいない。

### 出典

元ファイルは `https://raw.githubusercontent.com/google/fonts/5bd87dfc525c8a0d1d8529d3684716e00c3fc59a/ofl/inter/Inter[opsz,wght].ttf`。
ライセンスは SIL OFL 1.1。

### 読み込みと当て方

- 配信: 自前の `/fonts/numerals/` から行い、Google Fonts や CDN には通信しない(CSP `font-src 'self'` のまま)
- 当て方: `amount::Amount` の出す要素(`.numerals`)と数値列(`.col-num`)で `font-family: var(--font-numerals)` を指定する。`Shoken Numerals` の cmap と `unicode-range` は同じ 16 字のため、範囲外の文字(日本語・単位・括弧・空白・`—` など)は後ろの Noto Sans JP に落ちる
- 等幅: `tabular-nums`(表は `tabular-nums lining-nums`)を継承し、subset の `tnum` で出す
- 負号: `amount::Amount` の表示層で置き換える。DOM を書き換えるのではなく、描く文字列を作るときに替えるため、色の判定・テストの期待・CSV の文字に影響しない
- `amount::Amount` を通らない数字(株数・口数の素の文字列、日付、`ReceiptStatCard` の素の文字列など)は Noto のまま。表の中は `.col-num` で Inter にそろう

## 5. 撮影と比較の手順と結果

```bash
env -u NO_COLOR bash frontend/scripts/ui-shots.sh i2042 before --pr 2042 \
  --only home,asset-all,asset-stock,asset-fund,receipts-dividend,receipts-stock,receipts-fund,search \
  --viewports 390,1280
env -u NO_COLOR bash frontend/scripts/ui-shots.sh i2042 after --pr 2042 \
  --only home,asset-all,asset-stock,asset-fund,receipts-dividend,receipts-stock,receipts-fund,search \
  --viewports 390,1280
```

- 画像の置き場: screenshots ブランチの `pr2042/before`・`pr2042/after`
- 拡大(大きな金額と表の金額)は一時 spec で撮り、同じ場所に置いた(§6)
- 書体の当たり方は Playwright で確かめた(§6)

### 意図した差分と、変えてはいけないもの

| 区分 | 内容 |
| --- | --- |
| 意図した差(書体の違いで避けられない) | 数字と記号の字形、字幅による値の横幅、表の列幅の自動調整による列位置のずれ、負号の字形(`−`) |
| 変えてはいけない(比較で確かめる) | 文言、数値、正・負・0 の色、文字の大きさ、太さ、字間、行の高さ、日本語と単位の書体、余白、項目 |

### 結果

- 撮影日: 2026-11-19。`env -u NO_COLOR bash frontend/scripts/ui-shots.sh i2042 before|after --pr 2042 --only home,asset-all,asset-stock,asset-fund,receipts-dividend,receipts-stock,receipts-fund,search --viewports 390,1280`
- before: screenshots ブランチの `pr2042/before`(SHA `011c8de659909089794ba43e4376704950a92328`、現行 main 相当のクリーンな再撮影。初回は実装の混入があったため撮り直した)
- after: screenshots ブランチの `pr2042/after`(SHA `c48b576892972fb871ef0d38512203be4f06c6c9`)
- 拡大: 集計帯と資産管理・国内株式の表(端のデータ: 9桁の金額・マイナス・小数・0・長い名前)を deviceScaleFactor 2 で切り出し、`pr2042/before|after` に `zoom-*.png` で置いた(SHA `994469011c48662d7340599b6570766475a7c736`)
- 目視(全画面 390・1280 × ライト・ダーク、PC はスクロール後も): 意図した差(数字と記号の字形・値の横幅・列位置のずれ・負号 `−`)以外に差は無い。色・配置・余白・項目・文字の大きさは変わっていない。390px の主の値に折り返し・はみ出しは無い。字形見本の拡大で、マイナスが `−` になっている
- 書体の当たり: Playwright(CDP `CSS.getPlatformFontsForNode`)で、`amount::Amount` の描画に `Shoken Numerals` が使われ、日本語ラベルには Noto Sans JP が使われること(`Shoken Numerals` は含まれない)、表示のマイナスが U+2212 のみで U+002D を含まないことを確かめた(一時 spec、撮影後に削除)

## 6. やっていないこと

- `amount::Amount` を通らない数字のそろえ方(株数・口数・日付・`ReceiptStatCard` の素の文字列などは Noto のまま。表の中は `.col-num` でそろう)
- ¥・% の縮小・大きな金額の字詰め・`font-size-adjust`(§1 の理由で入れていない)
- merge
