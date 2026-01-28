import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import React, { useState } from 'react';
import {
    DividendData,
    DividendCalculations
} from '@/lib/interfaces/dividend';
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
    createSecurityCodeLink
} from '@/lib/utils/formatters';
import { useReceiptData, useReceiptCalculations } from '@/hooks/receipt/useReceiptData';
import {
    createYearOptions,
    createYearMonthOptions,
    getUniqueValues,
    matchesYear,
    matchesYearMonth,
    matchesDate,
    matchesAmounts
} from '@/lib/utils/searchUtils';
import { DividendInfo } from '@/components/molecules/DividendInfo/DividendInfo';
import { parseDividendCsvItem, sortDividendBySettlementDate } from '@/features/receipt/parsers';
import { calculateDividends } from '@/features/receipt/calculations';

interface DividendProps {
    csvData: Record<string, unknown>[];
}

/**
 * 配当金データを表示するコンポーネント
 */
export const Dividend: React.FC<DividendProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');

    // CSVデータを配当データ形式に変換
    const dividendData = useReceiptData(csvData, parseDividendCsvItem, sortDividendBySettlementDate);

    // 全体の集計
    const calculations = useReceiptCalculations(dividendData, calculateDividends);

    // 検索カテゴリーの生成
    const searchCategories = {
        securities: createSearchOptions(dividendData, 'security_code', 'security_name', true),
        products: getUniqueValues(dividendData, item => item.product),
        accounts: getUniqueValues(dividendData, item => item.account),
        years: createYearOptions(dividendData, item => item.settlement_date),
        yearMonths: createYearMonthOptions(dividendData, item => item.settlement_date)
    };

    // 検索クエリに基づくフィルタリング（複数フィールドに対応）
    const filteredData = !searchQuery ? dividendData : (() => {
        const query = searchQuery.toLowerCase();
        return dividendData.filter(item => {
            // 銘柄コード・銘柄名での検索
            if (item.security_code.toLowerCase() === query ||
                item.security_name.toLowerCase() === query) {
                return true;
            }

            // 商品での検索
            if (item.product.toLowerCase() === query) {
                return true;
            }

            // 口座での検索
            if (item.account.toLowerCase() === query) {
                return true;
            }

            // 年度での検索（YYYY形式）
            if (matchesYear(item.settlement_date, query)) {
                return true;
            }

            // 年月での検索（YYYY-MM形式）
            if (matchesYearMonth(item.settlement_date, query)) {
                return true;
            }

            // 日付での検索（YYYY-MM-DD形式）
            if (matchesDate(item.settlement_date, query)) {
                return true;
            }

            // 金額での検索（部分一致）
            return matchesAmounts([
                item.unit_price,
                item.shares,
                item.dividends_before_tax,
                item.taxes,
                item.net_amount_received
            ], query);
        });
    })();

    // グループキーの取得（検索タイプに応じて動的に変更）
    const getGroupKey = (item: DividendData): string => {
        if (!searchQuery) {
            return createYearMonthKey(item.settlement_date);
        }

        const query = searchQuery.toLowerCase();

        // 銘柄での検索の場合
        if (item.security_code.toLowerCase() === query ||
            item.security_name.toLowerCase() === query) {
            return item.security_code;
        }

        // 商品での検索の場合
        if (item.product.toLowerCase() === query) {
            return item.product;
        }

        // 口座での検索の場合
        if (item.account.toLowerCase() === query) {
            return item.account;
        }

        // 年度での検索の場合
        if (matchesYear(item.settlement_date, query)) {
            return item.settlement_date.getFullYear().toString();
        }

        // 年月での検索の場合
        if (matchesYearMonth(item.settlement_date, query)) {
            return query;
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

    // ヘッダー項目の定義
    const headerItems = [
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

    // テーブルカラムの定義（検索タイプに応じて表示順序を調整）
    const baseColumns: TableColumnConfig[] = [
        { key: 'settlement_date', header: '入金日', width: '90px', format: formatJPDate },
        { key: 'product', header: '商品', width: '80px' },
        { key: 'account', header: '口座', width: '70px' },
        { key: 'security_code', header: '銘柄コード', width: '80px', textAlign: 'center', format: createSecurityCodeLink },
        { key: 'security_name', header: '銘柄名', width: '200px' },
        { key: 'unit_price', header: '単価', width: '70px', textAlign: 'right', format: formatCurrency },
        { key: 'shares', header: '数量', width: '60px', textAlign: 'right', format: formatNumber },
        { key: 'dividends_before_tax', header: '配当金', width: '90px', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', width: '70px', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', header: '受取額', width: '90px', textAlign: 'right', format: formatCurrency },
    ];

    // 検索タイプに応じて重要なカラムを前面に配置
    const columns = !searchQuery ? baseColumns : (() => {
        const query = searchQuery.toLowerCase();

        // 商品検索の場合、商品カラムを前面に
        if (filteredData.some(item => item.product.toLowerCase().includes(query))) {
            const productCol = baseColumns.find(col => col.key === 'product')!;
            const otherCols = baseColumns.filter(col => col.key !== 'product');
            return [baseColumns[0], productCol, ...otherCols.slice(1)];
        }

        // 口座検索の場合、口座カラムを前面に
        if (filteredData.some(item => item.account.toLowerCase().includes(query))) {
            const accountCol = baseColumns.find(col => col.key === 'account')!;
            const otherCols = baseColumns.filter(col => col.key !== 'account');
            return [baseColumns[0], accountCol, ...otherCols.slice(1)];
        }

        return baseColumns;
    })();

    // サマリーカラムの定義
    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'dividends_before_tax', colSpan: columns.length - 2, textAlign: 'right', format: formatCurrency },
        { key: 'taxes', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', textAlign: 'right', format: formatCurrency },
    ];

    return (
        <ReceiptTemplate
            title="配当金"
            header={searchQuery ? (
                <DividendInfo searchQuery={searchQuery} summary={summary} />
            ) : (
                <ReceiptHeader items={headerItems} />
            )}
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
