import { ReceiptTemplate } from '@/components/templates';
import { ReceiptHeader } from '@/components/molecules';
import { ReceiptTable } from '@/components/organisms';
import React, { useMemo, useState, useCallback } from 'react';
import {
    DividendData,
    DividendCalculations
} from '@/lib/interfaces/dividend';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import {
    createSearchOptions,
    filterDataBySearchQuery,
    groupAndSummarizeData,
    SummaryResult
} from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    createYearMonthKey,
    formatCurrency,
    formatNumber
} from '@/lib/constants/formats';
import { NumberInputField, StatItem, StatItemWithRate } from '@/components';
import { parseNumber } from '@/lib/utils/number';
import { useReceiptData, useReceiptCalculations } from '@/hooks/receipt/useReceiptData';
import { useJQuantsDividend } from '@/lib/api/jquants';

// CSVアイテムをDividendDataに変換
const parseCsvItem = (item: Record<string, unknown>): DividendData => ({
    settlement_date: new Date(item['入金日'] as string),
    product: String(item['商品'] || ''),
    account: String(item['口座'] || ''),
    security_code: String(item['銘柄コード'] || ''),
    security_name: String(item['銘柄'] || ''),
    unit_price: parseNumber(item['単価[円/現地通貨]']),
    shares: parseNumber(item['数量[株/口]']),
    dividends_before_tax: parseNumber(item['配当・分配金合計（税引前）[円/現地通貨]']),
    taxes: parseNumber(item['税額合計[円/現地通貨]']),
    net_amount_received: parseNumber(item['受取金額[円/現地通貨]']),
});

// 決済日でソート
const sortBySettlementDate = (data: DividendData[]): DividendData[] => {
    return [...data].sort((a, b) =>
        a.settlement_date.getTime() - b.settlement_date.getTime()
    );
};

// 配当計算関数
const calculateDividends = (data: DividendData[]): DividendCalculations => {
    return data.reduce((acc, item) => ({
        total_dividends_before_tax: acc.total_dividends_before_tax + item.dividends_before_tax,
        total_taxes: acc.total_taxes + item.taxes,
        total_net_amount_received: acc.total_net_amount_received + item.net_amount_received,
    }), {
        total_dividends_before_tax: 0,
        total_taxes: 0,
        total_net_amount_received: 0
    });
};

// 配当情報コンポーネント
interface DividendInfoProps {
    searchQuery: string;
    summary: SummaryResult<keyof Pick<DividendData, 'dividends_before_tax' | 'taxes' | 'net_amount_received'>>[];
}

const DividendInfo: React.FC<DividendInfoProps> = React.memo(({ searchQuery, summary }) => {
    const [averageUnitPrice, setAverageUnitPrice] = useState<number | undefined>(undefined);
    const [holdingQuantity, setHoldingQuantity] = useState<number | undefined>(undefined);
    const [dividendPerShare, setDividendPerShare] = useState<number | undefined>(undefined);

    // J-Quants APIから配当情報を取得
    const {
        dividendPerShare: apiDividendPerShare,
        loading: apiLoading,
    } = useJQuantsDividend(searchQuery, !!searchQuery);

    // APIからデータが取得されたら自動設定
    React.useEffect(() => {
        if (searchQuery && apiDividendPerShare > 0) {
            setDividendPerShare(apiDividendPerShare);
        } else if (!searchQuery) {
            setDividendPerShare(undefined);
        }
    }, [apiDividendPerShare, searchQuery]);

    // 各種計算値
    const dividendYield = useMemo(() => {
        if (averageUnitPrice && dividendPerShare) {
            return (dividendPerShare / averageUnitPrice) * 100;
        }
        return 0;
    }, [averageUnitPrice, dividendPerShare]);

    const annualDividendAmount = useMemo(() => {
        return parseNumber(holdingQuantity) * parseNumber(dividendPerShare);
    }, [holdingQuantity, dividendPerShare]);

    const totalInvestment = useMemo(() => {
        return parseNumber(averageUnitPrice) * parseNumber(holdingQuantity);
    }, [averageUnitPrice, holdingQuantity]);

    const dividendReturnRate = useMemo(() => {
        if (totalInvestment > 0 && summary[0]) {
            return (summary[0].net_amount_received / totalInvestment) * 100;
        }
        return 0;
    }, [summary, totalInvestment]);

    if (!searchQuery) {
        return null;
    }

    return (
        <div className="card shadow-sm mt-1">
            <div className="card-header bg-primary text-white">
                <h5 className="mb-0">配当情報</h5>
            </div>
            <div className="card-body">
                <div className="row">
                    <div className='col'>
                        <NumberInputField label="平均取得価格" value={averageUnitPrice} onChange={setAverageUnitPrice} />
                    </div>
                    <div className='col'>
                        <NumberInputField label="保有数量(株)" value={holdingQuantity} onChange={setHoldingQuantity} />
                    </div>
                    <div className='col'>
                        <NumberInputField
                            label="一株配当"
                            value={dividendPerShare}
                            onChange={setDividendPerShare}
                            disabled={apiLoading}
                            placeholder={apiLoading ? "データ取得中..." : ""}
                        />
                    </div>
                </div>

                <div className="row mt-3">
                    <StatItem title="取得総額" value={formatCurrency(totalInvestment)} />
                    <StatItemWithRate title="合計受取金額 (累積利回り)" value={summary[0]?.net_amount_received || 0} rate={dividendReturnRate} format={formatCurrency} />
                    <StatItemWithRate title="年間配当金額 (配当利回り)" value={annualDividendAmount} rate={dividendYield} format={formatCurrency} />
                </div>
            </div>
        </div>
    );
});

