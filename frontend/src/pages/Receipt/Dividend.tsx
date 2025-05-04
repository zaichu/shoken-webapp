import { ReceiptTemplate, Table, TableBody, TableCell, TableHeader, TableRow } from '@/components';
import React, { useMemo, useState } from 'react';
import { formatCurrencyString, formatNumber } from '@/lib/utils/format';

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
        <div className="card shadow-sm mt-1">
            <div className="card-header bg-primary text-white">
                <h5 className="mb-0">集計情報</h5>
            </div>
            <div className="card-body">
                <div className="row">
                    <div className="col">
                        <h6 className='mb-0'>配当金合計</h6>
                        <h4 className='mb-0'>{formatCurrencyString(calculations.totalDividends)}</h4>
                    </div>
                    <div className="col">
                        <h6 className='mb-0'>税額合計</h6>
                        <h4 className='mb-0'>{formatCurrencyString(calculations.totalTaxes)}</h4>
                    </div>
                    <div className="col">
                        <h6 className='mb-0'>受取金額合計</h6>
                        <h4 className='mb-0'>{formatCurrencyString(calculations.totalNetAmount)}</h4>
                    </div>
                </div>
            </div>
        </div>
    )
}

export const Dividend: React.FC<DividendProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');
    const onSearch = (query: string) => setSearchQuery(query);

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

    const searchOptions =
        csvData.map((item) => ({
            value: item['銘柄コード'] ? item['銘柄コード'] : item['銘柄'],
            label: item['銘柄コード'] ? item['銘柄コード'] + ':' + item['銘柄'] : item['銘柄']
        })).filter((item, index, self) =>
            index === self.findIndex((t) => (
                t.value === item.value
            ))
        ).sort((a, b) => a.value.localeCompare(b.value));

    const filteredData = csvData.filter((item) => {
        const searchValue = searchQuery.toLowerCase();
        return item['銘柄コード'].toLowerCase().includes(searchValue) || item['銘柄'].toLowerCase().includes(searchValue);
    });

    const headerName = ['入金日', '商品', '口座', '銘柄コード', '銘柄名', '単価', '数量[株]', '配当・分配金', '税額', '受取金額', '配当・分配金合計', '税額合計', '受取金額合計'];
    return (
        <ReceiptTemplate title="配当金" header={<Header calculations={calculations} />} searchQuery={searchQuery} onSearch={onSearch} searchOptions={searchOptions}>
            <Table className='mb-0' striped bordered hover small responsive style={{ maxHeight: '500px', overflowY: 'auto' }}>
                <TableHeader>
                    <TableRow variant="warning">
                        {headerName.map((name, index) => (
                            <TableCell as="th" style={{ textAlign: 'center' }} key={index}>{name}</TableCell>
                        ))}
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {filteredData.map((item, index) => (
                        <TableRow key={index}>
                            <TableCell>{item['入金日']}</TableCell>
                            <TableCell>{item['商品']}</TableCell>
                            <TableCell style={{ width: '100px' }}>{item['口座']}</TableCell>
                            <TableCell>{item['銘柄コード']}</TableCell>
                            <TableCell style={{ width: '250px' }}>{item['銘柄']}</TableCell>
                            <TableCell style={{ width: '80px', textAlign: 'right' }}>{formatCurrencyString(item['単価[円/現地通貨]'])}</TableCell>
                            <TableCell style={{ width: '100px', textAlign: 'right' }}>{formatNumber(item['数量[株/口]'])}</TableCell>
                            <TableCell style={{ width: '150px', textAlign: 'right' }}>{formatCurrencyString(item['配当・分配金合計（税引前）[円/現地通貨]'])}</TableCell>
                            <TableCell style={{ width: '100px', textAlign: 'right' }}>{formatCurrencyString(item['税額合計[円/現地通貨]'])}</TableCell>
                            <TableCell style={{ width: '100px', textAlign: 'right' }}>{formatCurrencyString(item['受取金額[円/現地通貨]'])}</TableCell>
                        </TableRow>
                    ))}
                </TableBody>
            </Table>
        </ReceiptTemplate>
    )
};
