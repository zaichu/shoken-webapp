import React from 'react';
import { Table, TableBody, TableCell, TableHeader, TableRow } from '@/components/atoms/Table';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';

interface ReceiptTableProps<T extends Record<string, unknown>, S extends Record<string, unknown> & { filter: string }> {
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
export function ReceiptTable<T extends Record<string, unknown>, S extends Record<string, unknown> & { filter: string }>({
    data,
    summary,
    columns,
    summaryColumns,
    getGroupKey
}: ReceiptTableProps<T, S>) {
    const renderCell = (
        value: unknown,
        column: TableColumnConfig | SummaryColumnConfig,
        keyPrefix: string,
        index: number,
        additionalStyle?: React.CSSProperties
    ) => {
        const cellValue = value;
        const formattedValue = column.format ? column.format(cellValue) : cellValue;

        // HTMLタグが含まれているかどうかチェック
        const containsHtml = typeof formattedValue === 'string' &&
            /<[^>]*>/.test(formattedValue);

        const style: React.CSSProperties = {
            width: 'width' in column ? column.width : undefined,
            textAlign: column.textAlign,
            ...additionalStyle
        };

        if (containsHtml) {
            return (
                <TableCell
                    key={`${keyPrefix}-${index}`}
                    style={style}
                    colSpan={'colSpan' in column ? column.colSpan : undefined}
                    dangerouslySetInnerHTML={{ __html: formattedValue as string }}
                />
            );
        }

        return (
            <TableCell
                key={`${keyPrefix}-${index}`}
                style={style}
                colSpan={'colSpan' in column ? column.colSpan : undefined}
            >
                {String(formattedValue ?? '')}
            </TableCell>
        );
    };

    return (
        <Table className="mb-0" bordered small responsive>
            <TableHeader>
                <TableRow className="table-warning">
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
                        getGroupKey(item) === summaryItem.filter
                    );

                    return (
                        <React.Fragment key={`group-${summaryIndex}`}>
                            {groupItems.map((item, itemIndex) => (
                                <TableRow key={`item-${summaryIndex}-${itemIndex}`}>
                                    {columns.map((column, colIndex) =>
                                        renderCell(
                                            item[column.key],
                                            column,
                                            `cell-${summaryIndex}-${itemIndex}`,
                                            colIndex
                                        )
                                    )}
                                </TableRow>
                            ))}

                            <TableRow className="table-info">
                                {summaryColumns.map((column, colIndex) =>
                                    renderCell(
                                        summaryItem[column.key],
                                        column,
                                        `summary-${summaryIndex}`,
                                        colIndex,
                                        { fontWeight: 'bold' }
                                    )
                                )}
                            </TableRow>
                        </React.Fragment>
                    );
                })}
            </TableBody>
        </Table>
    );
}
