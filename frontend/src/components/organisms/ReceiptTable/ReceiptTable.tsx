import React from 'react';
import { Table, TableBody, TableCell, TableHeader, TableRow } from '@/components/atoms/Table';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';
import { useForceResize } from '@/hooks/common/useResize';
import { cn } from '@/lib/utils/classNames';

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

const renderTextValue = (value: unknown): string | number => {
    if (typeof value === 'string' || typeof value === 'number') return value;
    if (value === null || value === undefined) return '';
    return String(value);
};

const isNegativeValue = (value: unknown): boolean => {
    if (typeof value === 'number') {
        return Number.isFinite(value) && value < 0;
    }
    if (typeof value !== 'string') {
        return false;
    }

    const normalized = value
        .trim()
        .replace(/[¥￥$€£]/g, '')
        .replace(/,/g, '')
        .replace(/\s+/g, '');

    if (!normalized || !/^[-+]?\d+(\.\d+)?$/.test(normalized)) {
        return false;
    }

    return Number(normalized) < 0;
};

const renderCell = (value: unknown, column: ColumnConfig, key: string, style?: React.CSSProperties) => {
    const formattedValue = column.format ? column.format(value) : value;
    const isNegative = !React.isValidElement(formattedValue) && isNegativeValue(value);

    const cellStyle: React.CSSProperties = {
        minWidth: 'width' in column ? column.width : undefined,
        textAlign: column.textAlign,
        fontVariantNumeric: column.textAlign === 'right' ? 'tabular-nums' : undefined,
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

    return (
        <TableCell key={key} {...props} data-negative={isNegative ? 'true' : undefined}>
            {renderTextValue(formattedValue)}
        </TableCell>
    );
};

const formatSummaryValue = (value: unknown, column: SummaryColumnConfig) =>
    column.format ? column.format(value) : value;

function renderDataRows<T extends DataItem>(
    items: T[],
    columns: TableColumnConfig[],
    keyPrefix: string
) {
    return items.map((item, itemIndex) => (
        <TableRow key={`${keyPrefix}-${itemIndex}`}>
            {columns.map((column, colIndex) =>
                renderCell(item[column.key], column, `${keyPrefix}-${itemIndex}-${colIndex}`)
            )}
        </TableRow>
    ));
}

function renderGroupedRows<T extends DataItem, S extends SummaryItem>(
    summaryToRender: S[],
    data: T[],
    columns: TableColumnConfig[],
    summaryColumns: SummaryColumnConfig[],
    getGroupKey: (item: T) => string,
    formatGroupHeader: (key: string) => string
) {
    return summaryToRender.map((summaryItem, summaryIndex) => {
        const groupItems = data.filter(item => getGroupKey(item) === summaryItem.filter);
        const headerText = formatGroupHeader(summaryItem.filter);
        const itemCount = groupItems.length;

        const summaryValues = summaryColumns.map(column => ({
            key: column.key,
            value: formatSummaryValue(summaryItem[column.key], column),
            rawValue: summaryItem[column.key]
        }));

        const summaryBorderClass = summaryIndex > 0 && 'border-t border-slate-200';
        const summaryLeftClass = cn('bg-slate-50 text-slate-700 font-medium border-l-2 border-slate-400', summaryBorderClass);
        const summaryValueClass = cn('bg-slate-50 text-slate-700 text-right font-medium', summaryBorderClass);

        return (
            <React.Fragment key={`group-${summaryIndex}`}>
                {/* グループサマリー行 */}
                {summaryColumns.length > 0 && (
                    <TableRow>
                        <TableCell
                            colSpan={columns.length - summaryColumns.length}
                            className={summaryLeftClass}
                        >
                            <span className="text-sm font-medium">
                                {headerText}
                            </span>
                            <span className="ml-2 inline-flex items-center rounded bg-slate-600 px-2 py-0.5 text-xs font-medium text-white">
                                {itemCount}件
                            </span>
                        </TableCell>
                        {summaryValues.map((sv) => (
                            <TableCell
                                key={sv.key}
                                className={summaryValueClass}
                                data-negative={!React.isValidElement(sv.value) && isNegativeValue(sv.rawValue) ? 'true' : undefined}
                            >
                                {React.isValidElement(sv.value) ? sv.value : renderTextValue(sv.value)}
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
                            <span className="text-sm font-medium">
                                {headerText}
                            </span>
                            <span className="ml-2 inline-flex items-center rounded bg-slate-600 px-2 py-0.5 text-xs font-medium text-white">
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

    const summaryGroupKeys = new Set(data.map(item => getGroupKey(item)));
    const summaryToRender = summary.filter(summaryItem => summaryGroupKeys.has(summaryItem.filter));

    return (
        <Table
            className="mb-0"
            bordered
            small
            responsive
            forceResize={forceResize}
            onClick={handleTableClick}
        >
            <TableHeader>
                <TableRow className="bg-slate-50 text-center">
                    {columns.map((column) => (
                        <TableCell
                            as="th"
                            className="text-center whitespace-nowrap"
                            style={{ minWidth: column.width }}
                            key={column.header}
                        >
                            {column.header}
                        </TableCell>
                    ))}
                </TableRow>
            </TableHeader>
            <TableBody>
                {summaryToRender.length > 0
                    ? renderGroupedRows(summaryToRender, data, columns, summaryColumns, getGroupKey, formatGroupHeader)
                    : renderDataRows(data, columns, 'item')
                }
            </TableBody>
        </Table>
    );
}
