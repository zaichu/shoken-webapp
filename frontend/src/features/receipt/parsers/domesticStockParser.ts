import type { DomesticStockApiData, DomesticStockData } from '@/features/receipt/types';

type DomesticStockSource = Partial<DomesticStockApiData> | Record<string, unknown>;

/**
 * 取引日で降順（新しい順）にソート
 */
export const sortDomesticStockByTradeDate = (data: DomesticStockData[]): DomesticStockData[] => {
    return [...data].sort((a, b) =>
        b.trade_date.getTime() - a.trade_date.getTime()
    );
};

/**
 * DBレスポンスをDomesticStockDataに変換
 */
export const transformDBDomesticStock = (item: DomesticStockSource): DomesticStockData => {
    const record = item as Record<string, unknown>;
    const securityCode = String(record.security_code || '');
    const securityName = String(record.security_name || '');

    return {
        trade_date: new Date(record.trade_date as string),
        settlement_date: new Date(record.settlement_date as string),
        security_code: securityCode,
        security_name: securityName,
        account: String(record.account || ''),
        shares: Number(record.shares) || 0,
        asked_price: Number(record.asked_price) || 0,
        proceeds: Number(record.proceeds) || 0,
        purchase_price: Number(record.purchase_price) || 0,
        realized_profit_and_loss: Number(record.realized_profit_and_loss) || 0,
        taxes: Number(record.taxes) || 0,
        realized_profit_and_loss_after_tax: Number(record.realized_profit_and_loss_after_tax) || 0,
    };
};
