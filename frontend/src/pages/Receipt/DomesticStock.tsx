import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import React, { useMemo, useState, useCallback } from 'react';
import {
    DomesticStockData,
    DomesticStockCalculations,
    DomesticStockSummary
} from '@/lib/interfaces/domesticStock';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { createSearchOptions } from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    TAX_RATE,
    formatCurrency,
    formatNumber,
    createISODateKey
} from '@/lib/utils/formatters';
import { parseNumber } from '@/lib/utils/formatters';
import { useReceiptData, useReceiptCalculations } from '@/hooks/receipt/useReceiptData';

// CSVアイテムをDomesticStockDataに変換
const parseCsvItem = (item: Record<string, unknown>): DomesticStockData => ({
    trade_date: new Date(item['約定日'] as string),
    settlement_date: new Date(item['受渡日'] as string),
    security_code: String(item['銘柄コード']),
    security_name: String(item['銘柄名']),
    account: String(item['口座']),
    shares: parseNumber(item['数量[株]']),
    asked_price: parseNumber(item['売却/決済単価[円]']),
    proceeds: parseNumber(item['売却/決済額[円]']),
    purchase_price: parseNumber(item['平均取得価額[円]']),
    realized_profit_and_loss: parseNumber(item['実現損益[円]']),
});

// 取引日でソート
const sortByTradeDate = (data: DomesticStockData[]): DomesticStockData[] => {
    return [...data].sort((a, b) =>
        a.trade_date.getTime() - b.trade_date.getTime()
    );
};

// カスタムフック: 日次データ集計
const useDailyData = (domesticStockData: DomesticStockData[]): DomesticStockSummary[] => {
    return useMemo(() => {
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
        }).sort((a, b) => a.filter.localeCompare(b.filter));
    }, [domesticStockData]);
};

