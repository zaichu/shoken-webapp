import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { parseNumber, TAX_RATE } from '@/lib/utils/formatters';
import { createSecurityCodeLink } from './securityLink';

/**
 * データがDB形式かどうかを判定
 */
const isDBFormat = (item: Record<string, unknown>): boolean => {
    return 'trade_date' in item && 'security_code' in item && !('約定日' in item);
};

/**
 * CSVまたはDBアイテムをDomesticStockDataに変換
 */
export const parseDomesticStockCsvItem = (item: Record<string, unknown>): DomesticStockData => {
    // DB形式の場合はそのまま返す（既に変換済み）
    if (isDBFormat(item)) {
        const securityCode = String(item.security_code || '');
        return {
            trade_date: item.trade_date instanceof Date
                ? item.trade_date
                : new Date(item.trade_date as string),
            settlement_date: item.settlement_date instanceof Date
                ? item.settlement_date
                : new Date(item.settlement_date as string),
            security_code: securityCode,
            security_name: String(item.security_name || ''),
            security_info: item.security_info || createSecurityCodeLink(securityCode),
            account: String(item.account || ''),
            shares: Number(item.shares) || 0,
            asked_price: Number(item.asked_price) || 0,
            proceeds: Number(item.proceeds) || 0,
            purchase_price: Number(item.purchase_price) || 0,
            realized_profit_and_loss: Number(item.realized_profit_and_loss) || 0,
            taxes: Number(item.taxes) || 0,
            realized_profit_and_loss_after_tax: Number(item.realized_profit_and_loss_after_tax) || 0,
        };
    }

    // CSV形式の場合
    const account = String(item['口座'] || '');
    const realizedPnL = parseNumber(item['実現損益[円]']);
    // 特定口座の場合のみ税金を計算（利益がある場合のみ）
    const isSpecificAccount = account.includes('特定');
    const taxes = isSpecificAccount ? Math.floor(Math.max(0, realizedPnL) * TAX_RATE) : 0;
    const realizedPnLAfterTax = realizedPnL - taxes;

    const securityCode = String(item['銘柄コード']);
    const securityName = String(item['銘柄名']);

    return {
        trade_date: new Date(item['約定日'] as string),
        settlement_date: new Date(item['受渡日'] as string),
        security_code: securityCode,
        security_name: securityName,
        security_info: createSecurityCodeLink(securityCode),
        account,
        shares: parseNumber(item['数量[株]']),
        asked_price: parseNumber(item['売却/決済単価[円]']),
        proceeds: parseNumber(item['売却/決済額[円]']),
        purchase_price: parseNumber(item['平均取得価額[円]']),
        realized_profit_and_loss: realizedPnL,
        taxes,
        realized_profit_and_loss_after_tax: realizedPnLAfterTax,
    };
};

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
        security_info: createSecurityCodeLink(securityCode),
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
