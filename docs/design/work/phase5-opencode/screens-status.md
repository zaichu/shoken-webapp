# Phase 5 状況表(OpenCode 担当分)

- 採用方針: `docs/design/work/phase3-decision.md`(C を骨格・B のスマホ・A の PC 詳細パネル。縦ナビなし、月次は棒、円は A 方式、全桁表示)。
- 部品・色の根拠: `docs/design/work/phase4-codex/`(tokens.css・foundations.md・components.md)、`docs/design/work/phase4-opencode/`(finance-data.md・layout.md。PR #1899)。
- 文言の根拠: `frontend/src/ui/labels.rs` の定数のみ使用。架空データのみ(コードは 9900 番台、社名は架空)。
- 共通値(2 担当で同じ値。合計と内訳は検算済み。検算表は `docs/design/work/phase5-opencode/data-check.md`。生成は `docs/design/work/phase5-opencode/data/build-opencode.mjs` が Codex 側と同一の `docs/design/work/phase5-codex/data/fixtures.json` から行う): 評価額 ¥28,593,148、取得総額 ¥27,040,660、評価損益 +¥1,552,488(28,593,148 − 27,040,660)、年の配当 税引後 ¥3,163,438(内訳: 税引前 ¥3,969,907 − 税額 ¥806,469。12 か月分の合計、明細 18 行)。取引明細: 国内株式売買 3 行(実現損益 税引後 +¥99,017)、投資信託解約 2 行(税引後 +¥147,282)。CSV 保存件数: 資産管理 国内株式 2・投資信託 1、取引明細 配当金 18・国内株式 3・投資信託 2(いずれも行の数から数える)。ホームの最近の配当入金は明細行の新しい順の上位 2 行(12/24 ¥10,957・12/10 ¥23,905)。
- 見方: 1 行=画面の 1 状態。PC=1280 幅、スマホ=390 幅(切替 1024px)。ダークは各画面の代表 1 枚(PC)。

## 画面・状態一覧(担当分)

| 画面・状態 | デザイン | PC | スマホ | 解決した Phase 2 の課題 | 根拠の画像 |
|---|---|---|---|---|---|
| 取引明細・配当金タブ(一覧・年/月グループ・集計帯) | 済み | ○ | ○ | RE-3(小計行「2026年12月 · 2件の合計」など全 12 か月)、Codex-1(取込日は年付き)、Codex-4(詳細の項目順=表の列順)。18 行は fixtures から生成 | shots/receipts-dividend-pc-light.png・shots/receipts-dividend-sp-light.png |
| 取引明細・配当金タブ(条件パネル) | 済み | ○(ポップオーバー) | ○(シート) | Codex-1(期間・口座・商品。期間は下書き確定) | shots/receipts-dividend-cond-pc-light.png |
| 取引明細・配当金タブ(行詳細) | 済み | ○(詳細パネル) | ○(シート: 行ごと #rcpt-d1〜d18。スマホ全18行・PC全18行の税引後リンクが対応する行のシートを開く) | RE-1(部分対応: スマホ行 2 段目に税引前の目安を追加。完全な内訳は詳細に集約) | shots/receipts-dividend-dialog-pc-dark.png(ダーク代表) |
| 取引明細・国内株式タブ | 済み | ○ | ○ | RE-3(月グループ小計)、X-4(右揃え・符号+色・欠測 —)。見出し件数はタブ選択で 3件に切替。3 行は保有と整合する銘柄(9901/9902)で fixtures から生成 | shots/receipts-stock-pc-light.png・shots/receipts-stock-sp-light.png |
| 取引明細・投資信託タブ | 済み | ○ | ○ | RE-3、X-4。見出し件数はタブ選択で 2件に切替。2 行は保有ファンドと同一銘柄・取得単価で fixtures から生成 | shots/receipts-fund-pc-light.png・shots/receipts-fund-sp-light.png |
| CSV 管理(5 種類の表・取得カード・取込結果・部分失敗) | 済み | ○ | ○(2 段) | CSV-1(事実のみの状態文+時点を別項目)、Codex-6(状態名と時点を分離)。件数は行の数から生成(国内株式 2・投資信託 1・配当金 18・国内株式 3・投資信託 2) | shots/csv-pc-light.png・shots/csv-sp-light.png |
| CSV 管理(行メニュー・削除確認) | 済み | ○ | ○ | CSV-2(行メニュー・削除確認・取込結果を掲載。削除確認は対象ごと: 配当 18件 #csv-delete-div・国内株式 3件 #csv-delete・投信 2件 #csv-delete-fund・資産管理 2件/1件/3件) | shots/csv-delete-pc-dark.png(ダーク代表) |
| CSV 管理(急減の確認) | 済み | ○ | ○(共通ダイアログ) | CSV-2(配当金 18件 → 4件の例) | shots/csv-plunge-pc-light.png |
| 設定(7 セクション: アカウント・表示・データ・セキュリティ・登録端末・ログイン履歴・危険操作) | 済み | ○(左目次) | ○(簡易目次+縦積み) | ST-1(スマホ簡易目次)、ST-2(ブラウザ/OS を短い表示名に正規化)、ST-3(危険操作への到達可)。データ項目の件数は fixtures から生成 | shots/settings-pc-light.png・shots/settings-sp-light.png・shots/settings-pc-dark.png(ダーク代表) |
| 管理者ログイン履歴(表・検索・条件パネル・並べ替え) | 済み | ○ | ○(カード) | AD-2(部分対応: メール 1 行+日時・場所の 2 段に短縮)、AD-4(アカウントメニューに管理者導線) | shots/admin-logins-pc-light.png・shots/admin-logins-sp-light.png・shots/admin-logins-pc-dark.png(ダーク代表) |
| 管理者ログイン履歴(検索 0 件) | 済み | ○ | ○ | —(空状態の共通化 X-3) | shots/states-pc-light.png(状態ボードに掲載) |
| ログイン | 済み | ○ | ○ | —(中央カードのみ。課題なし) | shots/login-pc-light.png・shots/login-sp-light.png・shots/login-pc-dark.png(ダーク代表) |
| 404 | 済み | ○ | ○ | —(見出し+ホーム導線。課題なし) | shots/notfound-pc-light.png・shots/notfound-sp-light.png・shots/notfound-pc-dark.png(ダーク代表) |
| アカウントメニュー | 済み | ○ | ○ | Codex 横断(メニューは右端に開き、背後の内容を覆い尽くさない配置)。年間予想配当は fixtures から生成(¥3,969,907) | shots/account-pc-light.png・shots/account-sp-light.png |
| アカウント削除確認 | 済み | ○ | ○(共通ダイアログ) | —(タイトル・説明・取消不能の注意・実行/取消) | shots/account-delete-pc-dark.png(ダーク代表) |
| 状態ボード(読込スケルトン・空・エラー再試行・部分失敗・未ログイン案内・権限なし) | 済み | ○ | ○ | X-3(空の説明は 1〜2 行)・X-6(共通パターン)・端数データ(桁の多い値・長い名前・複数口座・欠測・更新待ち) | shots/states-pc-light.png・shots/states-sp-light.png・shots/states-pc-dark.png(ダーク代表) |

## 集計

- 担当画面: 8 種(取引明細・CSV 管理・設定・管理者ログイン履歴・ログイン・404・アカウントメニュー+削除確認・状態ボード)。HTML 9 枚(取引明細は 3 タブを 1 枚に収容)。
- 完成: 8/8。未作成: 0(担当分)。
- PNG: 30 枚、合計約 1.6MB(1 枚 120KB 以下)。PC ライト 12・スマホ ライト 10・PC ダーク 8(ダイアログ系の代表を含む)。Phase 7 で全30枚を撮り直し(高-4 の削除確認6件+アカウント削除1件にキーボード設計注記を追加)。

## 担当外(未作成の理由)

- ホーム・資産管理(3 タブ)・構成比・保有/投信詳細・銘柄検索: 別レーン(Codex)の担当のため本 PR では未作成。共通値(評価額・配当合計)は本表の冒頭に記載し、別レーンと突き合わせる。
- AD-1 の「メール省略+コピー」: 全文表示+折り返しを採用した(省略すると管理者の識別に支障があるため)。コピー操作は実装側の判断に委ねる。
- AD-3(a11y spec への admin 追加): 実装側の作業のため対象外。

## 未確認

- 実ブラウザ・実機での表示確認はしていない(Playwright の headless のみ。フォントは代替)。
- コントラスト比の実測はしていない(tokens.css の公称値のみ)。
- 1024〜1279px の表の収まりは 1280px の撮影のみで確認した。
- キーボード操作・フォーカス移動・スクリーンリーダー読み上げは未検証。
