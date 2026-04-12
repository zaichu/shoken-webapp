import type { DividendApiData, DividendData } from '@/lib/interfaces/dividend';

type DividendSource = Partial<DividendApiData> | Record<string, unknown>;

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
export const transformDBDividend = (item: DividendSource): DividendData => {
    const record = item as Record<string, unknown>;
    const securityCode = String(record.security_code || '');
    const securityName = String(record.security_name || '');

    return {
        settlement_date: new Date(record.settlement_date as string),
        product: String(record.product || ''),
        account: String(record.account || ''),
        security_code: securityCode,
        security_name: securityName,
        unit_price: Number(record.unit_price) || 0,
        shares: Number(record.shares) || 0,
        dividends_before_tax: Number(record.dividends_before_tax) || 0,
        taxes: Number(record.taxes) || 0,
        net_amount_received: Number(record.net_amount_received) || 0,
    };
};
