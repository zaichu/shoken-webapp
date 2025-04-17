import { useMemo } from 'react';
import { DividendItem } from '../features/receipt/types';
import { formatCurrencyString } from '../lib/utils/format';

export function useReceiptCalculation(items: DividendItem[]) {
    const calculations = useMemo(() => {
        const totals = items.reduce((acc, item) => ({
            totalDividends: acc.totalDividends + (item.dividends_before_tax || 0),
            totalTaxes: acc.totalTaxes + (item.taxes || 0),
            totalNetAmount: acc.totalNetAmount + (item.net_amount_received || 0),
        }), {
            totalDividends: 0,
            totalTaxes: 0,
            totalNetAmount: 0,
        });

        // 銘柄ごとの集計
        const bySecurityCode = items.reduce((acc, item) => {
            const code = item.security_code || 'unknown';
            if (!acc[code]) {
                acc[code] = {
                    securityName: item.security_name || '',
                    totalDividends: 0,
                    totalTaxes: 0,
                    totalNetAmount: 0,
                    count: 0,
                };
            }
            acc[code].totalDividends += item.dividends_before_tax || 0;
            acc[code].totalTaxes += item.taxes || 0;
            acc[code].totalNetAmount += item.net_amount_received || 0;
            acc[code].count += 1;
            return acc;
        }, {} as Record<string, {
            securityName: string;
            totalDividends: number;
            totalTaxes: number;
            totalNetAmount: number;
            count: number;
        }>);

        return {
            totals,
            bySecurityCode,
            formattedTotals: {
                totalDividends: formatCurrencyString(totals.totalDividends),
                totalTaxes: formatCurrencyString(totals.totalTaxes),
                totalNetAmount: formatCurrencyString(totals.totalNetAmount),
            },
        };
    }, [items]);

    return calculations;
}