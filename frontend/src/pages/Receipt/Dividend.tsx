import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import React, { useMemo, useCallback } from 'react';
import { DividendData } from '@/lib/interfaces/dividend';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
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
import { useReceiptData, useReceiptCalculations } from '@/hooks/receipt/useReceiptData';
import { useReceiptPageState } from '@/hooks/receipt/useReceiptPageState';
import {
    createYearOptions,
    createYearMonthOptions,
    getUniqueValues,
    matchesYear,
    matchesYearMonth,
    FilterConfig
} from '@/lib/utils/searchUtils';
import { DividendInfo } from '@/components/molecules/DividendInfo/DividendInfo';
import { parseDividendCsvItem, sortDividendBySettlementDate } from '@/features/receipt/parsers';
import { calculateDividends } from '@/features/receipt/calculations';
import { reorderColumnsBySearch, ColumnReorderRule } from '@/lib/utils/columnUtils';

interface DividendProps {
    csvData: Record<string, unknown>[];
}

/**
 * 配当金データを表示するコンポーネント
 */
export const Dividend: React.FC<DividendProps> = ({ csvData }) => {

    // CSVデータを配当データ形式に変換
    const dividendData = useReceiptData(csvData, parseDividendCsvItem, sortDividendBySettlementDate);

    // 検索カテゴリーの生成
    const searchCategories = useMemo(() => ({
        securities: createSearchOptions(dividendData, 'security_code', 'security_name', true),
        products: getUniqueValues(dividendData, item => item.product),
        accounts: getUniqueValues(dividendData, item => item.account),
        years: createYearOptions(dividendData, item => item.settlement_date),
        yearMonths: createYearMonthOptions(dividendData, item => item.settlement_date)
    }), [dividendData]);

    // フィルタ設定
    const filterConfig: FilterConfig<DividendData> = useMemo(() => ({
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
    }), []);

    // 検索クエリに基づくフィルタリング
    const { searchQuery, setSearchQuery, filteredData } = useReceiptPageState(dividendData, filterConfig);

    // 表示用の集計（検索前後で同一ロジック: フィルタ後データから計算）
    const calculations = useReceiptCalculations(filteredData, calculateDividends);

    // グループキーの取得（検索タイプに応じて動的に変更）
    const getGroupKey = useCallback((item: DividendData): string => {
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
    }, [searchQuery]);

    // サマリーデータの集計
    const summary = useMemo(() => groupAndSummarizeData(
        filteredData,
        getGroupKey,
        ['dividends_before_tax', 'taxes', 'net_amount_received']
    ), [filteredData, getGroupKey]);

    // 銘柄名検索時に銘柄コードを補完
    const searchSecurityCode = useMemo(() => {
        if (!searchQuery) return '';
        const normalizedQuery = searchQuery.toLowerCase();
        const labelMatch = searchQuery.match(/^\\s*([0-9A-Za-z]+)\\s*[:：]/);
        if (labelMatch) {
            return labelMatch[1];
        }
        const matchedItem = filteredData.find(item =>
            item.security_code.toLowerCase() === normalizedQuery ||
            item.security_name.toLowerCase() === normalizedQuery
        );
        return matchedItem?.security_code || '';
    }, [filteredData, searchQuery]);

    // 銘柄コード検索かどうかを判定（配当シミュレーション表示の条件）
    const isSecurityCodeSearch = useMemo(() => {
        return !!searchSecurityCode && SECURITY_CODE_REGEX.test(searchSecurityCode);
    }, [searchSecurityCode]);

    // ヘッダー項目の定義
    const allHeaderItems = [
        {
            title: '合計配当金',
            value: calculations.total_dividends_before_tax,
            format: formatCurrency
        },
        {
            title: '合計税額',
            value: calculations.total_taxes,
            format: formatCurrency
        },
        {
            title: '合計受取金額',
            value: calculations.total_net_amount_received,
            format: formatCurrency
        }
    ];

    // 銘柄検索時も集計情報は常に表示（詳細は折りたたみ内で確認）
    const headerItems = allHeaderItems;

    // テーブルカラムの定義（検索タイプに応じて表示順序を調整）
    const baseColumns: TableColumnConfig[] = useMemo(() => ([
        { key: 'settlement_date', header: '入金日', width: '90px', format: formatJPDate },
        { key: 'product', header: '商品', width: '80px' },
        { key: 'account', header: '口座', width: '70px' },
        { key: 'security_code', header: '銘柄コード', width: '80px', textAlign: 'center', format: renderSecurityCode },
        { key: 'security_name', header: '銘柄名', width: '200px' },
        { key: 'unit_price', header: '単価', width: '70px', textAlign: 'right', format: formatCurrency },
        { key: 'shares', header: '数量', width: '60px', textAlign: 'right', format: formatNumber },
        { key: 'dividends_before_tax', header: '配当金', width: '90px', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', width: '70px', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', header: '受取額', width: '90px', textAlign: 'right', format: formatCurrency },
    ]), []);

    // 列の前面配置ルール（商品 > 口座の優先順）
    const columnRules: ColumnReorderRule<DividendData>[] = useMemo(() => [
        { columnKey: 'product', match: (item, q) => item.product.toLowerCase().includes(q) },
        { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
    ], []);

    // 検索タイプに応じて重要なカラムを前面に配置
    const columns = useMemo(
        () => reorderColumnsBySearch(baseColumns, filteredData, searchQuery, columnRules, 1),
        [baseColumns, filteredData, searchQuery, columnRules]
    );

    // サマリーカラムの定義
    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'dividends_before_tax', colSpan: columns.length - 2, textAlign: 'right', format: formatCurrency },
        { key: 'taxes', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', textAlign: 'right', format: formatCurrency },
    ];

    return (
        <ReceiptTemplate
            title="配当金"
            header={
                <ReceiptHeader
                    items={headerItems}
                    title={isSecurityCodeSearch ? "集計情報 / 銘柄詳細" : "集計情報"}
                    collapsible={isSecurityCodeSearch}
                    defaultExpanded={!isSecurityCodeSearch}
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
            }
            onSearch={(query: string) => setSearchQuery(query)}
            searchCategories={searchCategories}
        >
            <ReceiptTable
                data={filteredData}
                summary={summary}
                columns={columns}
                summaryColumns={summaryColumns}
                getGroupKey={getGroupKey}
            />
        </ReceiptTemplate>
    );
};
