import { MutualfundData, MutualfundCalculations } from '@/lib/interfaces/mutualfund';

/**
 * 投資信託データの合計を計算
 */
export const calculateMutualfund = (data: MutualfundData[]): MutualfundCalculations => {
    return data.reduce((acc, item) => ({
        total_realized_profit_and_loss: acc.total_realized_profit_and_loss + item.realized_profit_and_loss,
        total_taxes: acc.total_taxes + item.taxes,
        total_realized_profit_and_loss_after_tax: acc.total_realized_profit_and_loss_after_tax + item.realized_profit_and_loss_after_tax,
    }), {
        total_realized_profit_and_loss: 0,
        total_taxes: 0,
        total_realized_profit_and_loss_after_tax: 0,
    });
};
