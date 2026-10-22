// 撮影用の架空データ。実保有データ・本番の接続情報は使わない。
// 列の折り返し・桁のそろえ・スクロール時の挙動を確かめるため、
// 長い名前・2桁の月・複数口座・小数を含む行を 30 件以上用意する。
// 銘柄コードは 9900 番台の架空番号、金額は規則的な値にする。
//
// 保有口数・株数は同じ添字の行で揃える(資産管理・取引明細・ホームの数字が矛盾しない)。

const STOCK_NAMES = [
  '架空自動車',
  '架空エレクトロニクス',
  '架空精密機器',
  '架空食品工業',
  '架空化学工業',
  '架空鉄鋼',
  '架空銀行',
  '架空証券',
  '架空損害保険',
  '架空不動産',
  '架空建設工業',
  '架空海運',
  '架空航空',
  '架空鉄道',
  '架空電力',
  '架空通信網',
  '架空ソフトウェア',
  '架空半導体',
  '架空電機',
  '架空機械',
  '架空資源',
  '架空石油開発',
  '架空製薬',
  '架空卸売業',
  '架空小売業',
  '架空商業施設',
  '架空物流',
  '架空広告代理',
  '架空出版',
  '架空教育サービス',
  '架空情報サービス',
  '架空環境技術',
  '架空再生可能エネルギー',
  '架空バイオテクノロジー',
  '架空グローバル先端素材ソリューションホールディングス',
  '架空先端産業技術研究開発機構',
];

const FUND_NAMES = [
  '架空バランスファンドA号',
  '架空インデックスファンドB号',
  '架空先進地域医療介護サービスファンド',
  '架空世界株インデックスファンド',
  '架空国内株式アクティブファンド',
  '架空高利回り短期債券ファンド',
  '架空株式市場インデックスファンド',
  '架空REITインデックスファンド',
  '架空インフラストラクチャファンド',
  '架空環境技術成長ファンド',
  '架空新興国株式ファンド',
  '架空グローバル株式アクティブファンド',
  '架空配当成長ファンド',
  '架空中小型株ファンド',
  '架空債券インデックスファンド',
  '架空年金これからファンド',
  '架空健康医療テーマファンド',
  '架空食品分野ファンド',
  '架空医療改革テーマファンド',
  '架空不動産的直接保有ファンド',
  '架空株式中長期成長ファンド',
  '架空グローバル債券ファンド',
  '架空宇宙開発テーマファンド',
  '架空DXテーマファンド',
  '架空自治体債ファンド',
  '架空先端サービス株ファンド',
  '架空成長株集中ファンド',
  '架空高配当株式ファンド',
  '架空エネルギートランジションファンド',
  '架空長期水池ファンド',
  '架空・世界革新テクノロジー株式インデックスファンド＜購入・換金手数料なし＞',
  '架空グローバル先端素材ソリューションホールディングス関連ファンド',
  '架空環境技術・再生可能エネルギーセクターファンド',
  '架空先端産業技術研究開発機構テーマファンド',
  '架空人生設計・長期安全感・資産形成ファンド',
  '架空定期預金型ファンド',
];

const ACCOUNTS = [
  '特定',
  '一般',
  'NISA成長投資枠',
  'NISAつみたて投資枠',
  '旧NISA',
  '特定・一般',
];

const DIVIDEND_PRODUCTS = ['国内株式', '特別利益分配金', '中間分配金'];

// 小数点以下を持つ金額を作る。列の中の桁がそろうことを確かめるため
function decimal(base, step, digits = 2) {
  return Number((base + step).toFixed(digits));
}

function taxOf(amount) {
  return Number(Math.floor(amount * 0.20315).toFixed(2));
}

// 2桁の月(10・11・12)を含む日付を作る
function settlementDate(index, year = 2026) {
  const month = (index % 12) + 1;
  const day = ((index * 7) % 27) + 1;
  return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}

// 同一の添字の行が同じ保有数・同じ取得単価を持つようにする
function stockShares(index) {
  return 20 + index * 2;
}

function fundShares(index) {
  return 20000 + index * 2000;
}

