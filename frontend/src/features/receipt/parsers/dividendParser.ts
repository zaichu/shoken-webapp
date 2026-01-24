import { DividendData } from '@/lib/interfaces/dividend';
import { parseNumber } from '@/lib/utils/formatters';
import { createSecurityCodeLink } from './securityLink';

/**
 * CSVアイテムをDividendDataに変換
 */
export const parseDividendCsvItem = (item: Record<string, unknown>): DividendData => {
    const securityCode = String(item['銘柄コード'] || '');
    const securityName = String(item['銘柄'] || '');

    return {
        settlement_date: new Date(item['入金日'] as string),
        product: String(item['商品'] || ''),
        account: String(item['口座'] || ''),
        security_code: securityCode,
        security_name: securityName,
        security_info: createSecurityCodeLink(securityCode),
        unit_price: parseNumber(item['単価[円/現地通貨]']),
        shares: parseNumber(item['数量[株/口]']),
        dividends_before_tax: parseNumber(item['配当・分配金合計（税引前）[円/現地通貨]']),
        taxes: parseNumber(item['税額合計[円/現地通貨]']),
        net_amount_received: parseNumber(item['受取金額[円/現地通貨]']),
    };
};

/**
 * 決済日でソート
 */
export const sortDividendBySettlementDate = (data: DividendData[]): DividendData[] => {
    return [...data].sort((a, b) =>
        a.settlement_date.getTime() - b.settlement_date.getTime()
    );
};

/**
 * DBレスポンスをDividendDataに変換
 */
export const transformDBDividend = (item: Record<string, unknown>): DividendData => {
    const securityCode = String(item.security_code || '');
    const securityName = String(item.security_name || '');

    return {
        settlement_date: new Date(item.settlement_date as string),
        product: String(item.product || ''),
        account: String(item.account || ''),
        security_code: securityCode,
        security_name: securityName,
        security_info: createSecurityCodeLink(securityCode),
        unit_price: Number(item.unit_price) || 0,
        shares: Number(item.shares) || 0,
        dividends_before_tax: Number(item.dividends_before_tax) || 0,
        taxes: Number(item.taxes) || 0,
        net_amount_received: Number(item.net_amount_received) || 0,
    };
};
