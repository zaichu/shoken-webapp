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
    groupAndSummarizeData,
    SummaryResult
} from '@/lib/utils/dataTransformer';
import {
    formatJPDate,
    createYearMonthKey,
    formatCurrency,
    formatNumber
} from '@/lib/utils/formatters';
import { NumberInputField } from '@/components/atoms/NumberInputField';
import { StatItem, StatItemWithRate } from '@/components/atoms/StatItem';
import { parseNumber } from '@/lib/utils/formatters';
import { useReceiptData, useReceiptCalculations } from '@/hooks/receipt/useReceiptData';
import { useJQuantsDividend } from '@/features/jquants/hooks/useJQuantsDividend';
import { useAssetBalanceStorage } from '@/hooks/common/useAssetBalanceStorage';

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

    // 保有銘柄データを取得
    const { getAssetBalanceByCode } = useAssetBalanceStorage();

    // J-Quants APIから配当情報を取得
    const {
        dividendPerShare: apiDividendPerShare,
        loading: apiLoading,
    } = useJQuantsDividend(searchQuery, !!searchQuery);

    // searchQueryが変更されたときにstateを初期化し、保有銘柄データがあれば自動入力
    React.useEffect(() => {
        if (searchQuery) {
            const assetBalanceData = getAssetBalanceByCode(searchQuery);
            if (assetBalanceData) {
                setAverageUnitPrice(assetBalanceData.average_purchase_price);
                setHoldingQuantity(assetBalanceData.shares);
            } else {
                setAverageUnitPrice(undefined);
                setHoldingQuantity(undefined);
            }
        } else {
            setAverageUnitPrice(undefined);
            setHoldingQuantity(undefined);
        }
    }, [searchQuery, getAssetBalanceByCode]);

    // APIからデータが取得されたら自動設定
    React.useEffect(() => {
        setDividendPerShare(undefined);
        if (searchQuery && apiDividendPerShare !== undefined && apiDividendPerShare > 0) {
            setDividendPerShare(apiDividendPerShare);
        }
    }, [apiDividendPerShare, searchQuery]);

    // 各種計算値
    const dividendYield = (() => {
        if (averageUnitPrice && dividendPerShare) {
            return (dividendPerShare / averageUnitPrice) * 100;
        }
        return 0;
    })();

    const annualDividendAmount = parseNumber(holdingQuantity) * parseNumber(dividendPerShare);

    const totalInvestment = parseNumber(averageUnitPrice) * parseNumber(holdingQuantity);

    const dividendReturnRate = (() => {
        if (totalInvestment > 0 && summary[0]) {
            return (summary[0].net_amount_received / totalInvestment) * 100;
        }
        return 0;
    })();

    if (!searchQuery) {
        return null;
    }

    const assetBalanceData = getAssetBalanceByCode(searchQuery);

    return (
        <div className="card shadow-sm mt-1">
            <div className="card-header bg-primary text-white d-flex justify-content-between align-items-center">
                <h5 className="mb-0">配当情報</h5>
                {assetBalanceData && (
                    <small className="text-light">
                        保有銘柄データから自動入力
                    </small>
                )}
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

    // CSVデータを配当データ形式に変換
    const dividendData = useReceiptData(csvData, parseCsvItem, sortBySettlementDate);

    // 全体の集計
    const calculations = useReceiptCalculations(dividendData, calculateDividends);

    // 検索カテゴリーの生成
    const searchCategories = {
        // 銘柄（銘柄コード + 銘柄名の形式）
        securities: createSearchOptions(dividendData, 'security_code', 'security_name', true),

        // 商品
        products: [...new Set(dividendData.map(item => item.product))]
            .filter(product => product && product.trim() !== ''),

        // 口座
        accounts: [...new Set(dividendData.map(item => item.account))]
            .filter(account => account && account.trim() !== ''),

        // 年度（昇順）
        years: [...new Set(dividendData.map(item => {
            const year = item.settlement_date.getFullYear().toString()
            const label = `${year}年`;
            return { value: year, label }
        }))].filter((item, index, self) => index === self.findIndex(t => t.value === item.value))
            .sort((a, b) => a.value.localeCompare(b.value)),

        // 年月（昇順）
        yearMonths: [...new Set(dividendData.map(item => {
            const year = item.settlement_date.getFullYear();
            const month = item.settlement_date.getMonth() + 1;
            const value = `${year}-${month.toString().padStart(2, '0')}`;
            const label = `${year}年${month.toString().padStart(2, '0')}月`;
            return { value, label }
        }))].filter((item, index, self) => index === self.findIndex(t => t.value === item.value))
            .sort((a, b) => a.value.localeCompare(b.value))
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
            const year = item.settlement_date.getFullYear().toString();
            if (year === query) {
                return true;
            }

            // 年月での検索（YYYY-MM形式）
            const yearMonth = `${item.settlement_date.getFullYear()}-${(item.settlement_date.getMonth() + 1).toString().padStart(2, '0')}`;
            if (yearMonth === query) {
                return true;
            }

            // 日付での検索（YYYY-MM-DD形式）
            const dateStr = item.settlement_date.toISOString().split('T')[0];
            if (dateStr === query) {
                return true;
            }

            // 金額での検索（部分一致）
            const amounts = [
                item.unit_price.toString(),
                item.shares.toString(),
                item.dividends_before_tax.toString(),
                item.taxes.toString(),
                item.net_amount_received.toString()
            ];

            return amounts.some(amount => amount.includes(query));
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
        const year = item.settlement_date.getFullYear().toString();
        if (year === query) {
            return year;
        }

        // 年月での検索の場合
        const yearMonth = `${item.settlement_date.getFullYear()}-${(item.settlement_date.getMonth() + 1).toString().padStart(2, '0')}`;
        if (yearMonth === query) {
            return yearMonth;
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
