use super::*;
use serde_json::json;

fn dividend(fields: serde_json::Value) -> Dividend {
    let mut base = json!({
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "settlement_date": "2024-03-01",
        "product": "特定口座",
        "account": "架空証券",
        "security_code": "9904",
        "security_name": "架空自動車",
        "unit_price": 30.0,
        "shares": 100,
        "dividends_before_tax": 3000,
        "taxes": 609,
        "net_amount_received": 2391,
        "created_at": "2024-03-01T00:00:00Z",
        "updated_at": "2024-03-01T00:00:00Z"
    });
    base.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    serde_json::from_value(base).expect("dividend")
}

fn stock(fields: serde_json::Value) -> DomesticStock {
    let mut base = json!({
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "trade_date": "2024-04-10",
        "settlement_date": "2024-04-15",
        "security_code": "9910",
        "security_name": "架空エレクトロニクス",
        "account": "特定",
        "shares": 50,
        "asked_price": 2500.5,
        "proceeds": 125025,
        "purchase_price": 100000,
        "realized_profit_and_loss": 25025,
        "taxes": 5082,
        "realized_profit_and_loss_after_tax": 19943,
        "created_at": "2024-04-10T00:00:00Z",
        "updated_at": "2024-04-10T00:00:00Z"
    });
    base.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    serde_json::from_value(base).expect("stock")
}

fn fund(fields: serde_json::Value) -> Mutualfund {
    let mut base = json!({
        "id": "550e8400-e29b-41d4-a716-446655440003",
        "trade_date": "2024-05-20",
        "settlement_date": "2024-05-23",
        "fund_name": "架空バランスファンド",
        "dividends": null,
        "account": "NISA",
        "shares": 12345.6,
        "exchange_rate": 1.0,
        "cancellation_unit_price_yen": 15234,
        "cancellation_amount_yen": 188062,
        "average_acquisition_price_yen": 14000,
        "realized_profit_and_loss": 15240,
        "taxes": 3095,
        "realized_profit_and_loss_after_tax": 12145,
        "created_at": "2024-05-20T00:00:00Z",
        "updated_at": "2024-05-20T00:00:00Z"
    });
    base.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    serde_json::from_value(base).expect("fund")
}

fn balance(fields: serde_json::Value) -> AssetBalance {
    let mut base = json!({
        "id": "550e8400-e29b-41d4-a716-446655440004",
        "security_code": "9904",
        "security_name": "架空自動車",
        "shares": 100,
        "average_purchase_price": 2000,
        "total_purchase_amount": 200000,
        "current_price": 2500,
        "daily_change": 15,
        "price_as_of": "2024-06-01",
        "created_at": "2024-06-01T00:00:00Z",
        "updated_at": "2024-06-01T00:00:00Z"
    });
    base.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    serde_json::from_value(base).expect("balance")
}

#[test]
fn dividends_csv_has_screen_columns_in_date_desc_order() {
    let older = dividend(json!({"settlement_date": "2024-01-05"}));
    let newer = dividend(json!({
        "settlement_date": "2024-05-10",
        "security_code": "9999",
        "security_name": "架空商事"
    }));
    let same_day_lower_code = dividend(json!({
        "settlement_date": "2024-05-10",
        "security_code": "1301",
        "security_name": "架空水産"
    }));

    let csv = dividends_csv(&[newer, older, same_day_lower_code]);
    let lines: Vec<&str> = csv.lines().collect();

    assert_eq!(
        lines[0],
        "入金日,商品,口座,銘柄コード,銘柄名,1株配当,数量,配当金,税額,税引後"
    );
    assert_eq!(lines.len(), 4);
    // 新しい入金日が先。同日は銘柄コード昇順
    assert!(lines[1].starts_with("2024-05-10,特定口座,架空証券,1301,架空水産"));
    assert!(lines[2].starts_with("2024-05-10,特定口座,架空証券,9999,架空商事"));
    assert!(lines[3].starts_with("2024-01-05,"));
    // 数値は桁区切り・¥なしの生値
    assert!(lines[3].ends_with(",30,100,3000,609,2391"));
}

