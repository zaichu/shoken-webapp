use serde::Deserialize;
use utoipa::ToSchema;

/// 決算サマリー取得パラメータ
#[derive(Debug, Deserialize, ToSchema)]
pub struct FinancialStatementsQuery {
    pub code: String,
    pub from: Option<String>,
    pub to: Option<String>,
}
