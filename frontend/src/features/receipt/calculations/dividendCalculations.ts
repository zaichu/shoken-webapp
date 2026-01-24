import { DividendData, DividendCalculations } from '@/lib/interfaces/dividend';

/**
 * 配当データの合計を計算
 */
export const calculateDividends = (data: DividendData[]): DividendCalculations => {
    return data.reduce((acc, item) => ({
        total_dividends_before_tax: acc.total_dividends_before_tax + item.dividends_before_tax,
        total_taxes: acc.total_taxes + item.taxes,
        total_net_amount_received: acc.total_net_amount_received + item.net_amount_received,
    }), {
        total_dividends_before_tax: 0,
        total_taxes: 0,
        total_net_amount_received: 0
    });
};
