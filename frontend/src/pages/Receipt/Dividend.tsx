import { ReceiptTemplate } from '@/components/templates';
import { ReceiptHeader } from '@/components/molecules';
import { ReceiptTable } from '@/components/organisms';
import React, { useMemo, useState } from 'react';
import { formatCurrencyString, formatNumber } from '@/lib/utils/format';
import {
    DividendData,
    DividendCalculations,
    DividendSummary
} from '@/lib/interfaces/dividend';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { createSearchOptions, filterDataBySearchQuery, groupAndSummarizeData } from '@/lib/utils/dataTransformer';

interface DividendProps {
    csvData: any[];
}

export const Dividend: React.FC<DividendProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');
    const onSearch = (query: string) => setSearchQuery(query);

    const dividendData = useMemo(() => {
        const rawData = csvData.map((item) => ({
            settlement_date: new Date(item['入金日']),
            product: item['商品'],
            account: item['口座'],
            security_code: item['銘柄コード'],
            security_name: item['銘柄'],
            unit_price: Number(item['単価[円/現地通貨]'].replace(/,/g, '')),
            shares: Number(item['数量[株/口]'].replace(/,/g, '')),
            dividends_before_tax: Number(item['配当・分配金合計（税引前）[円/現地通貨]'].replace(/,/g, '') || 0),
            taxes: Number(item['税額合計[円/現地通貨]'].replace(/,/g, '') || 0),
            net_amount_received: Number(item['受取金額[円/現地通貨]'].replace(/,/g, '') || 0),
        }));

        return [...rawData].sort((a, b) =>
            a.settlement_date.getTime() - b.settlement_date.getTime()
        );
    }, [csvData]);

    const calculations: DividendCalculations = useMemo(() => {
        return dividendData.reduce((acc, item) => ({
            total_dividends_before_tax: acc.total_dividends_before_tax + item.dividends_before_tax,
            total_taxes: acc.total_taxes + item.taxes,
            total_net_amount_received: acc.total_net_amount_received + item.net_amount_received,
        }), {
            total_dividends_before_tax: 0,
            total_taxes: 0,
            total_net_amount_received: 0
        });
    }, [dividendData]);

    const searchOptions = useMemo(() =>
        createSearchOptions(
            dividendData,
            'security_code',
            'security_name',
            true
        ),
        [dividendData]);

    const filteredData = useMemo(() =>
        filterDataBySearchQuery(
            dividendData,
            searchQuery,
            ['security_code', 'security_name']
        ),
        [dividendData, searchQuery]);

    const getGroupKey = (item: DividendData): string => {
        if (searchQuery) {
            return searchQuery.toLowerCase();
        }
        const date = item.settlement_date;
        return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}`;
    };

    const summary = useMemo(() =>
        groupAndSummarizeData(
            filteredData,
            getGroupKey,
            ['dividends_before_tax', 'taxes', 'net_amount_received'],
            searchQuery
        ),
        [filteredData, searchQuery]);

    const headerItems = [
        {
            title: '配当金合計',
            value: calculations.total_dividends_before_tax,
            format: formatCurrencyString
        },
        {
            title: '税額合計',
            value: calculations.total_taxes,
            format: formatCurrencyString
        },
        {
            title: '受取金額合計',
            value: calculations.total_net_amount_received,
            format: formatCurrencyString
        }
    ];

    const columns: TableColumnConfig[] = [
        { key: 'settlement_date', header: '入金日', format: (date) => date.toLocaleDateString('ja-JP', { year: 'numeric', month: '2-digit', day: '2-digit' }) },
        { key: 'product', header: '商品' },
        { key: 'account', header: '口座', width: '100px' },
        { key: 'security_code', header: '銘柄コード' },
        { key: 'security_name', header: '銘柄名', width: '250px' },
        { key: 'unit_price', header: '単価', width: '80px', textAlign: 'right', format: formatCurrencyString },
        { key: 'shares', header: '数量[株]', width: '100px', textAlign: 'right', format: formatNumber },
        { key: 'dividends_before_tax', header: '配当・分配金', width: '150px', textAlign: 'right', format: formatCurrencyString },
        { key: 'taxes', header: '税額', width: '100px', textAlign: 'right', format: formatCurrencyString },
        { key: 'net_amount_received', header: '受取金額', width: '100px', textAlign: 'right', format: formatCurrencyString },
        { key: '', header: '配当・分配金合計' },
        { key: '', header: '税額合計' },
        { key: '', header: '受取金額合計' }
    ];

    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'dividends_before_tax', colSpan: 11, textAlign: 'right', format: formatCurrencyString },
        { key: 'taxes', textAlign: 'right', format: formatCurrencyString },
        { key: 'net_amount_received', textAlign: 'right', format: formatCurrencyString },
    ];

    return (
        <ReceiptTemplate
            title="配当金"
            header={<ReceiptHeader items={headerItems} />}
            searchQuery={searchQuery}
            onSearch={onSearch}
            searchOptions={searchOptions}
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
