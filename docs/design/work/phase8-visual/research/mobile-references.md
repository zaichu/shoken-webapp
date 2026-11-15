# スマホ向け金融・資産管理アプリ ホーム画面リファレンス調査

- Issue: #1890 / Phase 8 visual (作画前・調査のみ)
- 調査日: 2026-11-16 / 対象: 公開情報のみ(公式サイト・公式ヘルプ・App Store説明とスクリーンショット・公式デザイン記事)
- 文字サイズ・行高・角丸の px 値は原則として**公開画像からの推定**。確定値は出典つきで明示する
- Dribbble/Behance は未使用(公式一次情報で足りたため)

## 一覧(16件)

| # | アプリ | 主役の数字 | 主役サイズ(推定, 390px換算) | トップの数字の数・順 | 損益の色・符号 |
|---|---|---|---|---|---|
| 1 | 楽天証券 iSPEED | 評価額合計 | 約24px | 3: 評価額合計→前日比金額→前日比率% | 赤=+/緑=−、±必須、▲▼なし |
| 2 | SBI証券 株アプリ | 評価損益合計 | 約28〜30px | 2: 評価損益合計→評価損益率 | 赤=+/緑=−、±と%、↑↓あり |
| 3 | SBI証券Plus | 資産合計 | 約28〜32px | 資産合計+損益+推移グラフ+ドーナツ | 赤=+(Plus販促画像より) |
| 4 | マネックス証券アプリ | 資産総額 | 約28〜32px | 2: 資産総額→前日比(金額+%) | 赤=+/青=−、±と%、↑↓あり |
| 5 | マネーフォワード ME | 資産合計 | 約28px | 3: 合計→3ヶ月前比→前日比 | 青=+/赤=−、丸矢印アイコン、資産トップに%なし |
| 6 | Zaim | 総残高 | 約28px | 1: 総残高のみ | プラス=緑、負債=黒、損益表示なし |
| 7 | moomoo証券 | (ホームに資産合計なし、口座タブにあり) | 一覧株価 約17〜19px | 取引画面で2: 現物口座/買付余力 | 赤=+/緑=−(中日系)、±と%必須、▲▼なし |
| 8 | Yahoo!ファイナンス JP | 資産合計 | 約22〜26px | 2〜3: 資産合計→評価損益(+内訳) | 青=+/赤=−、金額と%併記、▲▼なし |
| 9 | Yahoo!ファイナンス US | ポートフォリオ総額 | 約16〜18px | 2: 総額→損益(金額+%) | 緑=+/赤=− |
| 10 | Robinhood | Investing総額 | 約30〜34px | 2: ラベル→総額(損益行は確認不可) | 上昇=ネオングリーン、下落色は不明 |
| 11 | Revolut | 投資総額 | 約30〜34px | 3: 注意書き→総額→Accountsピル | 上昇=緑▲、下落は不明 |
| 12 | Wealthsimple | 口座総額 | 約30〜34px | 3: 口座名→総額→利息 | +は緑・$符号+(%)併記、▲▼なし |
| 13 | Trade Republic | 銘柄価格(資産総額画面は判読不可) | 約26〜30px | 3: 銘柄名→価格+▲%→期間タブ | ▲+%、+金額に+符号、モノクロ基調 |
| 14 | Apple 株価 | (資産合計なし、ウォッチリスト型) | 詳細現在値 約28〜34px / 一覧 約15〜17px | 行: 銘柄→ sparkline →現在値→変動値 | +/-符号あり、色・▲▼は公式記載なし |
| 15 | Apple Wallet / Card | カード残高 | 約28〜34px | 残高→取引履歴 | 支出は色分けカテゴリ、損益色の定義なし |
| 16 | Google Finance | 指数現在値 | 約16〜20px / 詳細 約28〜34px | 名称→現在値→(変動幅)→変動率% | (+50.46) +0.10% の併記型、色定義なし |

## 詳細メモ

### 1. 楽天証券 iSPEED

