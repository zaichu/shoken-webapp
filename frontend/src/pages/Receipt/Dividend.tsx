import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import { EmptyState } from '@/components/atoms/EmptyState';
import React from 'react';
import type {
    DividendData,
    SummaryColumnConfig,
    TableColumnConfig
} from '@/features/receipt/types';
import type { DividendApiSummary } from '@/features/receipt/hooks/useReceiptsData';
import {
    createSearchOptions,
    groupAndSummarizeData
} from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    createYearMonthKey,
    formatCurrency,
    formatNumber,
    SECURITY_CODE_REGEX
} from '@/lib/utils/formatters';
import { CopyableInstrumentName, renderSecurityCode } from '@/components/atoms/SecurityCodeLink';
import { useReceiptCalculations, useReceiptBaseData, useReceiptHeaderSummary } from '@/hooks/receipt/useReceiptData';
import {
    createYearOptions,
    getUniqueValues,
    FilterConfig
} from '@/lib/utils/searchUtils';
import { createGroupKeyFn, deriveSecurityCodeFromQuery } from '@/lib/utils/searchGroupKey';
import { DividendInfo } from '@/components/molecules/DividendInfo/DividendInfo';
import { sortDividendBySettlementDate } from '@/features/receipt/parsers';
import { calculateDividends } from '@/features/receipt/calculations';
import { reorderColumnsBySearch, ColumnReorderRule } from '@/lib/utils/columnUtils';

// コンポーネント外に定数として定義（毎レンダーで新参照が生成されるのを防ぐ）
const FILTER_CONFIG: FilterConfig<DividendData> = {
    stringFields: [
        item => item.security_code,
        item => item.security_name,
        item => item.product,
        item => item.account,
    ],
    dateField: item => item.settlement_date,
    yearSearch: true,
    yearMonthSearch: true,
    dateSearch: true,
    dateRangeSearch: true,
    amountFields: [
        item => item.unit_price,
        item => item.shares,
        item => item.dividends_before_tax,
        item => item.taxes,
        item => item.net_amount_received,
    ],
};

interface DividendProps {
    data: DividendData[];
    previewData?: DividendData[];
    // 検索条件全体（DB 側）の集計。1000件超のユーザーでも正しい合計を表示するために使用する
    summary?: DividendApiSummary;
    utilityRail?: React.ReactNode;
}

/**
 * 配当金データを表示するコンポーネント
 */
