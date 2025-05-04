import { ReceiptTemplate } from '@/components';
import React, { useMemo } from 'react';
import { formatCurrencyString, formatNumber, formatDateString } from '@/lib/utils/format';

interface DividendData {
    入金日: string;
    商品: string;
    口座: string;
    銘柄コード: string;
    銘柄: string;
    '単価[円/現地通貨]': number;
    '数量[株/口]': number;
    '配当・分配金合計（税引前）[円/現地通貨]': string;
    '税額合計[円/現地通貨]': string;
    '受取金額[円/現地通貨]': string;
}

interface DividendProps {
    csvData: DividendData[];
}

interface Calculations {
    totalDividends: number;
    totalTaxes: number;
    totalNetAmount: number;
}

const Header: React.FC<{ calculations: Calculations }> = ({ calculations }) => {

    return (
        <div className="card mt-2">
            <div className="card-header bg-primary text-white">
                <h5 className="mb-0">集計情報</h5>
            </div>
            <div className="card-body">
                <div className="row">
                    <div className="col-md">
                        <h6>配当金合計</h6>
                        <p className="h4">{formatCurrencyString(calculations.totalDividends)}</p>
                    </div>
                    <div className="col-md">
                        <h6>税額合計</h6>
                        <p className="h4">{formatCurrencyString(calculations.totalTaxes)}</p>
                    </div>
                    <div className="col-md">
                        <h6>受取金額合計</h6>
                        <p className="h4">{formatCurrencyString(calculations.totalNetAmount)}</p>
                    </div>
                </div>
            </div>
        </div>
    )
}

export const Dividend: React.FC<DividendProps> = ({ csvData }) => {
    csvData = useMemo(() => {
        return [...csvData].sort((a, b) => {
            const dateA = new Date(a['入金日'].replace(/(\d+)\/(\d+)\/(\d+)/, '$1-$2-$3'));
            const dateB = new Date(b['入金日'].replace(/(\d+)\/(\d+)\/(\d+)/, '$1-$2-$3'));
            return dateA.getTime() - dateB.getTime();
        });
    }, [csvData]);

    const calculations = useMemo(() => {
        return csvData.reduce((acc, item) => {
            return {
                totalDividends: acc.totalDividends + Number(item['配当・分配金合計（税引前）[円/現地通貨]'].replace(/,/g, '') || 0),
                totalTaxes: acc.totalTaxes + Number(item['税額合計[円/現地通貨]'].replace(/,/g, '') || 0),
                totalNetAmount: acc.totalNetAmount + Number(item['受取金額[円/現地通貨]'].replace(/,/g, '') || 0)
            };
        }, {
            totalDividends: 0,
            totalTaxes: 0,
            totalNetAmount: 0
        });
    }, [csvData]);

    return (
        <ReceiptTemplate title="配当金" header={<Header calculations={calculations} />}>
            <div className="table-responsive" style={{ maxHeight: '500px', overflowY: 'auto' }}>
                <table className="table table-striped mb-0">
                    <thead className="bg-light sticky-top">
                        <tr>
                            <th scope="col">入金日</th>
                            <th scope="col">商品</th>
                            <th scope="col">口座</th>
                            <th scope="col">銘柄コード</th>
                            <th scope="col">銘柄名</th>
                            <th scope="col">単価</th>
                            <th scope="col">数量[株]</th>
                            <th scope="col">配当・分配金</th>
                            <th scope="col">税額</th>
                            <th scope="col">受取金額</th>
                            <th scope="col">配当・分配金合計</th>
                            <th scope="col">税額合計</th>
                            <th scope="col">受取金額合計</th>
                        </tr>
                    </thead>
                    <tbody>
                        {csvData.map((item, index) => (
                            <tr key={index}>
                                <td>{item['入金日']}</td>
                                <td>{item['商品']}</td>
                                <td>{item['口座']}</td>
                                <td>{item['銘柄コード']}</td>
                                <td>{item['銘柄']}</td>
                                <td>{formatCurrencyString(item['単価[円/現地通貨]'])}</td>
                                <td>{formatNumber(item['数量[株/口]'])}</td>
                                <td>{formatCurrencyString(item['配当・分配金合計（税引前）[円/現地通貨]'])}</td>
                                <td>{formatCurrencyString(item['税額合計[円/現地通貨]'])}</td>
                                <td>{formatCurrencyString(item['受取金額[円/現地通貨]'])}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </ReceiptTemplate>
    )
};
