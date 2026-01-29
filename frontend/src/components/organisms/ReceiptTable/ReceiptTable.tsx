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
    formatGroupHeader?: (key: string) => string;
    onSearch?: (query: string) => void;
}

/**
 * 明細表示用テーブルコンポーネント
 */
const defaultFormatGroupHeader = (key: string): string => {
    if (/^\d{4}-\d{2}-\d{2}$/.test(key)) {
        const date = new Date(key);
        return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
    }
    if (/^\d{4}-\d{2}$/.test(key)) {
        const [year, month] = key.split('-');
        return `${year}年${parseInt(month, 10)}月`;
    }
    if (/^\d{4}$/.test(key)) {
        return `${key}年`;
    }
    return key;
};

export function ReceiptTable<T extends DataItem, S extends SummaryItem>({
    data,
    summary,
    columns,
    summaryColumns,
    getGroupKey,
    formatGroupHeader = defaultFormatGroupHeader,
    onSearch
}: ReceiptTableProps<T, S>) {
    const forceResize = useForceResize();

    // テーブル内のクリックイベントをハンドル（銘柄コードリンク用）
    const handleTableClick = (e: React.MouseEvent<HTMLTableElement>) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('security-code-link') && onSearch) {
            const searchValue = target.dataset.search;
            if (searchValue) {
                onSearch(searchValue);
            }
        }
    };

    const renderCell = (value: unknown, column: ColumnConfig, key: string, style?: React.CSSProperties) => {
        const formattedValue = column.format ? column.format(value) : value;

        // カラム幅は minWidth として適用（レスポンシブ対応）
        const cellStyle: React.CSSProperties = {
            minWidth: 'width' in column ? column.width : undefined,
            textAlign: column.textAlign,
            ...style
        };

        const props = {
            style: cellStyle,
            colSpan: 'colSpan' in column ? column.colSpan : undefined
        };

        // ReactElementの場合はそのまま描画
        if (React.isValidElement(formattedValue)) {
            return <TableCell key={key} {...props}>{formattedValue}</TableCell>;
        }

        // 文字列でHTMLを含む場合（後方互換性のため残す）
        const isHtml = typeof formattedValue === 'string' && /<[^>]*>/.test(formattedValue);
        if (isHtml) {
            return (
                <TableCell key={key} {...props} dangerouslySetInnerHTML={{ __html: formattedValue as string }} />
            );
        }

        return <TableCell key={key} {...props}>{String(formattedValue ?? '')}</TableCell>;
    };

    const renderDataRows = (items: T[], keyPrefix: string) =>
        items.map((item, itemIndex) => (
            <TableRow key={`${keyPrefix}-${itemIndex}`}>
                {columns.map((column, colIndex) =>
                    renderCell(item[column.key], column, `${keyPrefix}-${itemIndex}-${colIndex}`)
                )}
            </TableRow>
        ));

    // サマリー値のフォーマット
    const formatSummaryValue = (value: unknown, column: SummaryColumnConfig) => {
        return column.format ? column.format(value) : value;
    };

    // サマリーのラベルを取得
    const getSummaryLabel = (key: string): string => {
        const labels: Record<string, string> = {
            'total_realized_profit_and_loss': '損益',
            'dividends_before_tax': '配当',
            'total_taxes': '税額',
            'taxes': '税額',
            'total_realized_profit_and_loss_after_tax': '税引後',
            'net_amount_received': '受取額',
        };
        return labels[key] || key;
    };

    const summaryGroupKeys = new Set(data.map(item => getGroupKey(item)));
    const summaryToRender = summary.filter(summaryItem => summaryGroupKeys.has(summaryItem.filter));

    const renderGroupedRows = () =>
        summaryToRender.map((summaryItem, summaryIndex) => {
            const groupItems = data.filter(item => getGroupKey(item) === summaryItem.filter);
            const headerText = formatGroupHeader(summaryItem.filter);
            const itemCount = groupItems.length;

            // サマリー値の取得
            const summaryValues = summaryColumns.map(column => ({
                key: column.key,
                label: getSummaryLabel(column.key),
                value: formatSummaryValue(summaryItem[column.key], column)
            }));

            const summaryBorderClass = summaryIndex > 0 ? 'border-t-2 border-primary-hover' : '';
            const summaryLeftClass = `bg-primary text-white font-semibold ${summaryBorderClass} border-l-4 border-primary-dark`.trim();
            const summaryValueClass = `bg-primary text-white text-right font-bold ${summaryBorderClass}`.trim();

            return (
                <React.Fragment key={`group-${summaryIndex}`}>
                    {/* グループサマリー行 */}
                    {summaryColumns.length > 0 && (
                        <TableRow>
                            <TableCell
                                colSpan={columns.length - summaryColumns.length}
                                className={summaryLeftClass}
                            >
                                <span className="text-base font-bold">
                                    {headerText}
                                </span>
                                <span className="ml-3 inline-flex items-center rounded bg-white/20 px-2.5 py-0.5 text-xs font-semibold text-white">
                                    {itemCount}件
                                </span>
                            </TableCell>
                            {summaryValues.map((sv, idx) => (
                                <TableCell
                                    key={idx}
                                    className={summaryValueClass}
                                >
                                    {String(sv.value)}
                                </TableCell>
                            ))}
                        </TableRow>
                    )}
                    {/* サマリーがない場合のヘッダー */}
                    {summaryColumns.length === 0 && (
                        <TableRow>
                            <TableCell
                                colSpan={columns.length}
                                className={summaryLeftClass}
                            >
                                <span className="text-base font-bold">
                                    {headerText}
                                </span>
                                <span className="ml-3 inline-flex items-center rounded bg-white/20 px-2.5 py-0.5 text-xs font-semibold text-white">
                                    {itemCount}件
                                </span>
                            </TableCell>
                        </TableRow>
                    )}
                    {/* 明細行 */}
                    {groupItems.map((item, itemIndex) => (
                        <TableRow key={`item-${summaryIndex}-${itemIndex}`}>
                            {columns.map((column, colIndex) =>
                                renderCell(item[column.key], column, `item-${summaryIndex}-${itemIndex}-${colIndex}`)
                            )}
                        </TableRow>
                    ))}
                </React.Fragment>
            );
        });

    return (
        <Table
            className="mb-0 table-fixed"
            bordered
            small
            responsive
            forceResize={forceResize}
            maxHeight={520}
            onClick={handleTableClick}
        >
            <TableHeader>
                <TableRow className="bg-warning/20 text-center">
                    {columns.map((column, index) => (
                        <TableCell
                            as="th"
                            className="text-center"
                            style={{ minWidth: column.width }}
                            key={index}
                        >
                            {column.header}
                        </TableCell>
                    ))}
                </TableRow>
            </TableHeader>
            <TableBody>
                {summaryToRender.length > 0 ? renderGroupedRows() : renderDataRows(data, 'item')}
            </TableBody>
        </Table>
    );
}
