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