export function assetBalances() {
  const rows = STOCK_NAMES.map((security_name, index) => {
    const shares = stockShares(index);
    const average_purchase_price = decimal(1500, index * 20);
    const current_price = decimal(1550, index * 21);
    const daily_change = decimal(index % 3 === 0 ? -25 : 20, index * 0.35);
    return {
      id: `asset-balance-${index + 1}`,
      security_code: String(9901 + index),
      security_name,
      account: index % 7 === 3 ? null : ACCOUNTS[index % ACCOUNTS.length],
      shares,
      average_purchase_price,
      total_purchase_amount: Number((shares * average_purchase_price).toFixed(2)),
      current_price,
      daily_change,
      price_as_of: '2026-10-20',
      // 前日終値が無い行(上場直後など)の「—」表示を見る
      prev_close: index % 11 === 5 ? null : Number((current_price - daily_change).toFixed(2)),
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
    };
  });
  // 集計は画面と同じ計算式で作る
  // (評価額=現在値×株数、前日比=前日比×株数、前日終値が無い行は前日比から除く)
  const priced = rows.filter((row) => row.prev_close !== null);
  const sum = (rows_, key) => Number(rows_.reduce((total, row) => total + row[key], 0).toFixed(2));
  const market = rows.reduce((total, row) => total + row.shares * row.current_price, 0);
  const daily = priced.reduce((total, row) => total + row.daily_change * row.shares, 0);
  return {
    rows,
    summary: {
      total_market_value: Number(market.toFixed(2)),
      total_purchase_amount: sum(rows, 'total_purchase_amount'),
      total_daily_change: Number(daily.toFixed(2)),
      priced_market_value: Number(market.toFixed(2)),
      priced_purchase_amount: sum(rows, 'total_purchase_amount'),
      has_missing_daily_change: rows.length !== priced.length,
    },
  };
}

export function fundBalances() {
  const rows = FUND_NAMES.map((fund_name, index) => {
    const shares = fundShares(index);
    const average_purchase_price = decimal(9000, index * 100);
    const base_price = decimal(9500, index * 110);
    const market_value = Number(((shares * base_price) / 10000).toFixed(2));
    const total_purchase_amount = Number(((shares * average_purchase_price) / 10000).toFixed(2));
    const profit_loss = Number((market_value - total_purchase_amount).toFixed(2));
    return {
      id: `fund-balance-${index + 1}`,
      fund_name,
      account: ACCOUNTS[index % ACCOUNTS.length],
      dividends_course: index % 3 === 0 ? '受取型' : '再投資型',
      shares,
      average_purchase_price,
      total_purchase_amount,
      base_price,
      base_price_day_change: decimal(index % 4 === 0 ? -95 : 80, index * 1.25),
      base_price_month_change: decimal(index % 5 === 0 ? -260 : 210, index * 3.5),
      market_value,
      profit_loss,
      profit_loss_rate: Number(((profit_loss / total_purchase_amount) * 100).toFixed(2)),
      total_return: index % 6 === 0 ? null : Number((profit_loss * 1.2).toFixed(2)),
      price_as_of: '2026-10-20',
      price_source: index % 2 === 0 ? 'api' : 'csv',
      nav_is_latest: true,
      expected_nav_date: null,
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
    };
  });
  // 前日比は「基準価額(前日比)×保有口数÷10000」の合計
  const daily_change = rows.reduce(
    (total, row) => total + (row.shares * row.base_price_day_change) / 10000,
    0,
  );
  const sum = (key) => Number(rows.reduce((total, row) => total + row[key], 0).toFixed(2));
  return {
    rows,
    summary: {
      total_market_value: sum('market_value'),
      total_purchase_amount: sum('total_purchase_amount'),
      total_daily_change: Number(daily_change.toFixed(2)),
      latest_nav_count: rows.length,
      pending_nav_count: 0,
      nav_status: {
        updated_at: '2026-10-20 18:05',
        // 対象+対象外=全件になるよう、取得数は api 由来の行数に合わせる
        fetched_count: rows.filter((row) => row.price_source === 'api').length,
        failed_count: 0,
        failure_reason: null,
      },
    },
  };
}

