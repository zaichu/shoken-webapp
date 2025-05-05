import { ReceiptTemplate } from '@/components/templates';
import { ReceiptHeader } from '@/components/molecules';
import { ReceiptTable } from '@/components/organisms';
import React, { useMemo, useState } from 'react';
import { formatCurrencyString, formatNumber } from '@/lib/utils/format';
import {
    DomesticStockData,
    DomesticStockCalculations,
    DomesticStockSummary
} from '@/lib/interfaces/domesticStock';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { createSearchOptions, filterDataBySearchQuery } from '@/lib/utils/dataTransformer';

const TAX_RATE = 0.20315;

interface DomesticStockProps {
    csvData: any[];
}

export const DomesticStock: React.FC<DomesticStockProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');
    const onSearch = (query: string) => setSearchQuery(query);

    const domesticStockData: DomesticStockData[] = useMemo(() => {
        const parsedData = csvData.map((item) => ({
            trade_date: new Date(item['約定日']),
            settlement_date: new Date(item['受渡日']),
            security_code: String(item['銘柄コード']),
            security_name: String(item['銘柄名']),
            account: String(item['口座']),
            shares: Number(String(item['数量[株]']).replace(/,/g, '')),
            asked_price: Number(String(item['売却/決済単価[円]']).replace(/,/g, '') || 0),
            proceeds: Number(String(item['売却/決済額[円]']).replace(/,/g, '') || 0),
            purchase_price: Number(String(item['平均取得価額[円]']).replace(/,/g, '') || 0),
            realized_profit_and_loss: Number(String(item['実現損益[円]']).replace(/,/g, '') || 0),
        }));

        return [...parsedData].sort((a, b) =>
            a.trade_date.getTime() - b.trade_date.getTime()
        );
    }, [csvData]);

    const dailyGroupMap = useMemo(() => {
        const map = new Map<string, DomesticStockData[]>();
        domesticStockData.forEach(item => {
            const dateKey = item.trade_date.toISOString().split('T')[0];
            if (!map.has(dateKey)) {
                map.set(dateKey, []);
            }
            map.get(dateKey)?.push(item);
        });
        return map;
    }, [domesticStockData]);

    const dailyData = useMemo(() => {
        return Array.from(dailyGroupMap.entries()).map(([dateKey, items]) => {
            const dailyTotals = items.reduce((acc, item) => {
                const isSpecificAccount = item.account.includes('特定');
                return {
                    specificTotal: acc.specificTotal + (isSpecificAccount ? item.realized_profit_and_loss : 0),
                    nisaTotal: acc.nisaTotal + (!isSpecificAccount ? item.realized_profit_and_loss : 0),
                    amount: acc.amount + item.proceeds
                };
            }, { specificTotal: 0, nisaTotal: 0, amount: 0 });

            return {
                filter: dateKey,
                total_realized_profit_and_loss: dailyTotals.specificTotal + dailyTotals.nisaTotal,
                total_taxes: Math.floor(dailyTotals.specificTotal * TAX_RATE),
                total_realized_profit_and_loss_after_tax: Math.floor(dailyTotals.specificTotal * (1.0 - TAX_RATE)) + dailyTotals.nisaTotal,
            } as DomesticStockSummary;
        }).sort((a, b) => a.filter.localeCompare(b.filter));
    }, [dailyGroupMap]);

    const calculations: DomesticStockCalculations = useMemo(() => {
        return dailyData.reduce((acc, item) => {
            return {
                total_realized_profit_and_loss: acc.total_realized_profit_and_loss + item.total_realized_profit_and_loss,
                total_taxes: acc.total_taxes + item.total_taxes,
                total_realized_profit_and_loss_after_tax: acc.total_realized_profit_and_loss_after_tax + item.total_realized_profit_and_loss_after_tax
            };
        }, {
            total_realized_profit_and_loss: 0,
            total_taxes: 0,
            total_realized_profit_and_loss_after_tax: 0,
        });
    }, [dailyData]);

    const searchOptions = useMemo(() =>
        createSearchOptions(
            domesticStockData,
            'security_code',
            'security_name',
            true
        ),
        [domesticStockData]);

    const filteredData = useMemo(() =>
        filterDataBySearchQuery(
            domesticStockData,
            searchQuery,
            ['security_code', 'security_name']
        ),
        [domesticStockData, searchQuery]);

    const getGroupKey = (item: DomesticStockData): string => {
        return item.trade_date.toISOString().split('T')[0];
    };

    const headerItems = [
        {
            title: '合計実現損益',
            value: calculations.total_realized_profit_and_loss,
            format: formatCurrencyString
        },
        {
            title: '合計税額',
            value: calculations.total_taxes,
            format: formatCurrencyString
        },
        {
            title: '合計実現損益(税引)',
            value: calculations.total_realized_profit_and_loss_after_tax,
            format: formatCurrencyString
        }
    ];

    const columns: TableColumnConfig[] = [
        { key: 'trade_date', header: '約定日', format: (date) => date.toLocaleDateString('ja-JP', { year: 'numeric', month: '2-digit', day: '2-digit' }) },
        { key: 'settlement_date', header: '受渡日', format: (date) => date.toLocaleDateString('ja-JP', { year: 'numeric', month: '2-digit', day: '2-digit' }) },
        { key: 'security_code', header: '銘柄コード' },
        { key: 'security_name', header: '銘柄名', width: '250px' },
        { key: 'account', header: '口座', width: '100px' },
        { key: 'shares', header: '数量[株]', textAlign: 'right', format: formatNumber },
        { key: 'asked_price', header: '売却単価', textAlign: 'right', format: formatCurrencyString },
        { key: 'proceeds', header: '売却額', textAlign: 'right', format: formatCurrencyString },
        { key: 'purchase_price', header: '平均取得価額', textAlign: 'right', format: formatCurrencyString },
        { key: 'realized_profit_and_loss', header: '実現損益', textAlign: 'right', format: formatCurrencyString },
        { key: 'total_realized_profit_and_loss', header: '合計実現損益' },
        { key: 'total_taxes', header: '合計税額' },
        { key: 'total_realized_profit_and_loss_after_tax', header: '合計実現損益(税引)' },

    ];

    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'total_realized_profit_and_loss', colSpan: 11, textAlign: 'right', format: formatCurrencyString },
        { key: 'total_taxes', textAlign: 'right', format: formatCurrencyString },
        { key: 'total_realized_profit_and_loss_after_tax', textAlign: 'right', format: formatCurrencyString },
    ];

    return (
        <ReceiptTemplate
            title="国内株式"
            header={<ReceiptHeader items={headerItems} />}
            searchQuery={searchQuery}
            onSearch={onSearch}
            searchOptions={searchOptions}
        >
            <ReceiptTable
                data={filteredData}
                summary={dailyData}
                columns={columns}
                summaryColumns={searchQuery ? [] : summaryColumns}
                getGroupKey={getGroupKey}
            />
        </ReceiptTemplate>
    );
};