DividendInfo.displayName = 'DividendInfo';

interface DividendProps {
    csvData: Record<string, unknown>[];
}

/**
 * 配当金データを表示するコンポーネント
 */
export const Dividend: React.FC<DividendProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');
    const onSearch = useCallback((query: string) => setSearchQuery(query), []);

    // CSVデータを配当データ形式に変換
    const dividendData = useReceiptData(csvData, parseCsvItem, sortBySettlementDate);

    // 全体の集計
    const calculations = useReceiptCalculations(dividendData, calculateDividends);

    // 検索オプションの生成
    const searchOptions = useMemo(() =>
        createSearchOptions(
            dividendData,
            'security_code',
            'security_name',
            true
        ),
        [dividendData]
    );

    // 検索クエリに基づくフィルタリング
    const filteredData = useMemo(() =>
        filterDataBySearchQuery(
            dividendData,
            searchQuery,
            ['security_code', 'security_name']
        ),
        [dividendData, searchQuery]
    );

    // グループキーの取得（日付文字列：年月）
    const getGroupKey = useCallback((item: DividendData): string => {
        if (searchQuery) {
            return searchQuery.toLowerCase();
        }
        return createYearMonthKey(item.settlement_date);
    }, [searchQuery]);

    // サマリーデータの集計
    const summary = useMemo(() =>
        groupAndSummarizeData(
            filteredData,
            getGroupKey,
            ['dividends_before_tax', 'taxes', 'net_amount_received']
        ),
        [filteredData, getGroupKey]
    );

    // ヘッダー項目の定義
    const headerItems = useMemo(() => [
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
    ], [calculations]);

    // テーブルカラムの定義
    const columns = useMemo<TableColumnConfig[]>(() => [
        { key: 'settlement_date', header: '入金日', format: formatJPDate },
        { key: 'product', header: '商品' },
        { key: 'account', header: '口座', width: '100px' },
        { key: 'security_code', header: '銘柄コード' },
        { key: 'security_name', header: '銘柄名', width: '250px' },
        { key: 'unit_price', header: '単価', width: '80px', textAlign: 'right', format: formatCurrency },
        { key: 'shares', header: '数量[株]', width: '100px', textAlign: 'right', format: formatNumber },
        { key: 'dividends_before_tax', header: '配当・分配金', width: '150px', textAlign: 'right', format: formatCurrency },
        { key: 'taxes', header: '税額', width: '100px', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', header: '受取金額', width: '100px', textAlign: 'right', format: formatCurrency },
        { key: 'total_dividends_before_tax', header: '合計配当・分配金' },
        { key: 'total_taxes', header: '合計税額' },
        { key: 'total_net_amount_received', header: '合計受取金額' }
    ], []);

    // サマリーカラムの定義
    const summaryColumns = useMemo<SummaryColumnConfig[]>(() => [
        { key: 'dividends_before_tax', colSpan: columns.length - 2, textAlign: 'right', format: formatCurrency },
        { key: 'taxes', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', textAlign: 'right', format: formatCurrency },
    ], [columns.length]);

    return (
        <ReceiptTemplate
            title="配当金"
            header={searchQuery ? (
                <DividendInfo searchQuery={searchQuery} summary={summary} />
            ) : (
                <ReceiptHeader items={headerItems} />
            )}
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
