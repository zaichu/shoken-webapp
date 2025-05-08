import React from 'react';
import { Table, TableBody, TableCell, TableHeader, TableRow } from '@/components/atoms';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';

interface ReceiptTableProps<T, S> {
    data: T[];
    summary: S[];
    columns: TableColumnConfig[];
    summaryColumns: SummaryColumnConfig[];
    groupByField?: string;
    getGroupKey: (item: T) => string;
}

/**
 * 明細表示用テーブルコンポーネント
 * 数値のフォーマットやマイナス値の赤文字表示に対応
 */
export function ReceiptTable<T, S>({
    data,
    summary,
    columns,
    summaryColumns,
    getGroupKey
}: ReceiptTableProps<T, S>) {
    return (
        <Table className='mb-0' bordered small responsive style={{ maxHeight: '500px', overflowY: 'auto' }}>
            <TableHeader>
                <TableRow variant="warning">
                    {columns.map((column, index) => (
                        <TableCell
                            as="th"
                            style={{ textAlign: 'center', width: column.width }}
                            key={index}
                        >
                            {column.header}
                        </TableCell>
                    ))}
                </TableRow>
            </TableHeader>
            <TableBody>
                {summary.map((summaryItem, summaryIndex) => {
                    const groupItems = data.filter(item =>
                        getGroupKey(item) === (summaryItem as any).filter
                    );

                    return (
                        <React.Fragment key={`group-${summaryIndex}`}>
                            {groupItems.map((item, index) => (
                                <TableRow key={`item-${summaryIndex}-${index}`}>
                                    {columns.map((column, colIndex) => {
                                        const cellValue = (item as any)[column.key];
                                        const formattedValue = column.format
                                            ? column.format(cellValue)
                                            : cellValue;

                                        // HTMLタグが含まれているかどうかチェック
                                        const containsHtml = typeof formattedValue === 'string' &&
                                            (formattedValue.includes('<span') ||
                                                formattedValue.includes('<div') ||
                                                formattedValue.includes('<p'));

                                        return (
                                            <TableCell
                                                key={`cell-${summaryIndex}-${index}-${colIndex}`}
                                                style={{
                                                    width: column.width,
                                                    textAlign: column.textAlign
                                                }}
                                                dangerouslySetInnerHTML={containsHtml ? { __html: formattedValue as string } : undefined}
                                            >
                                                {!containsHtml && formattedValue}
                                            </TableCell>
                                        );
                                    })}
                                </TableRow>
                            ))}

                            <TableRow variant="info">
                                {summaryColumns.map((column, colIndex) => {
                                    const summaryValue = (summaryItem as any)[column.key];
                                    const formattedValue = column.format
                                        ? column.format(summaryValue)
                                        : summaryValue;

                                    // HTMLタグが含まれているかどうかチェック
                                    const containsHtml = typeof formattedValue === 'string' &&
                                        (formattedValue.includes('<span') ||
                                            formattedValue.includes('<div') ||
                                            formattedValue.includes('<p'));

                                    return (
                                        <TableCell
                                            key={`summary-${summaryIndex}-${colIndex}`}
                                            style={{
                                                fontWeight: 'bold',
                                                textAlign: column.textAlign || 'right'
                                            }}
                                            colSpan={column.colSpan}
                                            dangerouslySetInnerHTML={containsHtml ? { __html: formattedValue as string } : undefined}
                                        >
                                            {!containsHtml && formattedValue}
                                        </TableCell>
                                    );
                                })}
                            </TableRow>
                        </React.Fragment>
                    );
                })}
            </TableBody>
        </Table>
    );
}
