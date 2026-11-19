# フォント(Noto Sans JP の自前配信)

外部の Google Fonts は読まない(CSP を広げず、閲覧のたびに第三者へ要求を出さない)。
`frontend/fonts/NotoSansJP-400-700.woff2` を `index.html` の
`copy-dir` で配信し、`style/input.css` の `@font-face`
(`font-weight: 400 700`、`font-display: swap`)で読む。
読み込めないときは `--font-sans` のフォールバック
(Hiragino・Yu Gothic・Meiryo)で出す。数字は Noto 自体が等幅のため
`tabular-nums` は主にフォールバック用の指定になる。

## ファイル

- `frontend/fonts/NotoSansJP-400-700.woff2` (980,600 バイト、可変 wght 400-700)
- `frontend/fonts/OFL.txt` (SIL OFL 1.1 の文面。subset は Modified Version のため同梱)

## subset の作り方

元: https://github.com/google/fonts/raw/main/ofl/notosansjp/NotoSansJP%5Bwght%5D.ttf
道具: `fonttools` の `varLib.instancer` と `pyftsubset`

1. 軸を 400-700 に絞る(本文 400・見出し等 700 の2段だけ使うため)
   `fonttools varLib.instancer NotoSansJP[wght].ttf wght=400:700 -o NotoSansJP-400-700.ttf`
2. 文字を絞って woff2 化(約4,700字。常用漢字+旧字体・人名用漢字・
   アプリとモックの使用文字・姓名の表外変体(濵邉鰺峯嶋嵜舘壷𠮷)・
   かな・記号・CJK互換(﨑など)・㈱㈲㈳㍿・半角カナ。囲み数字・囲み英数は対象外)
   `pyftsubset NotoSansJP-400-700.ttf --flavor=woff2
   --text-file=chars.txt --layout-features='kern,palt,vert,halt,ccmp,locl,mark,vhal,vkrn,vpal'`

常用・人名用の典拠は Unihan の `kJoyoKanji`/`kJinmeiyoKanji` と
`常用漢字一覧`・`人名用漢字一覧`(旧字体を含む)。

## 数字と記号の Inter subset(#2042)

金額・率の数字と記号(16 字: `0-9` `,` `.` `¥` `%` `+` `−`(U+2212))だけ Inter を当てる。
日本語・単位(円・株・口・万・千)・括弧など範囲外の文字は後ろの Noto に落ちる。
決めた値(等幅・¥・% の比・字間・マイナスの字形)は `docs/design/numerals.md`。

### ファイル

- `frontend/fonts/numerals/inter-400-700.woff2` (4,128 バイト、可変 wght 400–700、opsz 14 固定、cmap は 16 字ちょうど)
- `frontend/fonts/numerals/OFL-inter.txt` (SIL OFL 1.1 の文面。subset は Modified Version のため同梱)
- `frontend/fonts/numerals/sources.json` (出典: google/fonts のコミット SHA で固定した元ファイルの URL・sha256・軸)

### 当て方

`style/input.css` の `@font-face` に `unicode-range`(16 字)を付け、
`Amount` の出す要素(`.numerals`)と数値列(`.col-num`)で
`font-family: var(--font-numerals)`(`'Shoken Numerals'` の後に `--font-sans`)を指定する。
等幅は `tabular-nums`(表は `tabular-nums lining-nums`)と subset の `tnum` で出す。
内部名は Reserved Font Name を避けて `Shoken Numerals` に替えている。

### subset の作り方

`python3 frontend/scripts/build-numerals-fonts.py`(fonttools・brotli が必要)。
`--verify-only` で検証だけを行う。検証の内容は cmap が 16 字ちょうど・
GSUB に `tnum` がある・`tnum` 適用後の 0–9 の送り幅が 1 つにそろうこと(400 と 700)。