// 全体集計関数
const calculateDomesticStock = (dailyData: DomesticStockSummary[]): DomesticStockCalculations => {
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

interface DomesticStockProps {
    csvData: Record<string, unknown>[];
}

/**
 * 国内株式取引データを表示するコンポーネント
 */
export const DomesticStock: React.FC<DomesticStockProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');
    const onSearch = useCallback((query: string) => setSearchQuery(query), []);

    // CSVデータを国内株式データ形式に変換
    const domesticStockData = useReceiptData(csvData, parseCsvItem, sortByTradeDate);

    // 日次データの集計
    const dailyData = useDailyData(domesticStockData);

    // 全体の集計
    const calculations = useReceiptCalculations(dailyData, calculateDomesticStock);

    // 検索カテゴリーの生成
    const searchCategories = useMemo(() => {
        // 銘柄（銘柄コード + 銘柄名の形式）
        const securities = createSearchOptions(domesticStockData, 'security_code', 'security_name', true);

        // 口座
        const accounts = [...new Set(domesticStockData.map(item => item.account))]
            .filter(account => account && account.trim() !== '');

        // 年度（昇順）
        const years = [...new Set(domesticStockData.map(item => {
            const year = item.trade_date.getFullYear().toString()
            const label = `${year}年`;
            return { value: year, label }
        }))].filter((item, index, self) => index === self.findIndex(t => t.value === item.value))
            .sort((a, b) => a.value.localeCompare(b.value));

        // 年月（昇順）
        const yearMonths = [...new Set(domesticStockData.map(item => {
            const year = item.trade_date.getFullYear();
            const month = item.trade_date.getMonth() + 1;
            const value = `${year}-${month.toString().padStart(2, '0')}`;
            const label = `${year}年${month.toString().padStart(2, '0')}月`;
            return { value, label }
        }))].filter((item, index, self) => index === self.findIndex(t => t.value === item.value))
            .sort((a, b) => a.value.localeCompare(b.value));

        return {
            securities,
            accounts,
            years,
            yearMonths
        };
    }, [domesticStockData]);

    // 検索クエリに基づくフィルタリング（複数フィールドに対応）
    const filteredData = useMemo(() => {
        if (!searchQuery) return domesticStockData;

        const query = searchQuery.toLowerCase();
        return domesticStockData.filter(item => {
            // 銘柄コード・銘柄名での検索
            if (item.security_code.toLowerCase() === query ||
                item.security_name.toLowerCase() === query) {
                return true;
            }

            // 口座での検索
            if (item.account.toLowerCase() === query) {
                return true;
            }

            // 年度での検索（YYYY形式）
            const year = item.trade_date.getFullYear().toString();
            if (year === query) {
                return true;
            }

            // 金額での検索（部分一致）
            const amounts = [
                item.shares.toString(),
                item.asked_price.toString(),
                item.proceeds.toString(),
                item.purchase_price.toString(),
                item.realized_profit_and_loss.toString()
            ];

            return amounts.some(amount => amount.includes(query));
        });
    }, [domesticStockData, searchQuery]);

    // グループキーの取得（検索タイプに応じて動的に変更）
    const getGroupKey = useCallback((item: DomesticStockData): string => {
        // if (searchQuery) {
        //     return searchQuery.toLowerCase();
        // }
        return createISODateKey(item.trade_date);
    }, [searchQuery]);

    // ヘッダー項目の定義
    const headerItems = useMemo(() => [
        {
            title: '合計実現損益',
            value: calculations.total_realized_profit_and_loss,
            format: formatCurrency
        },
        {
            title: '合計税額',
            value: calculations.total_taxes,
            format: formatCurrency
        },
        {
            title: '合計実現損益(税引)',
            value: calculations.total_realized_profit_and_loss_after_tax,
            format: formatCurrency
        }
    ], [calculations]);

    // テーブルカラムの定義（検索タイプに応じて表示順序を調整）
    const columns = useMemo<TableColumnConfig[]>(() => {
        const baseColumns = [
            { key: 'trade_date', header: '約定日', format: formatJPDate },
            { key: 'settlement_date', header: '受渡日', format: formatJPDate },
            { key: 'security_code', header: '銘柄コード' },
            { key: 'security_name', header: '銘柄名', width: '250px' },
            { key: 'account', header: '口座', width: '100px' },
            { key: 'shares', header: '数量[株]', textAlign: 'right', format: formatNumber },
            { key: 'asked_price', header: '売却単価', textAlign: 'right', format: formatCurrency },
            { key: 'proceeds', header: '売却額', textAlign: 'right', format: formatCurrency },
            { key: 'purchase_price', header: '平均取得価額', textAlign: 'right', format: formatCurrency },
            { key: 'realized_profit_and_loss', header: '実現損益', textAlign: 'right', format: formatCurrency },
            { key: 'total_realized_profit_and_loss', header: '合計実現損益' },
            { key: 'total_taxes', header: '合計税額' },
            { key: 'total_realized_profit_and_loss_after_tax', header: '合計実現損益(税引)' },
        ] as TableColumnConfig[];

        // 検索タイプに応じて重要なカラムを前面に配置
        if (searchQuery) {
            const query = searchQuery.toLowerCase();

            // 口座検索の場合、口座カラムを前面に
            if (filteredData.some(item => item.account.toLowerCase().includes(query))) {
                const accountCol = baseColumns.find(col => col.key === 'account')!;
                const otherCols = baseColumns.filter(col => col.key !== 'account');
                return [baseColumns[0], baseColumns[1], accountCol, ...otherCols.slice(2)];
            }
        }

        return baseColumns;
    }, [searchQuery, filteredData]);

    // サマリーカラムの定義
    const summaryColumns = useMemo<SummaryColumnConfig[]>(() => [
        { key: 'total_realized_profit_and_loss', colSpan: columns.length - 2, textAlign: 'right', format: formatCurrency },
        { key: 'total_taxes', textAlign: 'right', format: formatCurrency },
        { key: 'total_realized_profit_and_loss_after_tax', textAlign: 'right', format: formatCurrency },
    ], [columns.length]);

    return (
        <ReceiptTemplate
            title="国内株式"
            header={<ReceiptHeader items={headerItems} />}
            onSearch={onSearch}
            searchCategories={searchCategories}
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
