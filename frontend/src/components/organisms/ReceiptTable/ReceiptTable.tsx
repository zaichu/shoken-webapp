import React from 'react';
import { Table, TableBody, TableCell, TableHeader, TableRow } from '@/components/atoms/Table';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { useForceResize } from '@/hooks/common/useResize';

type DataItem = Record<string, unknown>;
type SummaryItem = Record<string, unknown> & { filter: string };
type ColumnConfig = TableColumnConfig | SummaryColumnConfig;

interface ReceiptTableProps<T extends DataItem, S extends SummaryItem> {
    data: T[];
    summary: S[];
    columns: TableColumnConfig[];
    summaryColumns: SummaryColumnConfig[];
    getGroupKey: (item: T) => string;
}

/**
 * 明細表示用テーブルコンポーネント
 * 数値のフォーマットやマイナス値の赤文字表示に対応
 * Context APIを使用してリサイズイベントを受信
 */
export function ReceiptTable<T extends DataItem, S extends SummaryItem>({
    data,
    summary,
    columns,
    summaryColumns,
    getGroupKey
}: ReceiptTableProps<T, S>) {
    // Context からの forceResize を取得
    const forceResize = useForceResize();

    const renderCell = (value: unknown, column: ColumnConfig, key: string, style?: React.CSSProperties) => {
        const formattedValue = column.format ? column.format(value) : value;
        const isHtml = typeof formattedValue === 'string' && /<[^>]*>/.test(formattedValue);
        
        const cellStyle: React.CSSProperties = {
            width: 'width' in column ? column.width : undefined,
            textAlign: column.textAlign,
            ...style
        };

        const props = {
            key,
            style: cellStyle,
            colSpan: 'colSpan' in column ? column.colSpan : undefined
        };

        return isHtml ? (
            <TableCell {...props} dangerouslySetInnerHTML={{ __html: formattedValue as string }} />
        ) : (
            <TableCell {...props}>{String(formattedValue ?? '')}</TableCell>
        );
    };

    const renderDataRows = (items: T[], keyPrefix: string) => 
        items.map((item, itemIndex) => (
            <TableRow key={`${keyPrefix}-${itemIndex}`}>
                {columns.map((column, colIndex) =>
                    renderCell(item[column.key], column, `${keyPrefix}-${itemIndex}-${colIndex}`)
                )}
            </TableRow>
        ));

    const renderGroupedRows = () => 
        summary.map((summaryItem, summaryIndex) => {
            const groupItems = data.filter(item => getGroupKey(item) === summaryItem.filter);
            
            return (
                <React.Fragment key={`group-${summaryIndex}`}>
                    {renderDataRows(groupItems, `item-${summaryIndex}`)}
                    <TableRow className="table-info">
                        {summaryColumns.map((column, colIndex) =>
                            renderCell(
                                summaryItem[column.key],
                                column,
                                `summary-${summaryIndex}-${colIndex}`,
                                { fontWeight: 'bold' }
                            )
                        )}
                    </TableRow>
                </React.Fragment>
            );
        });

    return (
        <Table className="mb-0" bordered small responsive forceResize={forceResize}>
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
                {summary.length > 0 ? renderGroupedRows() : renderDataRows(data, 'item')}
            </TableBody>
        </Table>
    );
}
