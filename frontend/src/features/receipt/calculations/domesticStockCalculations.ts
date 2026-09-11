import {
    DomesticStockCalculations,
    DomesticStockData,
    DomesticStockSummary
} from '@/features/receipt/types';
import { TAX_RATE, createISODateKey } from '@/lib/utils/formatters';

/**
 * 日次データ集計（新しい日付順）
 */
export const calculateDailyData = (domesticStockData: DomesticStockData[]): DomesticStockSummary[] => {
    // データを日付ごとにグループ化
    const dailyGroupMap = new Map<string, DomesticStockData[]>();

    domesticStockData.forEach(item => {
        const dateKey = createISODateKey(item.trade_date);
        if (!dailyGroupMap.has(dateKey)) {
            dailyGroupMap.set(dateKey, []);
        }
        dailyGroupMap.get(dateKey)?.push(item);
    });

    // 日次データの集計
    return Array.from(dailyGroupMap.entries()).map(([dateKey, items]) => {
        // 特定口座とNISA口座の集計を分離
        const dailyTotals = items.reduce((acc, item) => {
            const isSpecificAccount = item.account.includes('特定');
            return {
                specificTotal: acc.specificTotal + (isSpecificAccount ? item.realized_profit_and_loss : 0),
                nisaTotal: acc.nisaTotal + (!isSpecificAccount ? item.realized_profit_and_loss : 0),
                amount: acc.amount + item.proceeds
            };
        }, { specificTotal: 0, nisaTotal: 0, amount: 0 });

        // 実現損益の計算
        const totalRealizedPnL = dailyTotals.specificTotal + dailyTotals.nisaTotal;
        const tax = Math.floor(Math.max(0, dailyTotals.specificTotal) * TAX_RATE);
        const totalRealizedPnLAfterTax = dailyTotals.specificTotal - tax + dailyTotals.nisaTotal;

        return {
            filter: dateKey,
            total_realized_profit_and_loss: totalRealizedPnL,
            total_taxes: tax,
            total_realized_profit_and_loss_after_tax: totalRealizedPnLAfterTax,
        };
    }).sort((a, b) => b.filter.localeCompare(a.filter));
};

/**
 * 全体集計関数
 */
export const calculateDomesticStock = (dailyData: DomesticStockSummary[]): DomesticStockCalculations => {
    return dailyData.reduce((acc, item) => ({
        total_realized_profit_and_loss: acc.total_realized_profit_and_loss + item.total_realized_profit_and_loss,
        total_taxes: acc.total_taxes + item.total_taxes,
        total_realized_profit_and_loss_after_tax: acc.total_realized_profit_and_loss_after_tax + item.total_realized_profit_and_loss_after_tax
    }), {
        total_realized_profit_and_loss: 0,
        total_taxes: 0,
        total_realized_profit_and_loss_after_tax: 0,
    });
};
