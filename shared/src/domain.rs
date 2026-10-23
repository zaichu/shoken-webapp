// `typed` feature 有効時は正規型、無効時は wire 上の文字列型になる。serde の wire 形は同一で、
// utoipa の derive が型名をそのまま見るため、フィールド宣言は共通のまま型の実体だけを切り替える。
// user_id は backend 専用（wire に出ない）なので typed 時のみ存在させる。
#[cfg(feature = "typed")]
pub use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "typed")]
pub use uuid::Uuid;
#[cfg(not(feature = "typed"))]
pub type Uuid = String;
#[cfg(not(feature = "typed"))]
pub type NaiveDate = String;

#[cfg(feature = "typed")]
use crate::value::UserId;
use crate::value::{Account, RecordId, SecurityCode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 国内株式取引モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DomesticStock {
    #[cfg_attr(feature = "utoipa", schema(value_type = Uuid))]
    pub id: RecordId,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: UserId,
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    #[serde(deserialize_with = "crate::value::unchecked::security_code")]
    pub security_code: SecurityCode,
    pub security_name: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    #[serde(deserialize_with = "crate::value::unchecked::account")]
    pub account: Account,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub asked_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub proceeds: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub purchase_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub realized_profit_and_loss_after_tax: Decimal,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 国内株式取引 検索条件全体の集計
///
/// trade_date ごとに特定口座（account に「特定」を含む）と NISA 等口座を分離し、
/// 特定口座の実現損益合計がプラスの時だけ `floor(合計 * 0.20315)` を日次税額として
/// 日次集計した結果を合計する。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DomesticStockSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

/// 配当金モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Dividend {
    #[cfg_attr(feature = "utoipa", schema(value_type = Uuid))]
    pub id: RecordId,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: UserId,
    pub settlement_date: NaiveDate,
    pub product: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    #[serde(deserialize_with = "crate::value::unchecked::account")]
    pub account: Account,
    // 配当の銘柄コードは空文字を許容する既存仕様のため String のまま
    pub security_code: String,
    pub security_name: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub unit_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub dividends_before_tax: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub net_amount_received: Decimal,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 配当金 検索条件全体の集計
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DividendSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_dividends_before_tax: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_net_amount_received: Decimal,
}

/// 投資信託モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Mutualfund {
    #[cfg_attr(feature = "utoipa", schema(value_type = Uuid))]
    pub id: RecordId,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: UserId,
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    pub fund_name: String,
    pub dividends: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    #[serde(deserialize_with = "crate::value::unchecked::account")]
    pub account: Account,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub exchange_rate: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub cancellation_unit_price_yen: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub cancellation_amount_yen: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub average_acquisition_price_yen: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub realized_profit_and_loss_after_tax: Decimal,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 投資信託 検索条件全体の集計
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct MutualfundSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

/// 保有銘柄モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AssetBalance {
    #[cfg_attr(feature = "utoipa", schema(value_type = Uuid))]
    pub id: RecordId,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: UserId,
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    #[serde(deserialize_with = "crate::value::unchecked::security_code")]
    pub security_code: SecurityCode,
    pub security_name: String,
    /// 口座区分。CSV の「口座合計」区切りが無い・区切りより前の行は持たない
    #[cfg_attr(feature = "utoipa", schema(value_type = Option<String>))]
    #[serde(
        default,
        deserialize_with = "crate::value::unchecked::optional_account"
    )]
    pub account: Option<Account>,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub average_purchase_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_purchase_amount: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub current_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub daily_change: Decimal,
    /// 現在値の基準日（JST）。CSV 取込値・cron 自動取得値のどちらが入っているかの判定に使う
    pub price_as_of: Option<NaiveDate>,
    /// 前日終値。共有価格(security_prices)由来の行のみ持ち、
    /// 上場直後・取得失敗で前日終値がない銘柄は null（行の前日比は「—」・合計から除外）
    #[cfg_attr(feature = "utoipa", schema(value_type = Option<f64>))]
    pub prev_close: Option<Decimal>,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 保有銘柄 検索条件全体の集計
///
/// profit_loss_rate は銘柄ごとの比率のため単純合算せず、summary には含めない。
/// priced_* は価格あり(current_price > 0)行だけの集計で、含み損益の分子・分母に使う。
/// 価格なし行を含めないため total_* とは分母が違う。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AssetBalanceSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_market_value: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_purchase_amount: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_daily_change: Decimal,
    #[serde(default)]
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub priced_market_value: Decimal,
    #[serde(default)]
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub priced_purchase_amount: Decimal,
    /// 前日終値がない(上場直後・取得失敗)保有銘柄があるか。
    /// 前日比の合計から除いた行があるとき注記の表示に使う
    #[serde(default)]
    pub has_missing_daily_change: bool,
}

/// 投資信託の基準価額まわりの値の出どころ。
/// Csv = CSV 取込値、Api = 三菱UFJアセット公式APIの共有値。
/// 1行の基準価額・前日比・評価額・評価損益・損益率は必ずこの1つの出どころでそろう
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum FundPriceSource {
    #[default]
    Csv,
    Api,
}

