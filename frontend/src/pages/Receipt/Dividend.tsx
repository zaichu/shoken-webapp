import { ReceiptTemplate } from '@/components/templates';
import { CSVFileInput, ReceiptHeader } from '@/components/molecules';
import { ReceiptTable } from '@/components/organisms';
import React, { useMemo, useState, useCallback } from 'react';
import {
    DividendData,
    DividendCalculations
} from '@/lib/interfaces/dividend';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { createSearchOptions, filterDataBySearchQuery, groupAndSummarizeData } from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    createYearMonthKey,
    formatCurrency,
    formatNumber
} from '@/lib/constants/formats';
import { InputField } from '@/components';
import { useCSVReader } from '@/hooks/useCSVReader';

interface DividendProps {
    csvData: Record<string, unknown>[];
}

/**
 * 配当金データを表示するコンポーネント
 */
export const Dividend: React.FC<DividendProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');
    const onSearch = useCallback((query: string) => setSearchQuery(query), []);

    /**
     * CSVデータをDividendData形式に変換
     */
    const dividendData = useMemo<DividendData[]>(() => {
        const parsedData = csvData.map((item) => ({
            settlement_date: new Date(item['入金日'] as string),
            product: String(item['商品'] || ''),
            account: String(item['口座'] || ''),
            security_code: String(item['銘柄コード'] || ''),
            security_name: String(item['銘柄'] || ''),
            unit_price: Number(String(item['単価[円/現地通貨]'] || '0').replace(/,/g, '')),
            shares: Number(String(item['数量[株/口]'] || '0').replace(/,/g, '')),
            dividends_before_tax: Number(String(item['配当・分配金合計（税引前）[円/現地通貨]'] || '0').replace(/,/g, '')),
            taxes: Number(String(item['税額合計[円/現地通貨]'] || '0').replace(/,/g, '')),
            net_amount_received: Number(String(item['受取金額[円/現地通貨]'] || '0').replace(/,/g, '')),
        }));

        return [...parsedData].sort((a, b) =>
            a.settlement_date.getTime() - b.settlement_date.getTime()
        );
    }, [csvData]);

    /**
     * 全体の集計
     */
    const calculations = useMemo<DividendCalculations>(() => {
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

    /**
     * 検索オプションの生成
     */
    const searchOptions = useMemo(() =>
        createSearchOptions(
            dividendData,
            'security_code',
            'security_name',
            true
        ),
        [dividendData]);

    /**
     * 検索クエリに基づくフィルタリング
     */
    const filteredData = useMemo(() =>
        filterDataBySearchQuery(
            dividendData,
            searchQuery,
            ['security_code', 'security_name']
        ),
        [dividendData, searchQuery]);

    /**
     * グループキーの取得（日付文字列：年月）
     */
    const getGroupKey = useCallback((item: DividendData): string => {
        if (searchQuery) {
            return searchQuery.toLowerCase();
        }
        return createYearMonthKey(item.settlement_date);
    }, [searchQuery]);

    /**
     * サマリーデータの集計
     */
    const summary = useMemo(() =>
        groupAndSummarizeData(
            filteredData,
            getGroupKey,
            ['dividends_before_tax', 'taxes', 'net_amount_received']
        ),
        [filteredData, getGroupKey]);

    /**
     * ヘッダー項目の定義
     */
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

    /**
     * テーブルカラムの定義
     */
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

    /**
     * サマリーカラムの定義
     */
    const summaryColumns = useMemo<SummaryColumnConfig[]>(() => [
        { key: 'dividends_before_tax', colSpan: columns.length - 2, textAlign: 'right', format: formatCurrency },
        { key: 'taxes', textAlign: 'right', format: formatCurrency },
        { key: 'net_amount_received', textAlign: 'right', format: formatCurrency },
    ], [columns.length]);

    const [assetbalance, setAssetbalanceCsvData] = useState<Record<string, unknown>[]>([]);
    const assetbalanceCSV = useCSVReader();
    const { isLoading, error, fileName } = assetbalanceCSV;
    const handleFileSelect = async (file: File) => {
        try {
            setAssetbalanceCsvData(await assetbalanceCSV.parseCSV(file));
            if (assetbalanceCSV.error) assetbalanceCSV.resetError();
        } catch (e) {
            console.error('CSV処理エラー:', e);
        }
    };
    const [averageUnitPrice, setAverageUnitPrice] = useState<number>(0); // 平均取得単価
    const [holdingQuantity, setHoldingQuantity] = useState<number>(0);   // 保有数量(株)
    const [dividendPerShare, setDividendPerShare] = useState<number>(0); // 一株配当
    // 配当利回りの計算
    const dividendYield = useMemo(() => {
        if (averageUnitPrice && dividendPerShare) {
            return (dividendPerShare / averageUnitPrice) * 100;
        }
        return 0;
    }, [averageUnitPrice, dividendPerShare]);

    // 年間配当金額の計算
    const annualDividendAmount = useMemo(() => {
        return holdingQuantity * dividendPerShare;
    }, [holdingQuantity, dividendPerShare]);

    // 投資金額の計算
    const totalInvestment = useMemo(() => {
        return averageUnitPrice * holdingQuantity;
    }, [averageUnitPrice, holdingQuantity]);

    // 投資リターン率の計算
    const dividendReturnRate = useMemo(() => {
        if (totalInvestment > 0) {
            return (summary[0]["net_amount_received"] / totalInvestment) * 100;
        }
        return 0;
    }, [summary, totalInvestment]);

    const AnalyzeDividend = useMemo(() => {
        if (!searchQuery) {
            return;
        }

        return (
            <div className="card shadow-sm mt-1">
                <div className="card-header bg-primary text-white">
                    <h5 className="mb-0">配当情報</h5>
                </div>
                <div className="card-body">
                    {/* <CSVFileInput onFileSelect={handleFileSelect} selectedFileName={fileName} /> */}
                    {error && (
                        <div className="alert alert-danger my-3" role="alert">
                            <strong>エラー:</strong> {error}
                        </div>
                    )}

                    {isLoading && (
                        <div className="text-center my-4">
                            <div className="spinner-border text-primary" role="status">
                                <span className="visually-hidden">Loading...</span>
                            </div>
                        </div>
                    )}

                    <div className="row">
                        <div className='col'>
                            <InputField
                                label="平均取得価格"
                                type="number"
                                className="form-control-plaintext border"
                                value={averageUnitPrice}
                                onChange={(e) => setAverageUnitPrice(Number(e.target.value))}
                            />
                        </div>
                        <div className='col'>
                            <InputField
                                label="保有数量(株)"
                                type="number"
                                className="form-control-plaintext border"
                                value={holdingQuantity}
                                onChange={(e) => setHoldingQuantity(Number(e.target.value))}
                            />
                        </div>
                        <div className='col'>
                            <InputField
                                label="一株配当"
                                type="number"
                                className="form-control-plaintext border"
                                value={dividendPerShare}
                                onChange={(e) => setDividendPerShare(Number(e.target.value))}
                            />
                        </div>
                    </div>

                    <div className="row mt-3">
                        <div className="col">
                            <h6 className='mb-0'>取得総額</h6>
                            <h4 className='mb-0'>{formatCurrency(totalInvestment)}</h4>
                        </div>
                        <div className="col">
                            <h6 className='mb-0'>合計受取金額 (累積利回り)</h6>
                            <h4 className='mb-0'>{formatCurrency(summary[0]['net_amount_received'])} ({dividendReturnRate.toFixed(2)}%)</h4>
                        </div>
                        <div className="col">
                            <h6 className='mb-0'>年間配当金額 (配当利回り)</h6>
                            <h4 className='mb-0'>{formatCurrency(annualDividendAmount)} ({dividendYield.toFixed(2)}%)</h4>
                        </div>
                    </div>
                </div>
            </div >
        );
    }, [searchQuery, averageUnitPrice, holdingQuantity, dividendPerShare, calculations.total_net_amount_received, totalInvestment, dividendYield]);

    return (
        <ReceiptTemplate
            title="配当金"
            header={AnalyzeDividend ?? <ReceiptHeader items={headerItems} />}
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
