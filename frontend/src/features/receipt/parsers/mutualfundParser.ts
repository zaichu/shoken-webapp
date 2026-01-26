import { MutualfundData } from '@/lib/interfaces/mutualfund';
import { parseNumber, TAX_RATE } from '@/lib/utils/formatters';

/**
 * データがDB形式かどうかを判定
 */
const isDBFormat = (item: Record<string, unknown>): boolean => {
    return 'trade_date' in item && 'fund_name' in item && !('約定日' in item);
};

/**
 * CSVまたはDBアイテムをMutualfundDataに変換
 */
export const parseMutualfundCsvItem = (item: Record<string, unknown>): MutualfundData => {
    // DB形式の場合はそのまま返す（既に変換済み）
    if (isDBFormat(item)) {
        return {
            trade_date: item.trade_date instanceof Date
                ? item.trade_date
                : new Date(item.trade_date as string),
            settlement_date: item.settlement_date instanceof Date
                ? item.settlement_date
                : new Date(item.settlement_date as string),
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
    }

    // CSV形式の場合
    const account = String(item['口座'] || '');
    const realizedPnL = parseNumber(item['実現損益［円］']);
    const isSpecificAccount = account.includes('特定');
    const taxes = isSpecificAccount ? Math.floor(Math.max(0, realizedPnL) * TAX_RATE) : 0;
    const realizedPnLAfterTax = realizedPnL - taxes;

    return {
        trade_date: new Date(item['約定日'] as string),
        settlement_date: new Date(item['受渡日'] as string),
        fund_name: String(item['ファンド名'] || ''),
        dividends: String(item['分配金'] || ''),
        account,
        shares: parseNumber(item['数量[口]']),
        exchange_rate: parseNumber(item['為替レート［円］']),
        cancellation_unit_price_yen: parseNumber(item['解約単価［円］']),
        cancellation_amount_yen: parseNumber(item['解約額［円］']),
        average_acquisition_price_yen: parseNumber(item['平均取得価額［円］']),
        realized_profit_and_loss: realizedPnL,
        taxes,
        realized_profit_and_loss_after_tax: realizedPnLAfterTax,
    };
};

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