export const Dividend: React.FC<DividendProps> = ({ data, previewData, summary: apiSummary, utilityRail }) => {
    const { sortedData: dividendData, searchQuery, setSearchQuery, filteredData } =
        useReceiptBaseData(data, previewData, sortDividendBySettlementDate, FILTER_CONFIG);

    // 検索カテゴリーの生成
    const searchCategories = {
        securities: createSearchOptions(dividendData, 'security_code', 'security_name', true, 'settlement_date'),
        products: getUniqueValues(dividendData, item => item.product),
        accounts: getUniqueValues(dividendData, item => item.account),
        years: createYearOptions(dividendData, item => item.settlement_date),
        dates: true,
    };

    // 表示用の集計（検索前後で同一ロジック: フィルタ後データから計算）
    const clientCalculations = useReceiptCalculations(filteredData, calculateDividends);
    const calculations = useReceiptHeaderSummary(apiSummary, previewData, searchQuery, clientCalculations);

    // 入金日が最新の銘柄名を銘柄コードへマッピング（グループキー用）
    const latestSecurityNameByCode = new Map<string, { name: string; settlementTime: number }>();
    for (const item of dividendData) {
        const current = latestSecurityNameByCode.get(item.security_code);
        const settlementTime = item.settlement_date.getTime();
        if (!current || settlementTime > current.settlementTime) {
            latestSecurityNameByCode.set(item.security_code, {
                name: item.security_name,
                settlementTime,
            });
        }
    }

    // 検索タイプに応じたグループキー関数
    const getGroupKey = createGroupKeyFn<DividendData>(
        searchQuery,
        item => createYearMonthKey(item.settlement_date),
        [
            {
                test: (item, t) =>
                    item.security_code.toLowerCase() === t ||
                    item.security_name.toLowerCase() === t,
                keyFn: item =>
                    latestSecurityNameByCode.get(item.security_code)?.name ?? item.security_name,
            },
            {
                test: (item, t) => item.product.toLowerCase() === t,
                keyFn: item => item.product,
            },
            {
                test: (item, t) => item.account.toLowerCase() === t,
                keyFn: item => item.account,
            },
        ]
    );

    // サマリーデータの集計
    const groupedSummary = groupAndSummarizeData(
        filteredData,
        getGroupKey,
        ['dividends_before_tax', 'taxes', 'net_amount_received']
    );

    // 銘柄名検索時に銘柄コードを補完
    const searchSecurityCode = deriveSecurityCodeFromQuery(searchQuery, filteredData);

    // 銘柄コード検索かどうかを判定（配当シミュレーション表示の条件）
    const isSecurityCodeSearch = !!searchSecurityCode && SECURITY_CODE_REGEX.test(searchSecurityCode);

    // ヘッダー項目の定義
    const allHeaderItems = [
        {
            title: '配当金',
            value: calculations.total_dividends_before_tax,
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
            title: '受取金額',
            value: calculations.total_net_amount_received,
            format: formatCurrency,
            tone: 'emerald' as const,
        }
    ];

    // 銘柄検索時は内部に同等の指標があるため上段の集計は非表示
    const headerItems = isSecurityCodeSearch ? [] : allHeaderItems;

    // テーブルカラムの定義（検索タイプに応じて表示順序を調整）
    const baseColumns: TableColumnConfig[] = [
        { key: 'settlement_date', header: '入金日', width: '84px', format: formatJPDate },
        { key: 'product', header: '商品', width: '64px' },
        { key: 'account', header: '口座', width: '64px' },
        { key: 'security_code', header: '銘柄コード', width: '72px', textAlign: 'center', format: renderSecurityCode },
        { key: 'security_name', header: '銘柄名', width: '160px', render: (value, row) => <CopyableInstrumentName name={value} code={row.security_code} /> },
        { key: 'unit_price', header: '単価', width: '72px', textAlign: 'right', format: formatCurrency },
        { key: 'shares', header: '数量', width: '56px', textAlign: 'right', format: formatNumber },
        { key: 'dividends_before_tax', header: '配当金', width: '84px', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', width: '64px', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', header: '受取額', width: '84px', textAlign: 'right', format: formatCurrency },
    ];

    // 列の前面配置ルール（商品 > 口座の優先順）
    const columnRules: ColumnReorderRule<DividendData>[] = [
        { columnKey: 'product', match: (item, q) => item.product.toLowerCase().includes(q) },
        { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
    ];

    // 検索タイプに応じて重要なカラムを前面に配置
    const columns = reorderColumnsBySearch(baseColumns, filteredData, searchQuery, columnRules, 1);

    // サマリーカラムの定義
    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'dividends_before_tax', colSpan: columns.length - 2, textAlign: 'right', format: formatCurrency },
        { key: 'taxes', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', textAlign: 'right', format: formatCurrency },
    ];

    return (
        <ReceiptTemplate
            title="配当金"
            header={dividendData.length > 0 ? (
                <ReceiptHeader
                    items={headerItems}
                    title="集計情報"
                    collapsible={isSecurityCodeSearch}
                >
                    {isSecurityCodeSearch && (
                        <DividendInfo
                            searchQuery={searchQuery}
                            securityCode={searchSecurityCode}
                            summary={groupedSummary}
                            embedded
                        />
                    )}
                </ReceiptHeader>
            ) : undefined}
            onSearch={(query: string) => setSearchQuery(query)}
            searchCategories={searchCategories}
            utilityRail={utilityRail}
        >
            {dividendData.length === 0 ? (
                <EmptyState
                    title="データがありません"
                    description="配当金明細をCSVで追加してください"
                />
            ) : (
                <ReceiptTable
                    primaryKey="net_amount_received"
                    nameKey="security_name"
                    dateKey="settlement_date"
                    accountKey="account"
                    data={filteredData}
                    summary={groupedSummary}
                    columns={columns}
                    summaryColumns={summaryColumns}
                    getGroupKey={getGroupKey}
                />
            )}
        </ReceiptTemplate>
    );
};
