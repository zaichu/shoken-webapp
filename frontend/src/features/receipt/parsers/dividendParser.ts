import { DividendData } from '@/lib/interfaces/dividend';

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
        unit_price: Number(item.unit_price) || 0,
        shares: Number(item.shares) || 0,
        dividends_before_tax: Number(item.dividends_before_tax) || 0,
        taxes: Number(item.taxes) || 0,
        net_amount_received: Number(item.net_amount_received) || 0,
    };
};
