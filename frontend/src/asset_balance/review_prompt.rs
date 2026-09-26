use crate::asset_balance_domain::format_number_value;
use crate::dto::AssetBalance;
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
                row.security_code.clone(),
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
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn make_asset(
        security_code: &str,
        security_name: &str,
        shares: i64,
        average_purchase_price: i64,
    ) -> AssetBalance {
        AssetBalance {
            id: format!("id-{security_code}"),
            security_code: security_code.to_string(),
            security_name: security_name.to_string(),
            shares: Decimal::new(shares, 0),
            executing_shares: Decimal::ZERO,
            average_purchase_price: Decimal::new(average_purchase_price, 0),
            total_purchase_amount: Decimal::ZERO,
            current_price: Decimal::ZERO,
            daily_change: Decimal::ZERO,
            market_value: Decimal::ZERO,
            profit_loss_rate: Decimal::ZERO,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn toyota() -> AssetBalance {
        make_asset("7203", "トヨタ自動車", 100, 2500)
    }

    fn sony() -> AssetBalance {
        make_asset("6758", "ソニーグループ", 50, 12000)
    }

    #[test]
    fn generate_asset_review_prompt_matches_fixture_exactly() {
        let prompt = generate_asset_review_prompt(&[toyota(), sony()]);
        let expected = "あなたは日本株の公開情報を整理する調査サポーターです。
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

【保有銘柄データ】
銘柄コード | 銘柄名 | 保有株数 | 平均取得単価
7203 | トヨタ自動車 | 100 | ¥2,500
6758 | ソニーグループ | 50 | ¥12,000";
        assert_eq!(prompt, expected);
    }

    #[test]
    fn generate_asset_review_prompt_includes_all_codes_and_names() {
        let prompt = generate_asset_review_prompt(&[toyota(), sony()]);
        assert!(prompt.contains("7203"));
        assert!(prompt.contains("トヨタ自動車"));
        assert!(prompt.contains("6758"));
        assert!(prompt.contains("ソニーグループ"));
    }

    #[test]
    fn generate_asset_review_prompt_keeps_input_order() {
        let prompt = generate_asset_review_prompt(&[sony(), toyota()]);
        let sony_pos = prompt.find("6758").unwrap();
        let toyota_pos = prompt.find("7203").unwrap();
        assert!(sony_pos < toyota_pos);
    }

    #[test]
    fn generate_asset_review_prompt_formats_fractional_numbers() {
        let mut asset = toyota();
        asset.shares = Decimal::new(1005, 1);
        asset.average_purchase_price = Decimal::new(1234567, 2);
        let prompt = generate_asset_review_prompt(&[asset]);
        assert!(prompt.contains("7203 | トヨタ自動車 | 100.5 | ¥12,345.67"));
    }

    #[test]
    fn generate_asset_review_prompt_empty_rows_does_not_panic() {
        let prompt = generate_asset_review_prompt(&[]);
        assert!(
            prompt.ends_with("【保有銘柄データ】\n銘柄コード | 銘柄名 | 保有株数 | 平均取得単価\n")
        );
    }

    #[test]
    fn generate_asset_review_prompt_excludes_aggregate_and_personal_fields() {
        let prompt = generate_asset_review_prompt(&[toyota(), sony()]);
        assert!(!prompt.contains("集計:"));
        assert!(!prompt.contains("保有銘柄数"));
        assert!(!prompt.contains('@'));
        assert!(!prompt.contains("user_id"));
    }

    #[test]
    fn generate_asset_review_prompt_header_excludes_valuation_columns() {
        let prompt = generate_asset_review_prompt(&[toyota()]);
        let header_line = prompt
            .lines()
            .find(|line| line.starts_with("銘柄コード"))
            .unwrap();
        for excluded in [
            "現在値",
            "取得額",
            "評価額",
            "評価損益額",
            "損益率",
            "推定1株配当",
        ] {
            assert!(!header_line.contains(excluded), "{excluded}");
        }
    }

    #[test]
    fn generate_asset_review_prompt_states_review_purpose_and_limits() {
        let prompt = generate_asset_review_prompt(&[toyota()]);
        assert!(prompt.contains("調査サポーター"));
        assert!(!prompt.contains("証券アナリストの観点"));
        assert!(prompt.contains("金融商品取引業者"));
        assert!(prompt.contains("売買指示"));
        assert!(prompt.contains("ポートフォリオ総評"));
        assert!(prompt.contains("確認不能"));
        assert!(prompt.contains("追加質問"));
    }
}