/// 投資信託の保有残高（DB + APIレスポンス兼用）。
/// 基準価額まわりの値は CSV 取込値か三菱UFJ公式APIの共有値のどちらかでそろう
/// (対応表にないファンド・API が使えないときは常に CSV 値)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FundBalance {
    #[cfg_attr(feature = "utoipa", schema(value_type = Uuid))]
    pub id: RecordId,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: UserId,
    pub fund_name: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    #[serde(deserialize_with = "crate::value::unchecked::account")]
    pub account: Account,
    /// 分配金コース（再投資型・受取型など）。CSV が "-" のときは持たない
    pub dividends_course: Option<String>,
    /// 保有数量[口]
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    /// 平均取得価額[円]（1万口あたり）
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub average_purchase_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_purchase_amount: Decimal,
    /// 基準価額[円]（1万口あたり）
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub base_price: Decimal,
    /// 基準価額(前日比)[円]。CSV が "-" のときは持たない
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub base_price_day_change: Option<Decimal>,
    /// 基準価額(前月比)[円]
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub base_price_month_change: Option<Decimal>,
    /// 時価評価額[円]（CSV 値をそのまま保持）
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub market_value: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub profit_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub profit_loss_rate: Option<Decimal>,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_return: Option<Decimal>,
    /// 出どころ側の日付。Api は共有値の基準日、Csv は JST での CSV 取込日
    pub price_as_of: NaiveDate,
    /// 基準価額まわりの値の出どころ
    #[serde(default)]
    pub price_source: FundPriceSource,
    /// 前日比が「最新の基準価額の日の変動」として出せるか。false は「更新待ち」
    /// で、前日比は画面に出さない
    #[serde(default)]
    pub nav_is_latest: bool,
    /// nav_is_latest の判定に使った「公表済みのはずの最新の基準価額の日」(JST)
    #[serde(default)]
    pub expected_nav_date: Option<NaiveDate>,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 基準価額の自動更新(共有値の取得)の状態。
/// 取得できた/できなかった件数と、直近の失敗の理由で3状態を区別する
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FundNavStatus {
    /// 最終更新の時刻(JST の "YYYY-MM-DD HH:MM")。まだ取得していないときは持たない
    #[serde(default)]
    pub updated_at: Option<String>,
    /// 共有値を取得できたファンド数
    #[serde(default)]
    pub fetched_count: i64,
    /// 直近の取得で失敗したファンド数
    #[serde(default)]
    pub failed_count: i64,
    /// 保有ファンドの総数(対象・対象外の分母。口座違いの同名ファンドは1つに数える)
    #[serde(default)]
    pub fund_count: i64,
    /// 自動更新の対象となるファンド数(協会コードを持つファンド数)
    #[serde(default)]
    pub auto_update_target_count: i64,
    /// 直近の失敗の理由(URL・応答本文・内部 Details を含まない短い文)
    #[serde(default)]
    pub failure_reason: Option<String>,
}

/// 投資信託残高 検索条件全体の集計。
/// total_daily_change は保有数量で重み付けした基準価額(前日比)の円換算合計
/// （shares × 前日比 / 10000）。「更新待ち」の行と前日比を持たない行は入れない
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FundBalanceSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_market_value: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_purchase_amount: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_daily_change: Decimal,
    /// 前日比の合計に入った(最新で前日比を持つ)ファンド数。0 のとき合計は「—」にする
    #[serde(default)]
    pub latest_nav_count: i64,
    /// 「更新待ち」のファンド数
    #[serde(default)]
    pub pending_nav_count: i64,
    /// 基準価額の自動更新の状態。検索条件には依らない(共有値の取得はユーザー全体で行う)
    #[serde(default)]
    pub nav_status: FundNavStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domestic_stock_omits_user_id_and_accepts_its_absence() {
        let stock: DomesticStock = serde_json::from_str(
            r#"{
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "trade_date": "2024-01-15",
                "settlement_date": "2024-01-17",
                "security_code": "1301",
                "security_name": "極洋",
                "account": "特定",
                "shares": 100,
                "asked_price": 1500.5,
                "proceeds": 150050,
                "purchase_price": 1400.25,
                "realized_profit_and_loss": 10000.1,
                "taxes": 2031.5,
                "realized_profit_and_loss_after_tax": 7968.6,
                "created_at": "2024-01-15T01:23:45Z",
                "updated_at": "2024-01-16T01:23:45Z"
            }"#,
        )
        .expect("deserialize");

        let json = serde_json::to_value(&stock).expect("serialize");
        assert!(json.get("user_id").is_none());
        assert_eq!(json["trade_date"], "2024-01-15");
        assert_eq!(json["created_at"], "2024-01-15T01:23:45Z");
        assert_eq!(json["id"], "550e8400-e29b-41d4-a716-446655440000");
    }
}
