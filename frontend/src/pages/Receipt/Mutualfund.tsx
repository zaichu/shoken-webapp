import { ReceiptTemplate } from '@/components/templates';
import { ReceiptHeader } from '@/components/molecules';
import { ReceiptTable } from '@/components/organisms';
import React, { useMemo, useState, useCallback } from 'react';
import { formatCurrencyString, formatNumber } from '@/lib/utils/format';
import {
    MutualfundData,
    MutualfundCalculations
} from '@/lib/interfaces/mutualfund';
import { TableColumnConfig } from '@/lib/interfaces/receipt';
import { createSearchOptions, filterDataBySearchQuery, groupAndSummarizeData } from '@/lib/utils/dataTransformer';

const TAX_RATE = 0.20315;

/**
 * 日本の日付フォーマット用のオプション
 */
const JP_DATE_FORMAT_OPTIONS = {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
} as const;

interface MutualfundProps {
    csvData: any[];
}

/**
 * 投資信託データを表示するコンポーネント
 */
export const Mutualfund: React.FC<MutualfundProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');
    const onSearch = useCallback((query: string) => setSearchQuery(query), []);

    /**
     * CSVデータをMutualfundData形式に変換
     */
    const mutualfundData = useMemo<MutualfundData[]>(() => {
        // ,平均取得価額［円］,実現損益［円］
        const parsedData = csvData.map((item) => ({
            trade_date: new Date(item['約定日'] || ''),
            settlement_date: new Date(item['受渡日'] || ''),
            fund_name: String(item['ファンド名'] || ''),
            dividends: String(item['分配金'] || ''),
            account: String(item['口座'] || ''),
            shares: Number(String(item['数量[口]'] || '0').replace(/,/g, '')),
            exchange_rate: Number(String(item['為替レート［円］'] || '0').replace(/,/g, '')),
            cancellation_unit_price_yen: Number(String(item['解約単価［円］'] || '0').replace(/,/g, '')),
            cancellation_amount_yen: Number(String(item['解約額［円］'] || '0').replace(/,/g, '')),
            average_acquisition_price_yen: Number(String(item['平均取得価額［円］'] || '0').replace(/,/g, '')),
            realized_profit_and_loss: Number(String(item['実現損益［円］'] || '0').replace(/,/g, '')),
            taxes: String(item['口座'] || '').includes('特定') ? Number(String(item['実現損益［円］'] || '0').replace(/,/g, '')) * TAX_RATE : 0,
            realized_profit_and_loss_after_tax: Number(String(item['実現損益［円］'] || '0').replace(/,/g, '')) * (String(item['口座'] || '').includes('特定') ? (1 - TAX_RATE) : 1),
        }));

        return [...parsedData].sort((a, b) =>
            a.trade_date.getTime() - b.trade_date.getTime()
        );
    }, [csvData]);

    /**
     * 全体の集計
     */
    const calculations = useMemo<MutualfundCalculations>(() => {
        return mutualfundData.reduce((acc, item) => ({
            total_realized_profit_and_loss: acc.total_realized_profit_and_loss + item.realized_profit_and_loss,
            total_taxes: acc.total_taxes + item.taxes,
            total_realized_profit_and_loss_after_tax: acc.total_realized_profit_and_loss_after_tax + item.realized_profit_and_loss_after_tax,

        }), {
            total_realized_profit_and_loss: 0,
            total_taxes: 0,
            total_realized_profit_and_loss_after_tax: 0,
        });
    }, [mutualfundData]);

    /**
     * 検索オプションの生成
     */
    const searchOptions = useMemo(() =>
        createSearchOptions(
            mutualfundData,
            'fund_name',
            'fund_name',
            false
        ),
        [mutualfundData]);

    /**
     * 検索クエリに基づくフィルタリング
     */
    const filteredData = useMemo(() =>
        filterDataBySearchQuery(
            mutualfundData,
            searchQuery,
            ['fund_name']
        ),
        [mutualfundData, searchQuery]);

    /**
     * グループキーの取得（日付文字列：年月）
     */
    const getGroupKey = useCallback((item: MutualfundData): string => {
        if (searchQuery) {
            return searchQuery.toLowerCase();
        }
        const date = item.trade_date;
        return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}`;
    }, [searchQuery]);

    /**
     * サマリーデータの集計
     */
    const summary = useMemo(() =>
        groupAndSummarizeData(
            filteredData,
            getGroupKey,
            ['cancellation_amount_yen', 'realized_profit_and_loss', 'taxes', 'realized_profit_and_loss_after_tax'],
            searchQuery
        ),
        [filteredData, getGroupKey, searchQuery]);

    /**
     * ヘッダー項目の定義
     */
    const headerItems = useMemo(() => [
        {
            title: '合計実現損益',
            value: calculations.total_realized_profit_and_loss,
            format: formatCurrencyString
        },
        {
            title: '合計税額',
            value: calculations.total_taxes,
            format: formatCurrencyString
        },
        {
            title: '合計実現損益(税引)',
            value: calculations.total_realized_profit_and_loss_after_tax,
            format: formatCurrencyString
        }
    ], [calculations]);

    /**
     * テーブルカラムの定義
     */
    const columns = useMemo<TableColumnConfig[]>(() => [
        {
            key: 'trade_date',
            header: '約定日',
            format: (date) => date.toLocaleDateString('ja-JP', JP_DATE_FORMAT_OPTIONS)
        },
        {
            key: 'settlement_date',
            header: '受渡日',
            format: (date) => date.toLocaleDateString('ja-JP', JP_DATE_FORMAT_OPTIONS)
        },
        { key: 'fund_name', header: 'ファンド名', width: '250px' },
        { key: 'account', header: '口座', width: '80px' },
        { key: 'shares', header: '数量[株]', textAlign: 'right', format: formatNumber },
        { key: 'exchange_rate', header: '為替レート', textAlign: 'right', format: formatNumber },
        { key: 'cancellation_unit_price_yen', header: '解約単価', textAlign: 'right', format: formatCurrencyString },
        { key: 'cancellation_amount_yen', header: '解約額', textAlign: 'right', format: formatCurrencyString },
        { key: 'average_acquisition_price_yen', header: '平均取得価額', textAlign: 'right', format: formatCurrencyString },
        { key: 'realized_profit_and_loss', header: '実現損益', textAlign: 'right', format: formatCurrencyString },
        { key: 'taxes', header: '税額', textAlign: 'right', format: formatCurrencyString },
        { key: 'realized_profit_and_loss_after_tax', header: '実現損益(税引)', textAlign: 'right', format: formatCurrencyString },
    ], []);

    return (
        <ReceiptTemplate
            title="投資信託"
            header={<ReceiptHeader items={headerItems} />}
            searchQuery={searchQuery}
            onSearch={onSearch}
            searchOptions={searchOptions}
        >
            <ReceiptTable
                data={filteredData}
                summary={summary}
                columns={columns}
                summaryColumns={[]}
                getGroupKey={getGroupKey}
            />
        </ReceiptTemplate>
    );
};
