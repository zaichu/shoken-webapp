import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import { EmptyState } from '@/components/atoms/EmptyState';
import React from 'react';
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
import { useReceiptCalculations, useReceiptBaseData } from '@/hooks/receipt/useReceiptData';
import {
    createYearOptions,
    createYearMonthOptions,
    getUniqueValues,
    FilterConfig
} from '@/lib/utils/searchUtils';
import { sortDomesticStockByTradeDate } from '@/features/receipt/parsers';
import { calculateDailyData, calculateDomesticStock } from '@/features/receipt/calculations';
import { reorderColumnsBySearch, ColumnReorderRule } from '@/lib/utils/columnUtils';

// コンポーネント外に定数として定義（毎レンダーで新参照が生成されるのを防ぐ）
const FILTER_CONFIG: FilterConfig<DomesticStockData> = {
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
};

interface DomesticStockProps {
    data: DomesticStockData[];
    previewData?: DomesticStockData[];
    utilityRail?: React.ReactNode;
}

/**
 * 国内株式取引データを表示するコンポーネント
 */
export const DomesticStock: React.FC<DomesticStockProps> = ({ data, previewData, utilityRail }) => {
    const { sortedData: domesticStockData, searchQuery, setSearchQuery, filteredData } =
        useReceiptBaseData(data, previewData, sortDomesticStockByTradeDate, FILTER_CONFIG);

    // 検索カテゴリーの生成
    const searchCategories = {
        securities: createSearchOptions(domesticStockData, 'security_code', 'security_name', true),
        accounts: getUniqueValues(domesticStockData, item => item.account),
        years: createYearOptions(domesticStockData, item => item.trade_date),
        yearMonths: createYearMonthOptions(domesticStockData, item => item.trade_date)
    };

    // 日次集計（searchQuery がある場合はフィルタ後データ、ない場合は全データを使用）
    const filteredDailyData = calculateDailyData(searchQuery ? filteredData : domesticStockData);

    // 表示用の集計（検索前後で同一ロジック: フィルタ後データから計算）
    const calculations = useReceiptCalculations(filteredDailyData, calculateDomesticStock);

    // グループキーの取得
    const getGroupKey = (item: DomesticStockData): string => createISODateKey(item.trade_date);

    // ヘッダー項目の定義
    const headerItems = [
        {
            title: '実現損益',
            value: calculations.total_realized_profit_and_loss,
            format: formatCurrency,
            className: 'bg-emerald-50',
            valueClassName: 'text-emerald-600'
        },
        {
            title: '税額',
            value: calculations.total_taxes,
            format: formatCurrency,
            className: 'bg-red-50',
            valueClassName: 'text-red-500'
        },
        {
            title: '実現損益(税引)',
            value: calculations.total_realized_profit_and_loss_after_tax,
            format: formatCurrency,
            className: 'bg-emerald-50',
            valueClassName: 'text-emerald-600'
        }
    ];

    // テーブルカラムの定義（検索タイプに応じて表示順序を調整）
    // サマリーと一致するよう、最後の3カラムは「実現損益」「税額」「税引後」にする
    const baseColumns: TableColumnConfig[] = [
        { key: 'trade_date', header: '約定日', width: '84px', format: formatJPDate },
        { key: 'security_code', header: '銘柄コード', width: '72px', textAlign: 'center', format: renderSecurityCode },
        { key: 'security_name', header: '銘柄名', width: '156px' },
        { key: 'account', header: '口座', width: '60px' },
        { key: 'shares', header: '数量', width: '56px', textAlign: 'right', format: formatNumber },
        { key: 'asked_price', header: '売却単価', width: '76px', textAlign: 'right', format: formatCurrency },
        { key: 'proceeds', header: '売却額', width: '82px', textAlign: 'right', format: formatCurrency },
        { key: 'purchase_price', header: '取得価額', width: '82px', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss', header: '損益', width: '82px', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', width: '64px', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss_after_tax', header: '税引後', width: '84px', textAlign: 'right', format: formatCurrency },
    ];

    // 列の前面配置ルール（口座のみ）
    const columnRules: ColumnReorderRule<DomesticStockData>[] = [
        { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
    ];

    // 検索タイプに応じて重要なカラムを前面に配置（日付・コードの2列固定）
    const columns = reorderColumnsBySearch(baseColumns, filteredData, searchQuery, columnRules, 2);

    // サマリーカラムの定義（最後の3カラムと一致）
    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'total_realized_profit_and_loss', textAlign: 'right', format: formatCurrency },
        { key: 'total_taxes', textAlign: 'right', format: formatCurrency },
        { key: 'total_realized_profit_and_loss_after_tax', textAlign: 'right', format: formatCurrency },
    ];

    return (
        <ReceiptTemplate
            title="国内株式"
            header={domesticStockData.length > 0 ? <ReceiptHeader items={headerItems} compact={Boolean(utilityRail)} /> : undefined}
            onSearch={(query: string) => setSearchQuery(query)}
            searchCategories={searchCategories}
            layout={utilityRail ? 'workspace' : 'stack'}
            utilityRail={utilityRail}
        >
            {domesticStockData.length === 0 ? (
                <EmptyState
                    title="データがありません"
                    description="国内株式明細をCSVで追加してください"
                />
            ) : (
                <ReceiptTable
                    data={filteredData}
                    summary={filteredDailyData}
                    columns={columns}
                    summaryColumns={summaryColumns}
                    getGroupKey={getGroupKey}
                />
            )}
        </ReceiptTemplate>
    );
};
