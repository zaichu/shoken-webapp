import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { parseNumber, TAX_RATE } from '@/lib/utils/formatters';
import { createSecurityCodeLink } from './securityLink';

/**
 * CSVアイテムをDomesticStockDataに変換
 */
export const parseDomesticStockCsvItem = (item: Record<string, unknown>): DomesticStockData => {
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
