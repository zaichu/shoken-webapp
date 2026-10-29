# データの保護: 列の機微度と選択肢の評価

判断はしない。材料のみ示す。

## 列の機微度

高(資産・識別):
users.email・google_id、sessions.token_hash、dividends の金額列(unit_price・shares・dividends_before_tax・taxes・net_amount_received)と銘柄・口座・日付、domestic_stocks の金額列(shares・asked_price・proceeds・purchase_price・realized_profit_and_loss・taxes)と銘柄・口座・日付、mutualfunds の金額列(shares・exchange_rate・cancellation_amount_yen・average_acquisition_price_yen・realized_profit_and_loss・taxes)と fund 名・口座、asset_balances・fund_balances の数量・金額・評価額・損益、csv_import_history の(user_id・kind・last_imported_at)。

中(再取得可の市場データ):
stock の code・name・market、security_prices・fund_prices、dividend_per_share_cache、market_data_provider_rate_control。

## 列に依存する SQL

- 集計(SUM): `backend/src/services/asset_balance.rs:100-105`、`backend/src/services/dividend.rs:56-58`、`backend/src/services/domestic_stock.rs:113-126`、`backend/src/services/mutualfund.rs:54-56`、`backend/src/services/fund_balance.rs:81-106`
- 並べ替え: 明細は trade/settlement_date DESC、残高は security_code・fund_name
- 検索: `backend/src/services/domain/search_filters.rs:171-217` で銘柄名・fund 名・口座等に ILIKE(ワイルドカード逃避あり)。trgm 索引あり
- 削除・上限: `backend/src/services/domain.rs:18-22`、`backend/src/services/bulk.rs:98-145` は user_id 絞り

## 選択肢の評価

(A)マスク用ビュー: 現実性は高い。集計・検索の SQL はそのまま動く。影響は小さい(参照側の切替のみ)。作業量は S〜M。管理者には見えるまま。

(B)権限の分離: 現実性は高い。参照用ロールの追加・RLS が中心。影響は中(移行時の権限確認)。作業量は M。管理者には見えるまま。

(C)列の暗号化(鍵は Worker の秘密情報): 現実性は中。等価検索は維持できるが範囲検索・並べ替え・LIKE・SUM は DB 側で効かなくなる(取得後復号の作り替え)。影響は大きい。作業量は L。管理者には見えなくできるが鍵管理が残る。

(D)利用者本人だけが開ける暗号化(E2EE): 現実性は低い。検索・集計・並べ替えのほぼ全 SQL を作り替える。鍵紛失時の復旧なし。影響は最大。作業量は L。管理者には見えなくできる。

推奨に代わる1段落: 現スキーマは検索・集計・並べ替えを DB に頼るため、(A)か(B)から着手するのが現実的で、(C)は等価検索に絞る列から試す範囲に留め、(D)は鍵管理と機能縮小を受け入れられる場合のみ別計画にするのが順序として無理がない。
