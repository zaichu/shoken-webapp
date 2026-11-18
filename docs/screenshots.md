# スクリーンショットの置き場と撮り方

`screenshots` ブランチに置く。作業ブランチには画像を載せない(PR の差分に混ぜない)。

- `latest/` … 全画面の最新(390・1280・1920 × ライト・ダーク、表・一覧は PC のスクロール後つき)。
  `latest/README.md` に画面の一覧と撮った main のコミット・日時を書く。
  ファイル名は `<画面キー>-<幅>x<高さ>-<light|dark>[-scrolled].png`
  (例: `asset-stock-1280x800-dark.png`)。Muse に見せて横断レビューをもらうときの正本。
- `pr<番号>/before・after` … PR の確認用。PR が開いている間だけ置き、マージや close で消す。

データはすべて架空のモック(`frontend/scripts/ui-shots/data.mjs`)。本番の接続情報・実保有データには触らない。

## 撮り方(`frontend/scripts/ui-shots.sh`)

```bash
# PR の確認用(変更前・変更後)
frontend/scripts/ui-shots.sh <名前> before --pr <番号>
frontend/scripts/ui-shots.sh <名前> after --pr <番号>

# 全画面の最新を作り直す(latest/ を置き換える)
frontend/scripts/ui-shots.sh <名前> latest

# PR の確認用画像を消す(マージ・close 後に統合担当が実行)
frontend/scripts/ui-shots.sh --cleanup-pr <番号>
```

主なオプション: `--only home,asset-stock`(画面を絞る)、
`--viewports 390,1280`(幅を絞る。latest の既定は `390,1280,1920`)、
`--no-push`(撮るだけで push しない。`--cleanup-pr` では消す内容の表示だけ)、
`--shots-branch <名>`(push 先を変える。push 動作の検証用に一時名を指定できる)。

## PR 本文への貼り方

`after`(または `before`)を push すると `frontend/test-results/shots.md` に
前後比較の表が出る。その Markdown を PR 本文に貼る。
画像 URL は push したコミットの SHA 固定(`blob/<sha>/…?raw=true` 形)のため、
後から他の lane が push しても貼った画像は有効なまま。

1 枚だけ貼るときの形:

```text
![資産管理 PC](https://github.com/zaichu/shoken-webapp/blob/<sha>/pr<番号>/after/asset-all-1280x800-dark.png?raw=true)
```

## 統合担当の後片付け(main に frontend の変更がマージされたら)

CI では作り直さない。frontend の変更を含むマージの後片付けで統合担当が実行する
(手順は `docs/runbook.md`「PR マージ後のクリーンアップ」)。
CI に載せない理由は重さと確実さ: 全画面の撮影は trunk の release ビルドと
Playwright の全 matrix(約 150 枚)を回し、画像バイナリの push が他の lane の
`pr<番号>/` push と競合する。frontend 変更時だけ手元で1回走らせる方が軽く、
単一の書き手になるため確実。

```bash
git switch main
git pull --ff-only origin main
frontend/scripts/ui-shots.sh i<Issue番号> latest
frontend/scripts/ui-shots.sh --cleanup-pr <マージしたPR番号>
```

## Muse へのデザインレビュー依頼

`latest/` が新しくなったら Muse(OpenCode の `muse-spark`)に全画面の
デザインレビューを頼み、指摘を Issue にまとめる。
観点は `frontend-design` と `web-design-guidelines` で、画面をまたいだ不ぞろいを中心に見る
(1画面の作り込みではなく、用語・注記の型・部品・小数桁・見出しの太さと位置の揺れ)。

手順:

1. `latest/` を作り直す(上の後片付け)。
2. OpenCode で `muse-spark` モデルを使い、下のひな形で依頼する。
   読ませる画像は `latest/` の現物(ファイル名に画面・幅・テーマが入っている)と
   `latest/README.md`(画面の一覧)。
3. 返ってきた指摘を Issue にまとめる。1画面だけの好みの指摘は採用しない。
   画面をまたいだ不ぞろい(全画面に効く直し)から直す。

依頼文のひな形:

```text
screenshots ブランチの latest/ にある全画面のスクリーンショットを見て、
デザインレビューをしてください。
目録は latest/README.md です(ファイル名は <画面キー>-<幅>x<高さ>-<light|dark>[-scrolled].png)。

観点:
- frontend-design の品質(作り込みではなく画面をまたいだ一貫性)
- web-design-guidelines の検査観点(アクセシビリティ・余白・見出し・表の体裁)
- 中心は画面をまたいだ不ぞろい: 用語・注記の型・部品・小数桁・見出しの太さと位置の揺れ

出してほしいもの:
- 直すべき不ぞろいの一覧(どの画面とどの画面がどう違うか、具体的に)
- 1画面だけの好みの指摘は分けて書く
```
