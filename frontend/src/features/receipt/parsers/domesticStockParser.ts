import { DomesticStockData } from '@/lib/interfaces/domesticStock';

/**
 * 取引日でソート
 */
export const sortDomesticStockByTradeDate = (data: DomesticStockData[]): DomesticStockData[] => {
    return [...data].sort((a, b) =>
        a.trade_date.getTime() - b.trade_date.getTime()
    );
};

/**
 * DBレスポンスをDomesticStockDataに変換
 */
export const transformDBDomesticStock = (item: Record<string, unknown>): DomesticStockData => {
    const securityCode = String(item.security_code || '');
    const securityName = String(item.security_name || '');

    return {
        trade_date: new Date(item.trade_date as string),
        settlement_date: new Date(item.settlement_date as string),
        security_code: securityCode,
        security_name: securityName,
        account: String(item.account || ''),
        shares: Number(item.shares) || 0,
        asked_price: Number(item.asked_price) || 0,
        proceeds: Number(item.proceeds) || 0,
        purchase_price: Number(item.purchase_price) || 0,
        realized_profit_and_loss: Number(item.realized_profit_and_loss) || 0,
        taxes: Number(item.taxes) || 0,
        realized_profit_and_loss_after_tax: Number(item.realized_profit_and_loss_after_tax) || 0,
    };
};
