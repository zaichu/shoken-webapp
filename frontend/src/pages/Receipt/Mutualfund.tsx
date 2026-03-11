import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import { EmptyState } from '@/components/atoms/EmptyState';
import React from 'react';
import { MutualfundData } from '@/lib/interfaces/mutualfund';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { createSearchOptions, groupAndSummarizeData } from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    createYearMonthKey,
    formatCurrency,
    formatNumber
} from '@/lib/utils/formatters';
import { createYearOptions, FilterConfig } from '@/lib/utils/searchUtils';
import { useReceiptCalculations, useReceiptBaseData } from '@/hooks/receipt/useReceiptData';
import { sortMutualfundByTradeDate } from '@/features/receipt/parsers';
import { calculateMutualfund } from '@/features/receipt/calculations';

// コンポーネント外に定数として定義（毎レンダーで新参照が生成されるのを防ぐ）
const FILTER_CONFIG: FilterConfig<MutualfundData> = {
    stringFields: [
        item => item.fund_name,
        item => item.account,
    ],
    dateField: item => item.trade_date,
    yearSearch: true,
};

interface MutualfundProps {
    data: MutualfundData[];
    previewData?: MutualfundData[];
    utilityRail?: React.ReactNode;
}

/**
 * 投資信託データを表示するコンポーネント
 */
export const Mutualfund: React.FC<MutualfundProps> = ({ data, previewData, utilityRail }) => {
    const { sortedData: mutualfundData, searchQuery, setSearchQuery, filteredData } =
        useReceiptBaseData(data, previewData, sortMutualfundByTradeDate, FILTER_CONFIG);

    // 検索オプションの生成
    const searchCategories = {
        securities: createSearchOptions(mutualfundData, '', 'fund_name', true),
        years: createYearOptions(mutualfundData, item => item.trade_date)
    };

    // 表示用の集計（検索前後で同一ロジック: フィルタ後データから計算）
    const calculations = useReceiptCalculations(filteredData, calculateMutualfund);

    // グループキーの取得（日付文字列：年月）
    // ファンド名検索時は元データの正式表記を使用
    const getGroupKey = (item: MutualfundData): string => {
        if (searchQuery) {
            // ファンド名検索の場合は元データの表記を使用（小文字化しない）
            const query = searchQuery.toLowerCase();
            if (item.fund_name.toLowerCase().includes(query)) {
                return item.fund_name;
            }
            // 年検索など他の場合は年月でグループ化
            return createYearMonthKey(item.trade_date);
        }
        return createYearMonthKey(item.trade_date);
    };

    // サマリーデータの集計
    const summary = groupAndSummarizeData(
        filteredData,
        getGroupKey,
        ['cancellation_amount_yen', 'realized_profit_and_loss', 'taxes', 'realized_profit_and_loss_after_tax']
    );

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

    // テーブルカラムの定義（列幅を明示的に設定して右端切れを防止）
    const columns: TableColumnConfig[] = [
        { key: 'trade_date', header: '約定日', width: '84px', format: formatJPDate },
        { key: 'fund_name', header: 'ファンド名', width: '180px' },
        { key: 'account', header: '口座', width: '60px' },
        { key: 'shares', header: '数量', width: '64px', textAlign: 'right', format: formatNumber },
        { key: 'cancellation_unit_price_yen', header: '解約単価', width: '82px', textAlign: 'right', format: formatCurrency },
        { key: 'cancellation_amount_yen', header: '解約額', width: '82px', textAlign: 'right', format: formatCurrency },
        { key: 'average_acquisition_price_yen', header: '取得価額', width: '82px', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss', header: '実現損益', width: '82px', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', width: '64px', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss_after_tax', header: '税引損益', width: '84px', textAlign: 'right', format: formatCurrency },
    ];

    // サマリーカラムの定義（ヘッダーと同じ項目: 実現損益、税額、税引後）
    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'realized_profit_and_loss', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss_after_tax', textAlign: 'right', format: formatCurrency },
    ];

    return (
        <ReceiptTemplate
            title="投資信託"
            header={mutualfundData.length > 0 ? <ReceiptHeader items={headerItems} compact={Boolean(utilityRail)} /> : undefined}
            onSearch={(query: string) => setSearchQuery(query)}
            searchCategories={searchCategories}
            utilityRail={utilityRail}
        >
            {mutualfundData.length === 0 ? (
                <EmptyState
                    title="データがありません"
                    description="投資信託明細をCSVで追加してください"
                />
            ) : (
                <ReceiptTable
                    data={filteredData}
                    summary={summary}
                    columns={columns}
                    summaryColumns={summaryColumns}
                    getGroupKey={getGroupKey}
                />
            )}
        </ReceiptTemplate>
    );
};
