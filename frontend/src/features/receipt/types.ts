/**
 * 共通のフィールドヘッダー
 */
export const HEADERS: Record<string, string> = {
  settlement_date: "入金日",
  product: "商品",
  account: "口座",
  security_code: "銘柄コード",
  security_name: "銘柄名",
  currency: "受取通貨",
  unit_price: "単価",
  shares: "数量[株/口]",
  dividends_before_tax: "配当・分配金",
  taxes: "税額",
  net_amount_received: "受取金額",
  total_dividends_before_tax: "合計配当・分配金",
  total_taxes: "合計税額",
  total_net_amount_received: "受取金額",

  // 株式取引関連
  transaction_date: "取引日",
  transaction_type: "取引種別",
  price: "価格",
  quantity: "数量",
  amount: "金額",
  profit_loss: "損益",

  // 投資信託関連
  fund_code: "ファンドコード",
  fund_name: "ファンド名",
  nav: "基準価額"
};

/**
 * 配当金データモデル
 */
export interface DividendItem {
  id: string;
  settlement_date?: Date;
  product?: string;
  account?: string;
  security_code?: string;
  security_name?: string;
  currency?: string;
  unit_price?: string;
  shares?: number;
  dividends_before_tax?: number;
  taxes?: number;
  net_amount_received?: number;
}

/**
 * 配当金サマリー
 */
export interface DividendSummary {
  total_dividends_before_tax: number;
  total_taxes: number;
  total_net_amount_received: number;
}

/**
 * 株式取引データモデル
 */
export interface StockTransaction {
  id: string;
  transaction_date?: Date;
  security_code?: string;
  security_name?: string;
  transaction_type?: '買付' | '売却';
  price?: number;
  quantity?: number;
  amount?: number;
}

/**
 * 投資信託データモデル
 */
export interface FundTransaction {
  id: string;
  transaction_date?: Date;
  fund_code?: string;
  fund_name?: string;
  transaction_type?: '買付' | '売却' | '分配金';
  amount?: number;
}
