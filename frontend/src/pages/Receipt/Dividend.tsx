import { ReceiptTemplate, Table, TableBody, TableCell, TableHeader, TableRow } from '@/components';
import React, { useMemo, useState } from 'react';
import { formatCurrencyString, formatNumber } from '@/lib/utils/format';

interface DividendData {
    settlement_date: Date;
    product: string;
    account: string;
    security_code: string;
    security_name: string;
    unit_price: number;
    shares: number;
    dividends_before_tax: number;
    taxes: number;
    net_amount_received: number;
}

interface DividendProps {
    csvData: any[];
}

interface Calculations {
    total_dividends_before_tax: number;
    total_taxes: number;
    total_net_amount_received: number;
}

interface MonthlyTotal {
    month: string;
    dividends_before_tax: number;
    taxes: number;
    net_amount_received: number;
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
                        <h4 className='mb-0'>{formatCurrencyString(calculations.total_dividends_before_tax)}</h4>
                    </div>
                    <div className="col">
                        <h6 className='mb-0'>税額合計</h6>
                        <h4 className='mb-0'>{formatCurrencyString(calculations.total_taxes)}</h4>
                    </div>
                    <div className="col">
                        <h6 className='mb-0'>受取金額合計</h6>
                        <h4 className='mb-0'>{formatCurrencyString(calculations.total_net_amount_received)}</h4>
                    </div>
                </div>
            </div>
        </div>
    )
}

export const Dividend: React.FC<DividendProps> = ({ csvData }) => {
    const [searchQuery, setSearchQuery] = useState('');
    const onSearch = (query: string) => setSearchQuery(query);
    var dividendData: DividendData[] = useMemo(() => {
        return csvData.map((item) => ({
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
    }, [csvData]);

    dividendData = useMemo(() => {
        return [...dividendData].sort((a, b) => {
            return a.settlement_date.getTime() - b.settlement_date.getTime();
        });
    }, [dividendData]);

    const calculations: Calculations = useMemo(() => {
        return dividendData.reduce((acc, item) => {
            return {
                total_dividends_before_tax: acc.total_dividends_before_tax + item.dividends_before_tax,
                total_taxes: acc.total_taxes + item.taxes,
                total_net_amount_received: acc.total_net_amount_received + item.net_amount_received,
            };
        }, {
            total_dividends_before_tax: 0,
            total_taxes: 0,
            total_net_amount_received: 0
        });
    }, [csvData]);

    const monthlyTotals: MonthlyTotal[] = useMemo(() => {
        const monthMap = new Map<string, MonthlyTotal>();
        dividendData.forEach(item => {
            const date = item.settlement_date;
            const monthKey = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}`;
            const monthLabel = `${date.getFullYear()}年${(date.getMonth() + 1).toString().padStart(2, '0')}月`;

            if (!monthMap.has(monthKey)) {
                monthMap.set(monthKey, {
                    month: monthLabel,
                    dividends_before_tax: 0,
                    taxes: 0,
                    net_amount_received: 0
                });
            }

            const monthData = monthMap.get(monthKey)!;
            monthData.dividends_before_tax += item.dividends_before_tax;
            monthData.taxes += item.taxes;
            monthData.net_amount_received += item.net_amount_received;
        });

        // Mapから配列に変換して日付順にソート
        return Array.from(monthMap.values())
            .sort((a, b) => a.month.localeCompare(b.month));
    }, [dividendData]);

    const searchOptions =
        dividendData.map((item) => ({
            value: item.security_code ? item.security_code : item.security_name,
            label: (item.security_code ? item.security_code + ':' : '') + item.security_name,
        })).filter((item, index, self) =>
            index === self.findIndex((t) => (
                t.value === item.value
            ))
        ).sort((a, b) => a.value.localeCompare(b.value));

    const filteredData = dividendData.filter((item) => {
        const searchValue = searchQuery.toLowerCase();
        return item.security_code.toLowerCase().includes(searchValue) || item.security_name.toLowerCase().includes(searchValue);
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
                    {monthlyTotals.map((monthTotal, monthIndex) => {
                        const monthData = filteredData.filter(item => {
                            const date = item.settlement_date;
                            const monthStr = `${date.getFullYear()}年${(date.getMonth() + 1).toString().padStart(2, '0')}月`;
                            return monthStr === monthTotal.month;
                        });
                        return (
                            <React.Fragment key={`month-${monthIndex}`}>

                                {monthData.map((item, index) => (
                                    <TableRow key={`item-${monthIndex}-${index}`}>
                                        <TableCell>{item.settlement_date.toLocaleDateString('ja-JP', { year: 'numeric', month: '2-digit', day: '2-digit', })}</TableCell>
                                        <TableCell>{item.product}</TableCell>
                                        <TableCell style={{ width: '100px' }}>{item.account}</TableCell>
                                        <TableCell>{item.security_code}</TableCell>
                                        <TableCell style={{ width: '250px' }}>{item.security_name}</TableCell>
                                        <TableCell style={{ width: '80px', textAlign: 'right' }}>{formatCurrencyString(item.unit_price)}</TableCell>
                                        <TableCell style={{ width: '100px', textAlign: 'right' }}>{formatNumber(item.shares)}</TableCell>
                                        <TableCell style={{ width: '150px', textAlign: 'right' }}>{formatCurrencyString(item.dividends_before_tax)}</TableCell>
                                        <TableCell style={{ width: '100px', textAlign: 'right' }}>{formatCurrencyString(item.taxes)}</TableCell>
                                        <TableCell style={{ width: '100px', textAlign: 'right' }}>{formatCurrencyString(item.net_amount_received)}</TableCell>
                                    </TableRow>
                                ))}
                                <TableRow variant="info">
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell>{""}</TableCell>
                                    <TableCell style={{ fontWeight: 'bold', textAlign: 'right' }}>{formatCurrencyString(monthTotal.dividends_before_tax)}</TableCell>
                                    <TableCell style={{ fontWeight: 'bold', textAlign: 'right' }}>{formatCurrencyString(monthTotal.taxes)}</TableCell>
                                    <TableCell style={{ fontWeight: 'bold', textAlign: 'right' }}>{formatCurrencyString(monthTotal.net_amount_received)}</TableCell>
                                </TableRow>
                            </React.Fragment>
                        );
                    })}
                </TableBody>
            </Table>
        </ReceiptTemplate >
    )
};