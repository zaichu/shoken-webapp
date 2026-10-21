use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::settings::{PromptColumn, ReviewPromptPref};
use crate::ui::format::format_number_value;
use crate::ui::labels::{SECURITY_CODE, SECURITY_NAME};
use rust_decimal::prelude::ToPrimitive;

const PROMPT_BODY: &str = "あなたは日本株の公開情報を整理する調査サポーターです。
以下の保有銘柄データをもとに、保有株の構成レビュー（ポートフォリオ総評の下書き）を作成してください。

【重要】
- このプロンプトによる回答は金融商品取引業者・投資顧問としての助言ではなく、売買指示・目標株価・断定的推奨を行わないでください
- 最新情報を確認できない場合は「確認不能」と明記し、提供された保有構成から分かる範囲だけで整理してください
- 最新情報の取得・判断はユーザー自身が行う前提としてください
- このアプリから渡す保有データだけで評価損益や時価評価を判断しないでください
- 確認できない情報は推測せず「不明」または「要確認」と明記してください
- 銘柄コードが曖昧な場合は、市場・上場銘柄の確認から始めてください
- 投資目的、投資期間、リスク許容度、流動性需要、税務状況、他の資産状況が不足しているため、個別判断が必要な場合は追加質問として列挙してください

【作成してほしい内容】
1. ポートフォリオ総評（保有構成の概要・特徴）
2. 構成上のリスク視点（集中リスク・業種偏り・銘柄偏りの読み取り）
3. 各銘柄について確認を推奨する公開情報の観点（決算・IR・業績・指標など）
4. 追加で確認すべきニュース・開示・リスク要因
5. 個別判断に必要な追加質問リスト

【保有銘柄データ】";

fn column_cell(row: &AssetBalanceRow, column: PromptColumn) -> String {
    match column {
        PromptColumn::Shares => format_number_value(row.shares().to_f64().unwrap_or(0.0)),
        PromptColumn::AveragePrice => format!(
            "¥{}",
            format_number_value(row.average_purchase_price().to_f64().unwrap_or(0.0))
        ),
        PromptColumn::TotalPurchase => format!(
            "¥{}",
            format_number_value(row.total_purchase_amount().to_f64().unwrap_or(0.0))
        ),
        PromptColumn::CurrentPrice => format!(
            "¥{}",
            format_number_value(row.current_price().to_f64().unwrap_or(0.0))
        ),
        PromptColumn::Unknown => String::new(),
    }
}

pub(crate) fn generate_asset_review_prompt(
    rows: &[AssetBalanceRow],
    pref: &ReviewPromptPref,
) -> String {
    let preamble = pref.preamble.trim();
    let body = if preamble.is_empty() {
        PROMPT_BODY.to_string()
    } else {
        format!("{preamble}\n\n{PROMPT_BODY}")
    };
    let header = [SECURITY_CODE, SECURITY_NAME]
        .into_iter()
        .chain(pref.columns.iter().map(|column| column.label()))
        .collect::<Vec<_>>()
        .join(" | ");
    let table_rows = rows
        .iter()
        .map(|row| {
            let mut cells = vec![
                row.security_code().to_string(),
                row.security_name().to_string(),
            ];
            cells.extend(pref.columns.iter().map(|column| column_cell(row, *column)));
            cells.join(" | ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{body}\n{header}\n{table_rows}")
}

#[cfg(test)]
mod tests;
