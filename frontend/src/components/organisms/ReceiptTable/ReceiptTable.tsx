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

export function ReceiptTable<T, S>({
    data,
    summary,
    columns,
    summaryColumns,
    getGroupKey
}: ReceiptTableProps<T, S>) {
    return (
        <Table className='mb-0' striped bordered hover small responsive style={{ maxHeight: '500px', overflowY: 'auto' }}>
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
                                    {columns.map((column, colIndex) => (
                                        <TableCell
                                            key={`cell-${summaryIndex}-${index}-${colIndex}`}
                                            style={{
                                                width: column.width,
                                                textAlign: column.textAlign
                                            }}
                                        >
                                            {column.format
                                                ? column.format((item as any)[column.key])
                                                : (item as any)[column.key]
                                            }
                                        </TableCell>
                                    ))}
                                </TableRow>
                            ))}

                            <TableRow variant="info">
                                {summaryColumns.map((column, colIndex) => (
                                    <TableCell
                                        key={`summary-${summaryIndex}-${colIndex}`}
                                        style={{
                                            fontWeight: 'bold',
                                            textAlign: column.textAlign || 'right'
                                        }}
                                        colSpan={column.colSpan}
                                    >
                                        {column.format
                                            ? column.format((summaryItem as any)[column.key])
                                            : (summaryItem as any)[column.key]
                                        }
                                    </TableCell>
                                ))}
                            </TableRow>
                        </React.Fragment>
                    );
                })}
            </TableBody>
        </Table>
    );
}
