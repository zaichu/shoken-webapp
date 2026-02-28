import { MutualfundData } from '@/lib/interfaces/mutualfund';

/**
 * 取引日でソート
 */
export const sortMutualfundByTradeDate = (data: MutualfundData[]): MutualfundData[] => {
    return [...data].sort((a, b) =>
        a.trade_date.getTime() - b.trade_date.getTime()
    );
};

/**
 * DBレスポンスをMutualfundDataに変換
 */
export const transformDBMutualfund = (item: Record<string, unknown>): MutualfundData => {
    return {
        trade_date: new Date(item.trade_date as string),
        settlement_date: new Date(item.settlement_date as string),
        fund_name: String(item.fund_name || ''),
        dividends: String(item.dividends || ''),
        account: String(item.account || ''),
        shares: Number(item.shares) || 0,
        exchange_rate: Number(item.exchange_rate) || 0,
        cancellation_unit_price_yen: Number(item.cancellation_unit_price_yen) || 0,
        cancellation_amount_yen: Number(item.cancellation_amount_yen) || 0,
        average_acquisition_price_yen: Number(item.average_acquisition_price_yen) || 0,
        realized_profit_and_loss: Number(item.realized_profit_and_loss) || 0,
        taxes: Number(item.taxes) || 0,
        realized_profit_and_loss_after_tax: Number(item.realized_profit_and_loss_after_tax) || 0,
    };
};
