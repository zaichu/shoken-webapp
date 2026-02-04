import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptHeader } from '@/components/molecules/ReceiptHeader/ReceiptHeader';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import React, { useMemo, useCallback, useState } from 'react';
import {
    MutualfundData,
    MutualfundCalculations
} from '@/lib/interfaces/mutualfund';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { createSearchOptions, groupAndSummarizeData } from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    createYearMonthKey,
    formatCurrency,
    formatNumber
} from '@/lib/utils/formatters';
import { createYearOptions, filterByConfig, FilterConfig } from '@/lib/utils/searchUtils';
import { parseMutualfundCsvItem, sortMutualfundByTradeDate } from '@/features/receipt/parsers';

interface MutualfundProps {
    csvData: Record<string, unknown>[];
}

/**
 * 投資信託データを表示するコンポーネント
 */
export const Mutualfund: React.FC<MutualfundProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');

    /**
     * CSVまたはDBデータをMutualfundData形式に変換
     */
    const mutualfundData: MutualfundData[] = useMemo(() => sortMutualfundByTradeDate(
        csvData.map(parseMutualfundCsvItem)
    ), [csvData]);

    /**
     * 検索オプションの生成
     */
    const searchCategories = useMemo(() => ({
        securities: createSearchOptions(mutualfundData, '', 'fund_name', true),
        years: createYearOptions(mutualfundData, item => item.trade_date)
    }), [mutualfundData]);

    // フィルタ設定
    const filterConfig: FilterConfig<MutualfundData> = useMemo(() => ({
        stringFields: [
            item => item.fund_name,
            item => item.account,
        ],
        dateField: item => item.trade_date,
        yearSearch: true,
    }), []);

    // 検索クエリに基づくフィルタリング
    const filteredData = useMemo(
        () => filterByConfig(mutualfundData, searchQuery, filterConfig),
        [mutualfundData, searchQuery, filterConfig]
    );

    /**
     * 表示用の集計（検索前後で同一ロジック: フィルタ後データから計算）
     */
    const calculations: MutualfundCalculations = useMemo(() => filteredData.reduce((acc, item) => ({
        total_realized_profit_and_loss: acc.total_realized_profit_and_loss + item.realized_profit_and_loss,
        total_taxes: acc.total_taxes + item.taxes,
        total_realized_profit_and_loss_after_tax: acc.total_realized_profit_and_loss_after_tax + item.realized_profit_and_loss_after_tax,
    }), {
        total_realized_profit_and_loss: 0,
        total_taxes: 0,
        total_realized_profit_and_loss_after_tax: 0,
    }), [filteredData]);

    /**
     * グループキーの取得（日付文字列：年月）
     * ファンド名検索時は元データの正式表記を使用
     */
    const getGroupKey = useCallback((item: MutualfundData): string => {
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
    }, [searchQuery]);

    /**
     * サマリーデータの集計
     */
    const summary = useMemo(() => groupAndSummarizeData(
        filteredData,
        getGroupKey,
        ['cancellation_amount_yen', 'realized_profit_and_loss', 'taxes', 'realized_profit_and_loss_after_tax']
    ), [filteredData, getGroupKey]);

    /**
     * ヘッダー項目の定義
     */
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

    /**
     * テーブルカラムの定義
     */
    const columns: TableColumnConfig[] = useMemo(() => ([
        { key: 'trade_date', header: '約定日', format: formatJPDate },
        { key: 'settlement_date', header: '受渡日', format: formatJPDate },
        { key: 'fund_name', header: 'ファンド名', width: '250px' },
        { key: 'account', header: '口座', width: '80px' },
        { key: 'shares', header: '数量[株]', textAlign: 'right', format: formatNumber },
        { key: 'exchange_rate', header: '為替レート', textAlign: 'right', format: formatCurrency },
        { key: 'cancellation_unit_price_yen', header: '解約単価', textAlign: 'right', format: formatCurrency },
        { key: 'cancellation_amount_yen', header: '解約額', textAlign: 'right', format: formatCurrency },
        { key: 'average_acquisition_price_yen', header: '平均取得価額', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss', header: '実現損益', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss_after_tax', header: '実現損益(税引)', textAlign: 'right', format: formatCurrency },
    ]), []);

    /**
     * サマリーカラムの定義（ヘッダーと同じ項目: 実現損益、税額、税引後）
     */
    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'realized_profit_and_loss', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', textAlign: 'right', format: formatCurrency },
        { key: 'realized_profit_and_loss_after_tax', textAlign: 'right', format: formatCurrency },
    ];

    return (
        <ReceiptTemplate
            title="投資信託"
            header={<ReceiptHeader items={headerItems} />}
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
