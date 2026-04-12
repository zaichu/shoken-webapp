import type { MutualfundApiData, MutualfundData } from '@/lib/interfaces/mutualfund';

type MutualfundSource = Partial<MutualfundApiData> | Record<string, unknown>;

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
export const transformDBMutualfund = (item: MutualfundSource): MutualfundData => {
    const record = item as Record<string, unknown>;

    return {
        trade_date: new Date(record.trade_date as string),
        settlement_date: new Date(record.settlement_date as string),
        fund_name: String(record.fund_name || ''),
        dividends: String(record.dividends || ''),
        account: String(record.account || ''),
        shares: Number(record.shares) || 0,
        exchange_rate: Number(record.exchange_rate) || 0,
        cancellation_unit_price_yen: Number(record.cancellation_unit_price_yen) || 0,
        cancellation_amount_yen: Number(record.cancellation_amount_yen) || 0,
        average_acquisition_price_yen: Number(record.average_acquisition_price_yen) || 0,
        realized_profit_and_loss: Number(record.realized_profit_and_loss) || 0,
        taxes: Number(record.taxes) || 0,
        realized_profit_and_loss_after_tax: Number(record.realized_profit_and_loss_after_tax) || 0,
    };
};
