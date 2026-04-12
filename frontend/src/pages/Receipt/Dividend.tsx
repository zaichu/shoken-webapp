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
import { renderSecurityCode } from '@/components/atoms/SecurityCodeLink';
import { useReceiptCalculations, useReceiptBaseData } from '@/hooks/receipt/useReceiptData';
import {
    createYearOptions,
    getUniqueValues,
    matchesYear,
    matchesYearMonth,
    FilterConfig
} from '@/lib/utils/searchUtils';
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
    utilityRail?: React.ReactNode;
}

/**
 * 配当金データを表示するコンポーネント
 */
export const Dividend: React.FC<DividendProps> = ({ data, previewData, utilityRail }) => {
    const { sortedData: dividendData, searchQuery, setSearchQuery, filteredData } =
        useReceiptBaseData(data, previewData, sortDividendBySettlementDate, FILTER_CONFIG);

    // 検索カテゴリーの生成
    const searchCategories = {
        securities: createSearchOptions(dividendData, 'security_code', 'security_name', true),
        products: getUniqueValues(dividendData, item => item.product),
        accounts: getUniqueValues(dividendData, item => item.account),
        years: createYearOptions(dividendData, item => item.settlement_date)
    };

    // 表示用の集計（検索前後で同一ロジック: フィルタ後データから計算）
    const calculations = useReceiptCalculations(filteredData, calculateDividends);

    // グループキーの取得（検索タイプに応じて動的に変更）
    const getGroupKey = (item: DividendData): string => {
        if (!searchQuery) {
            return createYearMonthKey(item.settlement_date);
        }

        const query = searchQuery.toLowerCase();

        // 銘柄での検索の場合（銘柄名でグループ化、年と誤判定を防ぐ）
        if (item.security_code.toLowerCase() === query ||
            item.security_name.toLowerCase() === query) {
            return item.security_name;
        }

        // 商品での検索の場合
        if (item.product.toLowerCase() === query) {
            return item.product;
        }

        // 口座での検索の場合
        if (item.account.toLowerCase() === query) {
            return item.account;
        }

        // 年度での検索の場合も月単位でグループ化（検索なし時と同一ルール）
        if (matchesYear(item.settlement_date, query)) {
            return createYearMonthKey(item.settlement_date);
        }

        // 年月での検索の場合も月単位でグループ化（検索なし時と同一ルール）
        if (matchesYearMonth(item.settlement_date, query)) {
            return createYearMonthKey(item.settlement_date);
        }

        // デフォルトは年月でグループ化
        return createYearMonthKey(item.settlement_date);
    };

    // サマリーデータの集計
    const summary = groupAndSummarizeData(
        filteredData,
        getGroupKey,
        ['dividends_before_tax', 'taxes', 'net_amount_received']
    );

    // 銘柄名検索時に銘柄コードを補完
    const searchSecurityCode = (() => {
        if (!searchQuery) return '';
        const normalizedQuery = searchQuery.toLowerCase();
        const labelMatch = searchQuery.match(/^\s*([0-9A-Za-z]+)\s*[:：]/);
        if (labelMatch) {
            return labelMatch[1];
        }
        const matchedItem = filteredData.find(item =>
            item.security_code.toLowerCase() === normalizedQuery ||
            item.security_name.toLowerCase() === normalizedQuery
        );
        return matchedItem?.security_code || '';
    })();

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
        { key: 'security_name', header: '銘柄名', width: '160px' },
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
                    compact={Boolean(utilityRail)}
                >
                    {isSecurityCodeSearch && (
                        <DividendInfo
                            searchQuery={searchQuery}
                            securityCode={searchSecurityCode}
                            summary={summary}
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
