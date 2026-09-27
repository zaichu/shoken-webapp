use crate::api::dto::AssetBalance;
use crate::features::asset_balance::model::format_number_value;
use rust_decimal::prelude::ToPrimitive;

const PROMPT_BODY: &str = "あなたは日本株の公開情報を整理する調査サポーターです。
以下の保有銘柄データをもとに、保有株の構成レビュー（ポートフォリオ総評の下書き）を作成してください。

【重要】
- このプロンプトによる回答は金融商品取引業者・投資顧問としての助言ではなく、売買指示・目標株価・断定的推奨を行わないでください
- 最新情報を確認できない場合は「確認不能」と明記し、提供された保有構成から分かる範囲だけで整理してください
- 最新情報の取得・判断はユーザー自身が行う前提としてください
- このアプリから渡す保有データだけで含み損益や時価評価を判断しないでください
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

const TABLE_HEADER: &str = "銘柄コード | 銘柄名 | 保有株数 | 平均取得単価";

pub(crate) fn generate_asset_review_prompt(rows: &[AssetBalance]) -> String {
    let table_rows = rows
        .iter()
        .map(|row| {
            [
                row.security_code.to_string(),
                row.security_name.clone(),
                format_number_value(row.shares.to_f64().unwrap_or(0.0)),
                format!(
                    "¥{}",
                    format_number_value(row.average_purchase_price.to_f64().unwrap_or(0.0))
                ),
            ]
            .join(" | ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{PROMPT_BODY}\n{TABLE_HEADER}\n{table_rows}")
}

#[cfg(test)]
mod tests;
