import { DividendItem, DividendSummary } from './types';
import { parseDate, parseNumberString } from '../../lib/utils/format';

interface CSVRecord {
  '入金日': string;
  '商品': string;
  '口座': string;
  '銘柄コード': string;
  '銘柄': string;
  '受取通貨': string;
  '単価[円/現地通貨]': string;
  '数量[株/口]': string | null;
  '配当・分配金合計（税引前）[円/現地通貨]': string | null;
  '税額合計[円/現地通貨]': string | null;
  '受取金額[円/現地通貨]': string | null;
}

/**
 * CSVレコードから配当金データへの変換
 * @param record CSVレコード
 * @param index インデックス
 * @returns 配当金データ
 */
export function parseDividendItemFromCSV(record: CSVRecord, index: number): DividendItem {
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

/**
 * 配当金サマリーの計算
 * @param items 配当金データ配列
 * @returns 配当金サマリー
 */
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

/**
 * 銘柄コードでの検索
 * @param items 配当金データ配列
 * @param query 検索クエリ
 * @returns 検索結果
 */
export function searchBySecurityCode(items: DividendItem[], query: string): DividendItem[] {
  if (!query) return items;

  return items.filter(
    item => item.security_code === query || item.security_name === query
  );
}

/**
 * 日付でソート（昇順）
 * @param items ソート対象配列
 * @returns ソート結果
 */
export function sortByDate<T extends { settlement_date?: Date; transaction_date?: Date }>(
  items: T[]
): T[] {
  return [...items].sort((a, b) => {
    const dateA = a.settlement_date || a.transaction_date || new Date(0);
    const dateB = b.settlement_date || b.transaction_date || new Date(0);
    return dateA.getTime() - dateB.getTime();
  });
}

/**
 * 日付のフォーマット（YYYY/MM/DD）
 * @param date 日付
 * @returns フォーマットされた日付文字列
 */
export function formatDate(date?: Date | null): string {
  if (!date) return '';
  return `${date.getFullYear()}/${(date.getMonth() + 1).toString().padStart(2, '0')}/${date.getDate().toString().padStart(2, '0')}`;
}

/**
 * 数値の金額フォーマット
 * @param amount 金額
 * @returns フォーマットされた金額文字列
 */
export function formatCurrency(amount?: number | null): string {
  if (amount === undefined || amount === null) return '';
  return `¥ ${amount.toLocaleString()}`;
}
