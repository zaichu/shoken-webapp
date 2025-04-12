// 共通のフィールドヘッダー
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

// 配当金データモデル
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

// 配当金サマリー
export interface DividendSummary {
  total_dividends_before_tax: number;
  total_taxes: number;
  total_net_amount_received: number;
}

// 株式取引データモデル
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

// 投資信託データモデル
export interface FundTransaction {
  id: string;
  transaction_date?: Date;
  fund_code?: string;
  fund_name?: string;
  transaction_type?: '買付' | '売却' | '分配金';
  amount?: number;
}

// CSVレコードから配当金データへの変換
export function parseDividendItemFromCSV(record: any, index: number): DividendItem {
  const nullToUndefined = <T>(value: T | null): T | undefined =>
    value === null ? undefined : value;

  return {
    id: `dividend-${index}`,
    settlement_date: nullToUndefined(parseDate(record['入金日'])),
    product: record['商品'],
    account: record['口座'],
    security_code: record['銘柄コード'],
    security_name: record['銘柄'],
    currency: record['受取通貨'],
    unit_price: record['単価[円/現地通貨]'],
    shares: nullToUndefined(parseNumberString(record['数量[株/口]'])),
    dividends_before_tax: nullToUndefined(parseNumberString(record['配当・分配金合計（税引前）[円/現地通貨]'])),
    taxes: nullToUndefined(parseNumberString(record['税額合計[円/現地通貨]'])),
    net_amount_received: nullToUndefined(parseNumberString(record['受取金額[円/現地通貨]'])),
  };
}

// 配当金サマリーの計算
export function calculateDividendSummary(items: DividendItem[]): DividendSummary {
  const summary = items.reduce(
    (acc, item) => {
      return {
        total_dividends_before_tax: acc.total_dividends_before_tax + (item.dividends_before_tax || 0),
        total_taxes: acc.total_taxes + (item.taxes || 0),
        total_net_amount_received: acc.total_net_amount_received + (item.net_amount_received || 0),
      };
    },
    {
      total_dividends_before_tax: 0,
      total_taxes: 0,
      total_net_amount_received: 0,
    }
  );

  return summary;
}

// 銘柄コードでの検索
export function searchBySecurityCode(items: DividendItem[], query: string): DividendItem[] {
  if (!query) return items;

  return items.filter(
    item => item.security_code === query || item.security_name === query
  );
}

// 日付でソート（昇順）
export function sortByDate<T extends { settlement_date?: Date; transaction_date?: Date }>(
  items: T[]
): T[] {
  return [...items].sort((a, b) => {
    const dateA = a.settlement_date || a.transaction_date || new Date(0);
    const dateB = b.settlement_date || b.transaction_date || new Date(0);
    return dateA.getTime() - dateB.getTime();
  });
}

// 日付のフォーマット（YYYY/MM/DD）
export function formatDate(date?: Date | null): string {
  if (!date) return '';
  return `${date.getFullYear()}/${(date.getMonth() + 1).toString().padStart(2, '0')}/${date.getDate().toString().padStart(2, '0')}`;
}

// 数値の金額フォーマット
export function formatCurrency(amount?: number | null): string {
  if (amount === undefined || amount === null) return '';
  return `¥ ${amount.toLocaleString()}`;
}

// CSV読み込み時のパーサーを日付取得用に修正
import { parseDate, parseNumberString } from '../services/csvUtils';