// 配当。ホームの前年同期比較があるため年を指定して取り分ける
export function dividends(year = 2026) {
  const rows = STOCK_NAMES.map((security_name, index) => {
    const unit_price = decimal(20, index * 1.5);
    const shares = stockShares(index);
    const dividends_before_tax = Number((unit_price * shares).toFixed(2));
    const taxes = taxOf(dividends_before_tax);
    return {
      id: `dividend-${year}-${index + 1}`,
      settlement_date: settlementDate(index, year),
      product: DIVIDEND_PRODUCTS[index % DIVIDEND_PRODUCTS.length],
      account: ACCOUNTS[index % ACCOUNTS.length],
      security_code: String(9901 + index),
      security_name,
      unit_price,
      shares,
      dividends_before_tax,
      taxes,
      net_amount_received: Number((dividends_before_tax - taxes).toFixed(2)),
      created_at: `${year}-01-01T00:00:00Z`,
      updated_at: `${year}-01-01T00:00:00Z`,
    };
  });
  const sum = (key) => Number(rows.reduce((total, row) => total + row[key], 0).toFixed(2));
  return {
    rows,
    summary: {
      total_dividends_before_tax: sum('dividends_before_tax'),
      total_taxes: sum('taxes'),
      total_net_amount_received: sum('net_amount_received'),
    },
  };
}

export function domesticStockTransactions() {
  const rows = STOCK_NAMES.map((security_name, index) => {
    const shares = stockShares(index);
    const asked_price = decimal(1750, index * 19);
    const purchase_price = decimal(1500, index * 18);
    const proceeds = Number((shares * asked_price).toFixed(2));
    const realized_profit_and_loss = Number((proceeds - shares * purchase_price).toFixed(2));
    const taxes = taxOf(realized_profit_and_loss);
    return {
      id: `domestic-stock-${index + 1}`,
      trade_date: settlementDate(index),
      settlement_date: settlementDate(index + 2),
      security_code: String(9901 + index),
      security_name,
      account: ACCOUNTS[index % ACCOUNTS.length],
      shares,
      asked_price,
      proceeds,
      purchase_price,
      realized_profit_and_loss,
      taxes,
      realized_profit_and_loss_after_tax: Number((realized_profit_and_loss - taxes).toFixed(2)),
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
    };
  });
  const sum = (key) => Number(rows.reduce((total, row) => total + row[key], 0).toFixed(2));
  return {
    rows,
    summary: {
      total_realized_profit_and_loss: sum('realized_profit_and_loss'),
      total_taxes: sum('taxes'),
      total_realized_profit_and_loss_after_tax: sum('realized_profit_and_loss_after_tax'),
    },
  };
}

export function mutualFundTransactions() {
  const rows = FUND_NAMES.map((fund_name, index) => {
    const shares = fundShares(index);
    const cancellation_unit_price_yen = decimal(9500, index * 105);
    const cancellation_amount_yen = Number(((shares * cancellation_unit_price_yen) / 10000).toFixed(2));
    const average_acquisition_price_yen = decimal(9000, index * 95);
    const realized_profit_and_loss = Number(
      (cancellation_amount_yen - (shares * average_acquisition_price_yen) / 10000).toFixed(2),
    );
    const taxes = taxOf(realized_profit_and_loss);
    return {
      id: `mutual-fund-${index + 1}`,
      trade_date: settlementDate(index),
      settlement_date: settlementDate(index + 2),
      fund_name,
      dividends: index % 3 === 0 ? '受取型' : '再投資型',
      account: ACCOUNTS[index % ACCOUNTS.length],
      shares,
      exchange_rate: 1,
      cancellation_unit_price_yen,
      cancellation_amount_yen,
      average_acquisition_price_yen,
      realized_profit_and_loss,
      taxes,
      realized_profit_and_loss_after_tax: Number((realized_profit_and_loss - taxes).toFixed(2)),
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
    };
  });
  const sum = (key) => Number(rows.reduce((total, row) => total + row[key], 0).toFixed(2));
  return {
    rows,
    summary: {
      total_realized_profit_and_loss: sum('realized_profit_and_loss'),
      total_taxes: sum('taxes'),
      total_realized_profit_and_loss_after_tax: sum('realized_profit_and_loss_after_tax'),
    },
  };
}

// 銘柄検索の結果。1件だけ返す
export function stock(code) {
  return {
    code: code ?? '9901',
    name: STOCK_NAMES[0],
    date: '2026-10-20',
    market_category: 'プライム',
    industry_category_33: '情報・通信業',
    industry_category_17: '情報通信・サービスその他',
    size_category: 'TOPIX Large70',
  };
}

// 保有銘柄の1株配当の見積もり
export function dividendEstimates() {
  return {
    items: STOCK_NAMES.map((_, index) => ({
      security_code: String(9901 + index),
      dividend_per_share: index % 4 === 0 ? 0 : decimal(20, index * 1.5),
      status: index % 4 === 0 ? 'zero' : 'ok',
    })),
  };
}

export const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};
