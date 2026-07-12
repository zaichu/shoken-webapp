import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import { EmptyState } from '@/components/atoms/EmptyState';
import React from 'react';
import { CopyableInstrumentName } from '@/components/atoms/SecurityCodeLink';
import type {
    MutualfundData,
    SummaryColumnConfig,
    TableColumnConfig
} from '@/features/receipt/types';
import type { MutualfundApiSummary } from '@/features/receipt/hooks/useReceiptsData';
import { createSearchOptions, groupAndSummarizeData } from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    createYearMonthKey,
    formatCurrency,
    formatNumber
} from '@/lib/utils/formatters';
import { createYearOptions, FilterConfig } from '@/lib/utils/searchUtils';
import { createGroupKeyFn } from '@/lib/utils/searchGroupKey';
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
    yearMonthSearch: true,
    dateSearch: true,
    dateRangeSearch: true,
};

interface MutualfundProps {
    data: MutualfundData[];
    previewData?: MutualfundData[];
    // 検索条件全体（DB 側）の集計。1000件超のユーザーでも正しい合計を表示するために使用する
    summary?: MutualfundApiSummary;
    utilityRail?: React.ReactNode;
}

/**
 * 投資信託データを表示するコンポーネント
 */
export const Mutualfund: React.FC<MutualfundProps> = ({ data, previewData, summary: apiSummary, utilityRail }) => {
    const { sortedData: mutualfundData, searchQuery, setSearchQuery, filteredData } =
        useReceiptBaseData(data, previewData, sortMutualfundByTradeDate, FILTER_CONFIG);

    // 検索オプションの生成
    const searchCategories = {
        securities: createSearchOptions(mutualfundData, '', 'fund_name', true, 'trade_date'),
        years: createYearOptions(mutualfundData, item => item.trade_date),
        dates: true,
    };

    // 表示用の集計（検索前後で同一ロジック: フィルタ後データから計算）
    const clientCalculations = useReceiptCalculations(filteredData, calculateMutualfund);
    // 未フィルタ時は DB 側の集計（1000件キャップの影響を受けない）を優先する。
    // プレビュー中や絞り込み中は取得済みデータから計算した値を使う。
    const isPreviewMode = Boolean(previewData && previewData.length > 0);
    const calculations = apiSummary && !isPreviewMode && searchQuery === ''
        ? apiSummary
        : clientCalculations;

    // 検索タイプに応じたグループキー関数（ファンド名部分一致でグループ化）
    const getGroupKey = createGroupKeyFn<MutualfundData>(
        searchQuery,
        item => createYearMonthKey(item.trade_date),
        [
            {
                test: (item, t) => item.fund_name.toLowerCase().includes(t),
                keyFn: item => item.fund_name,
            },
        ]
    );

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
            tone: 'emerald' as const,
        },
        {
            title: '税額',
            value: calculations.total_taxes,
            format: formatCurrency,
            tone: 'red' as const,
        },
        {
            title: '実現損益(税引)',
            value: calculations.total_realized_profit_and_loss_after_tax,
            format: formatCurrency,
            tone: 'emerald' as const,
        }
    ];

    // テーブルカラムの定義（列幅を明示的に設定して右端切れを防止）
    const columns: TableColumnConfig[] = [
        { key: 'trade_date', header: '約定日', width: '112px', format: formatJPDate },
        { key: 'fund_name', header: 'ファンド名', width: '300px', render: (value) => <CopyableInstrumentName name={value} /> },
        { key: 'account', header: '口座', width: '60px' },
        { key: 'shares', header: '数量', width: '112px', textAlign: 'right', format: formatNumber },
        { key: 'cancellation_unit_price_yen', header: '解約単価', width: '98px', textAlign: 'right', format: formatCurrency },
        { key: 'cancellation_amount_yen', header: '解約額', width: '128px', textAlign: 'right', format: formatCurrency },
        { key: 'average_acquisition_price_yen', header: '取得価額', width: '116px', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss', header: '実現損益', width: '112px', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', width: '106px', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss_after_tax', header: '税引損益', width: '118px', textAlign: 'right', format: formatCurrency },
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
            header={mutualfundData.length > 0 ? <ReceiptHeader items={headerItems} /> : undefined}
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
