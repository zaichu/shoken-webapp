import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import React, { useMemo, useCallback, useState } from 'react';
import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { createSearchOptions } from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    formatCurrency,
    formatNumber,
    createISODateKey
} from '@/lib/utils/formatters';
import { renderSecurityCode } from '@/components/atoms/SecurityCodeLink';
import { useReceiptData, useReceiptCalculations } from '@/hooks/receipt/useReceiptData';
import {
    createYearOptions,
    createYearMonthOptions,
    getUniqueValues,
    filterByConfig,
    FilterConfig
} from '@/lib/utils/searchUtils';
import { parseDomesticStockCsvItem, sortDomesticStockByTradeDate } from '@/features/receipt/parsers';
import { calculateDailyData, calculateDomesticStock } from '@/features/receipt/calculations';

interface DomesticStockProps {
    csvData: Record<string, unknown>[];
}

/**
 * 国内株式取引データを表示するコンポーネント
 */
export const DomesticStock: React.FC<DomesticStockProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');

    // CSVデータを国内株式データ形式に変換
    const domesticStockData = useReceiptData(csvData, parseDomesticStockCsvItem, sortDomesticStockByTradeDate);

    // 日次データの集計（全データ）
    const dailyData = useMemo(() => calculateDailyData(domesticStockData), [domesticStockData]);

    // 検索カテゴリーの生成
    const searchCategories = useMemo(() => ({
        securities: createSearchOptions(domesticStockData, 'security_code', 'security_name', true),
        accounts: getUniqueValues(domesticStockData, item => item.account),
        years: createYearOptions(domesticStockData, item => item.trade_date),
        yearMonths: createYearMonthOptions(domesticStockData, item => item.trade_date)
    }), [domesticStockData]);

    // フィルタ設定
    const filterConfig: FilterConfig<DomesticStockData> = useMemo(() => ({
        stringFields: [
            item => item.security_code,
            item => item.security_name,
            item => item.account,
        ],
        dateField: item => item.trade_date,
        yearSearch: true,
        amountFields: [
            item => item.shares,
            item => item.asked_price,
            item => item.proceeds,
            item => item.purchase_price,
            item => item.realized_profit_and_loss,
        ],
    }), []);

    // 検索クエリに基づくフィルタリング
    const filteredData = useMemo(
        () => filterByConfig(domesticStockData, searchQuery, filterConfig),
        [domesticStockData, searchQuery, filterConfig]
    );

    // フィルタ後の日次集計
    const filteredDailyData = useMemo(
        () => (searchQuery ? calculateDailyData(filteredData) : dailyData),
        [dailyData, filteredData, searchQuery]
    );

    // 表示用の集計（検索前後で同一ロジック: フィルタ後データから計算）
    const calculations = useReceiptCalculations(filteredDailyData, calculateDomesticStock);

    // グループキーの取得
    const getGroupKey = useCallback((item: DomesticStockData): string => {
        return createISODateKey(item.trade_date);
    }, []);

    // ヘッダー項目の定義
    const headerItems = [
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
    ];

    // テーブルカラムの定義（検索タイプに応じて表示順序を調整）
    // サマリーと一致するよう、最後の3カラムは「実現損益」「税額」「税引後」にする
    const baseColumns: TableColumnConfig[] = useMemo(() => ([
        { key: 'trade_date', header: '約定日', width: '90px', format: formatJPDate },
        { key: 'security_code', header: '銘柄コード', width: '80px', textAlign: 'center', format: renderSecurityCode },
        { key: 'security_name', header: '銘柄名', width: '180px' },
        { key: 'account', header: '口座', width: '70px' },
        { key: 'shares', header: '数量', width: '60px', textAlign: 'right', format: formatNumber },
        { key: 'asked_price', header: '売却単価', width: '85px', textAlign: 'right', format: formatCurrency },
        { key: 'proceeds', header: '売却額', width: '90px', textAlign: 'right', format: formatCurrency },
        { key: 'purchase_price', header: '取得価額', width: '85px', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss', header: '損益', width: '90px', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', width: '70px', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss_after_tax', header: '税引後', width: '90px', textAlign: 'right', format: formatCurrency },
    ]), []);

    // 検索タイプに応じて重要なカラムを前面に配置
    const columns = useMemo(() => {
        if (!searchQuery) return baseColumns;

        const query = searchQuery.toLowerCase();

        // 口座検索の場合、口座カラムを前面に
        if (filteredData.some(item => item.account.toLowerCase().includes(query))) {
            const accountCol = baseColumns.find(col => col.key === 'account')!;
            const otherCols = baseColumns.filter(col => col.key !== 'account');
            return [baseColumns[0], baseColumns[1], accountCol, ...otherCols.slice(2)];
        }

        return baseColumns;
    }, [baseColumns, filteredData, searchQuery]);

    // サマリーカラムの定義（最後の3カラムと一致）
    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'total_realized_profit_and_loss', textAlign: 'right', format: formatCurrency },
        { key: 'total_taxes', textAlign: 'right', format: formatCurrency },
        { key: 'total_realized_profit_and_loss_after_tax', textAlign: 'right', format: formatCurrency },
    ];

    return (
        <ReceiptTemplate
            title="国内株式"
            header={<ReceiptHeader items={headerItems} />}
            onSearch={(query: string) => setSearchQuery(query)}
            searchCategories={searchCategories}
        >
            <ReceiptTable
                data={filteredData}
                summary={filteredDailyData}
                columns={columns}
                summaryColumns={summaryColumns}
                getGroupKey={getGroupKey}
            />
        </ReceiptTemplate>
    );
};
