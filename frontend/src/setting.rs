use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

type HeaderMap = HashMap<&'static str, &'static str>;

lazy_static! {
    pub static ref HEADERS: HeaderMap = {
        let mut map = HashMap::new();
        map.insert("trade_date", "約定日");
        map.insert("settlement_date", "受渡日");
        map.insert("security_code", "銘柄コード");
        map.insert("security_name", "銘柄名");
        map.insert("account", "口座");
        map.insert("shares", "数量[株]");
        map.insert("asked_price", "売却/決済単価");
        map.insert("proceeds", "売却/決済額");
        map.insert("purchase_price", "平均取得価額");
        map.insert("realized_profit_and_loss", "実現損益");
        map.insert("realized_profit_and_loss_after_tax", "実現損益(税引)");
        map.insert("total_realized_profit_and_loss", "合計実現損益");
        map.insert("total_realized_profit_and_loss_after_tax", "合計実現損益(税引)");
        map.insert("product", "商品");
        map.insert("currency", "受取通貨");
        map.insert("unit_price", "単価");
        map.insert("dividends_before_tax", "配当・分配金");
        map.insert("taxes", "税額");
        map.insert("net_amount_received", "受取金額");
        map.insert("total_dividends_before_tax", "合計配当・分配金");
        map.insert("total_taxes", "合計税額");
        map.insert("total_net_amount_received", "合計受取金額");
        map.insert("cancellation_unit_price_yen", "解約単価［円］");
        map.insert("cancellation_amount_yen", "解約額［円］");
        map.insert("average_acquisition_price_yen", "平均取得価額［円］");
        map.insert("fund_name", "ファンド名");
        map.insert("dividends", "分配金");
        map.insert("exchange_rate", "為替レート");

        map
    };

    pub static ref NUMBER_FORMAT_KEYS: HashSet<&'static str> = [
        "shares", // 数量
    ].iter().cloned().collect();

    pub static ref YEN_FORMAT_KEYS: HashSet<&'static str> = [
        "asked_price",                              // 売却/決済単価
        "dividends_before_tax",                     // 配当・分配金(税引前)
        "net_amount_received",                      // 受取金額
        "proceeds",                                 // 売却/決済額
        "total_realized_profit_and_loss_after_tax", // 損益
        "purchase_price",                           // 平均取得価額
        "realized_profit_and_loss",                 // 実現損益
        "taxes",                                    // 税額
        "total_dividends_before_tax",               // 配当・分配金合計(税引前)
        "total_net_amount_received",                // 受取金額合計
        "total_realized_profit_and_loss",           // 合計実現損益
        "total_taxes",                              // 税額合計
        "withholding_tax",                          // 源泉徴収税額
        "unit_price",                               // 単価
        "cancellation_unit_price_yen",              // 解約単価［円］
        "cancellation_amount_yen",                  // 解約額［円］
        "average_acquisition_price_yen",            // 平均取得価額［円］
        "realized_profit_and_loss_after_tax",       // 実現損益(税抜)
    ].iter().cloned().collect();

    pub static ref DATE_FORMAT_KEYS: HashSet<&'static str> = [
        "settlement_date", // 受取金
        "trade_date",      // 約定日
    ].iter().cloned().collect();

    pub static ref STOCK_INFO_LINKS: Vec<(&'static str, &'static str)> = vec![
        ("楽天証券", RAKUTEN_URL),
        ("SBI証券", SBI_URL),
        ("株探", KABUTAN_URL),
        ("Yahoo! Finance", YAHOO_URL),
        ("日経", NIKKEI_URL),
        ("バフェットコード", BUFFETT_CODE_URL),
        ("みんかぶ", MINKABU_URL),
        ("IR BANK", IR_BANK_URL),
        ("銘柄スカウター", SCOUTER_URL),
        ("ザイマニ", ZAIMANI_URL),
    ];
}

pub const TAX_RATE: f64 = 0.20315;

pub const MINKABU_URL: &'static str = "https://minkabu.jp/stock/{}/";
pub const KABUTAN_URL: &'static str = "https://kabutan.jp/stock/?code={}";
pub const YAHOO_URL: &'static str = "https://finance.yahoo.co.jp/quote/{}";
pub const NIKKEI_URL: &'static str = "https://www.nikkei.com/nkd/company/?scode={}";
pub const BUFFETT_CODE_URL: &'static str = "https://www.buffett-code.com/company/{}";
pub const RAKUTEN_URL: &'static str =
    "https://www.rakuten-sec.co.jp/web/market/search/quote.html?ric={}.T";
pub const IR_BANK_URL: &'static str = "https://irbank.net/{}";
pub const ZAIMANI_URL: &'static str = "https://zaimani.com/search/?_sf_s={}";
pub const SCOUTER_URL: &'static str = "https://monex.ifis.co.jp/index.php?sa=report_index&bcode={}";
pub const SBI_URL: &'static str = "https://site3.sbisec.co.jp/ETGate/?_ControlID=WPLETsiR001Control&_DataStoreID=DSWPLETsiR001Control&_PageID=WPLETsiR001Ilst10&_ActionID=getDetailOfStockPriceJP&s_rkbn=1&i_stock_sec=%94%43%93%56%93%B0&i_dom_flg=1&i_exchange_code=JPN&i_output_type=0&stock_sec_code_mul={}";

pub const SHOKEN_WEB_API_URL: &'static str = "https://shoken-webapp-api-b4a1.shuttle.app";