- トップ3要素: 評価額合計(白・最大)→前日比金額→前日比率%。②③同行に緑/赤で並列。「評価額合計/前日比」切替プルダウン付き
- 主役は390換算でフォント約24px前後(推定)。補助約12〜14px
- カードなし。ヘッダー帯(#383838)とリスト(#1B1B1B前後)の帯構成。明細は区切り線のみ、行高約55〜60px
- 総合サマリー・保有一覧にグラフなし(表のみ)
- 明細は2行1レコード型(左: 資産名 / 右: 時価+前日比+前日比率)
- 下部ナビ6項目: TODAY/お気に入り/検索/注文/資産・照会/メニュー
- 出典: https://ispeed.jp/smartphone/howto/account/ , https://ispeed.jp/smartphone/howto/today/ , https://itunes.apple.com/lookup?id=389339704&country=JP , https://www.rakuten-sec.co.jp/

### 2. SBI証券 株アプリ

- トップ2要素: ①評価損益合計(+532,200円・赤)→②評価損益率(+178.59%)。目アイコンで伏字化可。タブは保有証券/信用建玉/評価損益/余力
- 主役は390換算で約28〜30px(推定)。補助約12〜14px。行高約45〜50px
- カードなし、ダークネイビー(#090A1D系)フラットリスト。口座管理画面にグラフなし
- 損益は色(赤/緑)+±+%+↑↓矢印併用
- 下部ナビ6アイコン: マーケット/登録銘柄/口座管理/注文照会/銘柄検索/メニュー
- 出典: https://search.sbisec.co.jp/v2/popwin/guide/tool/kabu_app/ , https://search.sbisec.co.jp/v2/popwin/guide/tool/kabu_app/02_basic/screen_structure.html , https://search.sbisec.co.jp/v2/popwin/guide/tool/kabu_app/05_holdings/ck_holdings.html , https://apps.apple.com/jp/app/sbi証券-株-アプリ-株価-投資情報/id1240742779?uo=4

### 3. SBI証券Plus(資産管理特化・参考)

- ライトテーマ白カード型。ホーム: 資産合計+損益+資産推移エリアチャート(3ヶ月/6ヶ月/1年/全部)+ドーナツ+NISA投資額カード
- 下部ナビ4項目: ホーム/資産/見つける/お気に入り
- 出典: https://apps.apple.com/jp/app/sbi証券plus-資産管理-投資情報/id6756704350?uo=4

### 4. マネックス証券アプリ

- トップ2要素: ①資産総額(目隠し可)→②前日比(+1,000,000円(+1.25%)・赤)。ピル型ボタン群+サブタブ(推移/損益/分析/履歴)
- 主役約28〜32px、補助約14〜16px。白背景に白カード(角丸+細ボーダー)。行高約55〜60px
- グラフは積み上げ棒(国内株/外貨建/投信/債券/預り金)、画面の約1/3。棒/折れ線切替、日/月/年切替
- ブランド黄ヘッダーが画面幅いっぱい(面積大)。数値は赤=+/青=−
- 出典: https://apps.apple.com/jp/app/マネックス証券アプリ/id1317164503?uo=4 , https://faq.monex.co.jp/category/show/358?site_domain=default

### 5. マネーフォワード ME

- 資産タブは3数: 合計¥12,000,000→3ヶ月前比→前日比。主役約28px太字黒。補助ラベル約12〜13px
- 帯なし白地フラット。上部タブ(推移|内訳|ポートフォリオ|配当)+資産|負債ピル
- プラス=青丸↑、マイナス=赤丸↓。収入=青、支出=赤。±符号はマイナスのみ付与の場合あり。資産トップに%なし、カテゴリ行に構成比%
- グラフは積み上げ面グラフ(横幅いっぱい、高さ約180〜220px)+家計タブにドーナツ(直径約140〜160px)
- 明細行高約60〜68px(左: 色ドット+分類 / 右: 金額+2段目前日比)
- 出典: https://apps.apple.com/jp/app/%E5%AE%B6%E8%A8%88%E7%B0%BF%E3%83%9E%E3%83%8D%E3%83%BC%E3%83%95%E3%82%A9%E3%83%AF%E3%83%BC%E3%83%89-me-%E8%B3%87%E7%94%A7%E7%AE%A1%E7%90%86%E3%82%82%E3%81%93%E3%82%8C%E4%B8%80%E3%81%A4%E3%81%A7/id594145971

### 6. Zaim

- 残高タブは「総残高」の1数のみ(例¥3,081,988、約28px中央太字)。トップにグラフなし
- 薄グレー地に白カード2列グリッド。角丸約10〜12px、ギャップ約8〜12px、外側余白約16px
- プラス=緑、負債=黒。損益表示なし。分析タブにドーナツ(中央に支出合計)+横棒予算バー
- 下部ナビ5項目: ホーム/残高/記録/履歴/分析
- 出典: https://apps.apple.com/jp/app/%E5%AE%B6%E8%A8%88%E7%B0%BF%E3%81%8F%E3%81%B5%E3%81%86zaim-%E8%B2%AF%E9%87%91%E3%81%8C%E3%81%A7%E3%81%8D%E3%82%8B%E4%BA%BA%E6%B0%97%E5%AE%B6%E8%A8%83%E7%B0%BF-%E3%81%8B%E3%81%91%E3%81%84%E3%81%BC/id445850671?uo=4 , https://zaim.net/ , https://content.zaim.net/manuals

### 7. moomoo証券

- ホーム(お気に入り)に資産合計なし。口座タブに総資産(公式ガイド記載)。一覧の株価約17〜19px、変化率バッジ約13〜14px
- カード型でなく密な表型、左右余白約12pxで詰め気味
- 上昇=赤、下落=緑。±と%必須。行内スパークライン(幅約70〜90px、高さ約28〜36px)。明細行高約64〜76px
- 出典: https://apps.apple.com/jp/app/moomoo%E8%A8%BC%E5%88%B8-%E6%A0%AA%E5%8F%96%E5%BC%95%E3%82%A2%E3%83%97%E3%83%AA-%E6%A0%AA%E4%BE%A1-%E6%8A%95%E8%B3%87%E6%83%85%E5%A0%B1-%E9%8A%98%E6%9F%84/id6752380139?uo=4 , https://www.moomoo.com/jp/download

### 8. Yahoo!ファイナンス JP

- ホーム例は2数: 資産合計(約22〜26px)→評価損益(約18〜20px)。気になる銘柄例は3数: 時価→損益(+%)→前日比(+%)
- 青ヘッダベタ(面積大)+白カード(角丸約8〜12px)。上昇・プラス=青、下落・マイナス=赤。金額と%併記、±必須
- 保有資産チャートは面/折れ線、高さ約160〜200px。明細は2段型で行高約76〜92px
- 下部ナビ5項目: ホーム/ポートフォリオ/ニュース/掲示板/メニュー
- 出典: https://apps.apple.com/jp/app/yahoo-%E3%83%95%E3%82%A1%E3%82%A4%E3%83%8A%E3%83%B3%E3%82%B9/id843247815?uo=4 , https://finance.yahoo.co.jp/ , https://finance.yahoo.co.jp/feature/promotion/portfolio/index.html

### 9. Yahoo!ファイナンス US

- My Portfolioは2数: 総額→損益(金額+%)。総額約16〜18px(ダークカード内のため小さめ)
- 上昇=緑、下落=赤(JPと逆)。「-53.62(-0.08%)」の金額(%)併記型
- 行内スパークライン小+詳細でフルスクリーン級チャート
- 出典: https://apps.apple.com/us/app/yahoo-finance-stock-news/id328412701 , https://apps.apple.com/us/app/yahoo-finance-stocks-markets/id328412701?uo=4 , https://finance.yahoo.com/portfolios/

### 10. Robinhood

- トップ2要素: ラベル「Investing」→総額(約30〜34px太字・左寄せ)。損益行は確認不可
- ホームはカードなしフルブリード黒背景。期間切替1D/1W/1M/3M/YTD/1Y/ALL(選択中のみライム塗りピル、約11〜12px)
- ホームグラフは単線ライン・エリア塗りなし、高さ約180〜220px。詳細はローソク足+Volume
- 銘柄詳細は「HMNI $121.10 (1.43%)」のティッカー+$+(%)併記、▲▼なし
- 出典: https://apps.apple.com/us/app/robinhood-buy-stocks-crypto/id938003185 , https://robinhood.com/

### 11. Revolut

- 投資タブ3要素: 注意書き「Capital at risk」(約10〜11px)→総額(約30〜34px白太字)→Accountsピル
- 濃ダークグレーカード(角丸約16〜20px)。クイックアクションは円形アイコン4つ横並び
- 損益は金額+「▲2.52%」で方向を▲・%表示。上昇の▲は緑系
- 明細は左丸ロゴ+2行 / 右金額+損益率、行高約60〜68px
- 出典: https://www.revolut.com/ , https://help.revolut.com/help/wealth/getting-started-investing-basics/

### 12. Wealthsimple

- 3要素: 口座名→総額(約30〜34px)→利息。白/クリーム背景に薄グレーカード(角丸約16〜24px)
- 損益は「+$62.34 (+6.34%)」のように±+$と%を()併記。▲▼なし。利息は縦棒ミニバー(高さ約60〜80px)
- 明細は左アイコン+銘柄/数量 / 右金額/損益、行高約56〜64px。下部ナビは4アイコン(ラベルなし)
- 出典: https://apps.apple.com/us/app/wealthsimple/id1403491709 , https://www.wealthsimple.com/

### 13. Trade Republic

- 銘柄画面3要素: 銘柄名→価格+「▲10,55 %」横並び→期間タブ(1T/1W/1M/1J/Max)。価格約26〜30px
- 白黒モノクロ基調、ブランド色ほぼなし。+金額は緑、「+6,03 €」のように+符号+€
- グラフは単線細ライン・エリア塗りなし、高さ約180〜220px
- 出典: https://traderepublic.com/ , https://apps.apple.com/de/app/trade-republic-broker-bank/id1410703839?uo=4

### 14. Apple 純正「株価」

- 資産合計なし、ウォッチリスト型。行: ティッカー/銘柄名→sparkline→現在値→変動値ボタン(Price Change/Percentage/Market Capを行タップで一括切替)
- 一覧現在値約15〜17px、詳細現在値約28〜34px。補助(変動値)約13〜15px、銘柄名約12〜13px
- カード型でなく区切り線のみのフラットリスト。行高約56〜64px
- 詳細は時系列折れ線(1D〜ALL、ホバーで日付・価格)、上半分約40〜50%(高さ約180〜240px)
- 出典: https://support.apple.com/guide/stocks/change-the-ticker-symbol-display-st7e0b19a267/mac , https://support.apple.com/guide/stocks/change-the-chart-display-stc4cfc704df/mac , https://support.apple.com/guide/stocks/view-stocks-st96acfa8842/mac

### 15. Apple Wallet / Card

- カード選択→残高(約28〜34px)→取引履歴の順。カードスタック自体がナビ(タブバーなし)
- 取引行は左: 店名+日付・場所 / 右: 金額。行高約60〜72px
- 支出は色分けカテゴリ、週/月/年サマリ。残高の損益色定義なし
- 出典: https://www.apple.com/wallet/ , https://www.apple.com/apple-card/ , https://www.apple.com/apple-card/features/ , https://support.apple.com/en-us/111117

### 16. Google Finance

- ホーム先頭は指数リスト: 名称→現在値(約16〜20px)→(変動幅)→変動率%。例 "Dow Jones 52,347.55 (+50.46) +0.10%"
- ポートフォリオは保有テーブル(Symbols/Price/Quantity/Daily change/Total gain/loss)。「Missing info→Add info」表示あり
- カード分割が弱いフラット構成。グラフは時系列折れ線+比較機能、高さ約200〜260px
- 出典: https://www.google.com/finance/ , https://support.google.com/websearch/answer/7579076?hl=en , https://support.google.com/websearch/answer/18884462?hl=en

## デザイン指針(公式値のみ)

### Apple HIG(確定値)

- iOS デフォルト本文17pt / 最小11pt。軽いウェイト(Ultralight/Thin/Light)は避ける
- Dynamic Type(Large): Large Title 34pt / Title1 28pt / Title2 22pt / Title3 20pt / Headline・Body 17pt / Callout 16pt / Subhead 15pt / Footnote 13pt / Caption1 12pt / Caption2 11pt
- ボタンのヒット領域は少なくとも44x44pt
- 金額専用の指定なし。残高の主表示の候補は Large Title(34)またはTitle1(28)。固定サイズでなくテキストスタイルを使うこと
- 出典: https://developer.apple.com/design/human-interface-guidelines/typography , https://developer.apple.com/design/human-interface-guidelines/buttons , https://developer.apple.com/design/human-interface-guidelines/layout

### Material Design 3(確定値・公式CSS変数より)

- Display S 36/44、Headline M 28/36、Headline S 24/32、Title L 22/30、Title M 16/24、Title S 14/20、Body L 16/24、Body M 14/20、Body S 12/16、Label M 12/16、Label S 11/16(サイズ/行間px)
- 金額の主表示の候補は Display S(36)〜Headline M(28)。行内は Title L(22)〜Title M(16)
- リスト・カード仕様と48dpタップ領域は取得ページがJS必須で本文確認できず、不明とする
- 出典: https://m3.material.io/styles/typography/type-scale-tokens

### デジタル庁デザインシステム(確定値)

- 文字サイズ: Display 48〜64 / Standard 16〜45(本文・UIは16px以上が基準) / 14は例外的(フッター等)、14px未満は原則不可
- 行間: 単行100%、高密度120〜130%、大見出し140%、本文は最低150%
- テキストブロック幅は半角80文字(全角40文字)程度
- 等幅あり(Mono-17/16/14)。数字・金額の指定なし
- 余白は8px基準の倍率スケール。角丸は0/8/16/32の5段階。成功=緑・エラー=赤(損益への割当て指定なし)
- 出典: https://design.digital.go.jp/dads/foundations/typography/ , https://design.digital.go.jp/dads/foundations/typography/text-style/ , https://design.digital.go.jp/dads/foundations/typography/accessibility/ , https://design.digital.go.jp/dads/foundations/color/ , https://design.digital.go.jp/dads/foundations/spacing/ , https://design.digital.go.jp/dads/foundations/corner-shapes/

### 損益色の地域慣行(要実装前検証・一次資料未確認)

- 日本(国内証券): 上昇=赤、下落=緑 (iSPEED/SBI/moomooで実画面確認)
- 日本(家計・ポータル系): 上昇=青、下落=赤 (MF ME/Yahoo JPで実画面確認)
- 米国・欧州: 上昇=緑、下落=赤 (Yahoo US/Wealthsimpleで実画面確認)
- 中国系: 上昇=赤、下落=緑 (moomooで実画面確認)
- 色のみで損益を伝えない(デジタル庁: コントラスト比テキスト4.5:1以上、▲▼・±の記号併記)
