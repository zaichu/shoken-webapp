use super::*;
use crate::api::dto::AssetBalance;
use crate::support::row::Row;
use rust_decimal::Decimal;

fn make_asset(
    security_code: &str,
    security_name: &str,
    shares: i64,
    average_purchase_price: i64,
) -> AssetBalanceRow {
    Row::Saved(AssetBalance {
        id: format!("id-{security_code}").into(),
        security_code: security_code.parse().unwrap(),
        security_name: security_name.to_string(),
        account: None,
        shares: Decimal::new(shares, 0),
        average_purchase_price: Decimal::new(average_purchase_price, 0),
        total_purchase_amount: Decimal::ZERO,
        current_price: Decimal::ZERO,
        daily_change: Decimal::ZERO,
        price_as_of: Some("2026-01-01".to_string()),
        prev_close: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    })
}

fn toyota() -> AssetBalanceRow {
    make_asset("9904", "架空自動車", 100, 2500)
}

fn syn_electronics() -> AssetBalanceRow {
    make_asset("9914", "架空エレクトロニクス", 50, 12000)
}

#[test]
fn generate_asset_review_prompt_matches_fixture_exactly() {
    let prompt =
        generate_asset_review_prompt(&[toyota(), syn_electronics()], &ReviewPromptPref::default());
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
9904 | 架空自動車 | 100 | ¥2,500
9914 | 架空エレクトロニクス | 50 | ¥12,000";
    assert_eq!(prompt, expected);
}

#[test]
fn generate_asset_review_prompt_includes_all_codes_and_names() {
    let prompt =
        generate_asset_review_prompt(&[toyota(), syn_electronics()], &ReviewPromptPref::default());
    assert!(prompt.contains("9904"));
    assert!(prompt.contains("架空自動車"));
    assert!(prompt.contains("9914"));
    assert!(prompt.contains("架空エレクトロニクス"));
}

#[test]
fn generate_asset_review_prompt_keeps_input_order() {
    let prompt =
        generate_asset_review_prompt(&[syn_electronics(), toyota()], &ReviewPromptPref::default());
    let syn_pos = prompt.find("9914").unwrap();
    let toyota_pos = prompt.find("9904").unwrap();
    assert!(syn_pos < toyota_pos);
}

#[test]
fn generate_asset_review_prompt_formats_fractional_numbers() {
    let Row::Saved(mut asset) = toyota() else {
        unreachable!()
    };
    asset.shares = Decimal::new(1005, 1);
    asset.average_purchase_price = Decimal::new(1234567, 2);
    let prompt = generate_asset_review_prompt(&[Row::Saved(asset)], &ReviewPromptPref::default());
    assert!(prompt.contains("9904 | 架空自動車 | 100.5 | ¥12,345.67"));
}

#[test]
fn generate_asset_review_prompt_empty_rows_does_not_panic() {
    let prompt = generate_asset_review_prompt(&[], &ReviewPromptPref::default());
    assert!(
        prompt.ends_with("【保有銘柄データ】\n銘柄コード | 銘柄名 | 保有株数 | 平均取得単価\n")
    );
}

#[test]
fn generate_asset_review_prompt_excludes_aggregate_and_personal_fields() {
    let prompt =
        generate_asset_review_prompt(&[toyota(), syn_electronics()], &ReviewPromptPref::default());
    assert!(!prompt.contains("集計:"));
    assert!(!prompt.contains("保有銘柄数"));
    assert!(!prompt.contains('@'));
    assert!(!prompt.contains("user_id"));
}

#[test]
fn generate_asset_review_prompt_header_excludes_valuation_columns() {
    let prompt = generate_asset_review_prompt(&[toyota()], &ReviewPromptPref::default());
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
    let prompt = generate_asset_review_prompt(&[toyota()], &ReviewPromptPref::default());
    assert!(prompt.contains("調査サポーター"));
    assert!(!prompt.contains("証券アナリストの観点"));
    assert!(prompt.contains("金融商品取引業者"));
    assert!(prompt.contains("売買指示"));
    assert!(prompt.contains("ポートフォリオ総評"));
    assert!(prompt.contains("確認不能"));
    assert!(prompt.contains("追加質問"));
}

#[test]
fn generate_asset_review_prompt_prepends_preamble() {
    let pref = ReviewPromptPref {
        preamble: "前置きのテキストです".to_string(),
        ..ReviewPromptPref::default()
    };
    let prompt = generate_asset_review_prompt(&[toyota()], &pref);
    assert!(prompt.starts_with("前置きのテキストです\n\nあなたは日本株"));
}

#[test]
fn generate_asset_review_prompt_column_selection_changes_table() {
    let mut asset = toyota();
    let Row::Saved(ref mut inner) = asset else {
        unreachable!()
    };
    inner.total_purchase_amount = Decimal::new(250000, 0);
    inner.current_price = Decimal::new(3000, 0);

    let pref = ReviewPromptPref {
        preamble: String::new(),
        columns: vec![
            crate::settings::PromptColumn::Shares,
            crate::settings::PromptColumn::CurrentPrice,
        ],
    };
    let prompt = generate_asset_review_prompt(&[asset], &pref);
    assert!(prompt.contains("銘柄コード | 銘柄名 | 保有株数 | 現在値"));
    assert!(prompt.contains("9904 | 架空自動車 | 100 | ¥3,000"));
    assert!(!prompt.contains("平均取得単価 |"));
}

#[test]
fn generate_asset_review_prompt_allows_codes_only_table() {
    let pref = ReviewPromptPref {
        preamble: String::new(),
        columns: vec![],
    };
    let prompt = generate_asset_review_prompt(&[toyota()], &pref);
    assert!(prompt.contains("銘柄コード | 銘柄名\n9904 | 架空自動車"));
}