#[test]
fn asset_balances_csv_sorts_by_market_value_desc() {
    let low = balance(json!({
        "security_code": "1301",
        "security_name": "架空水産",
        "shares": 10,
        "total_purchase_amount": 10000,
        "current_price": 1000
    }));
    let unpriced = balance(json!({
        "security_code": "9999",
        "security_name": "架空商事",
        "current_price": 0,
        "price_as_of": null
    }));
    let high = balance(json!({
        "security_code": "9910",
        "security_name": "架空エレクトロニクス",
        "shares": 100,
        "average_purchase_price": 1500,
        "total_purchase_amount": 150000,
        "current_price": 3000
    }));

    let csv = asset_balances_csv(&[low.clone(), unpriced, high]);
    let lines: Vec<&str> = csv.lines().collect();

    assert_eq!(
        lines[0],
        "銘柄コード,銘柄名,保有数,平均取得単価,取得額,現在値,価格基準日,評価額,含み損益"
    );
    // 評価額 300,000(9910) → 10,000(1301) → 価格なし(9999)
    assert!(
        lines[1]
            .starts_with("9910,架空エレクトロニクス,100,1500,150000,3000,2024-06-01,300000,150000")
    );
    assert!(lines[2].starts_with("1301,架空水産,10,2000,10000,1000,2024-06-01,10000,0"));
    // 価格なし行は評価額・含み損益が空欄
    assert!(lines[3].starts_with("9999,架空商事,100,2000,200000,0,,,"));
}

#[test]
fn transactions_csv_combines_stocks_and_funds() {
    let stock_row = stock(json!({}));
    let fund_row = fund(json!({"trade_date": "2024-04-25"}));

    let csv = transactions_csv(&[stock_row], &[fund_row]);
    let lines: Vec<&str> = csv.lines().collect();

    assert_eq!(
        lines[0],
        "種別,約定日,銘柄コード,銘柄名,口座,数量,単価,売却・解約額,取得価額,実現損益,税額,税引後"
    );
    // 日付の新しい方(投資信託 2024-04-25)が先
    assert_eq!(
        lines[1],
        "投資信託,2024-04-25,,架空バランスファンド,NISA,12345.6,15234,188062,14000,15240,3095,12145"
    );
    assert_eq!(
        lines[2],
        "国内株式,2024-04-10,9910,架空エレクトロニクス,特定,50,2500.5,125025,100000,25025,5082,19943"
    );
}

#[test]
fn csv_escapes_commas_quotes_and_newlines() {
    let row = dividend(json!({
        "security_name": "架空\"重工\",株式会社\n第二"
    }));
    let csv = dividends_csv(&[row]);
    assert!(csv.contains("\"架空\"\"重工\"\",株式会社\n第二\""));
}

#[test]
fn csv_neutralizes_text_cells_that_look_like_formulas() {
    // 銘柄名などの文字列が = + - @ 始まりだと表計算ソフトで式として実行されるため ' で逃がす
    let csv = dividends_csv(&[dividend(json!({"security_name": "=cmd|' /C calc'!A0"}))]);
    assert!(csv.contains("'=cmd|' /C calc'!A0"));
    let csv = asset_balances_csv(&[balance(json!({"security_name": "@x"}))]);
    assert!(csv.contains("'@x"));
    // 数値の負値は数値として扱わせるため逃がさない
    let csv = transactions_csv(&[stock(json!({"realized_profit_and_loss": -12345}))], &[]);
    assert!(csv.contains(",-12345,"));
}

#[test]
fn export_filenames_use_label_and_date() {
    assert_eq!(
        ExportKind::Dividend.filename("2026-01-02"),
        "dividends_2026-01-02.csv"
    );
    assert_eq!(
        ExportKind::AssetBalance.filename("2026-01-02"),
        "asset-balances_2026-01-02.csv"
    );
    assert_eq!(
        ExportKind::Transaction.filename("2026-01-02"),
        "transactions_2026-01-02.csv"
    );
}

#[test]
fn csv_exports_never_contain_auth_or_identity_fields() {
    // id・作成/更新日時・user_id など一覧画面に出さない内部管理値は列に出さない。
    // メールアドレス等の認証情報は行の取得元(DTO)にそもそも含まれないが、
    // 列名に個人情報系の名前が混ざっていないことを併せて確かめる
    let csv_set = [
        dividends_csv(&[dividend(json!({}))]),
        asset_balances_csv(&[balance(json!({}))]),
        transactions_csv(&[stock(json!({}))], &[fund(json!({}))]),
    ];
    for csv in csv_set {
        assert!(!csv.contains("550e8400"), "{csv}");
        for forbidden in ["user_id", "created_at", "updated_at", "id", "@", "メール"] {
            assert!(
                !csv.lines().next().unwrap().contains(forbidden),
                "{forbidden} in {csv}"
            );
        }
    }
}
