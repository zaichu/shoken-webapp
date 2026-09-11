import React from 'react';
import { Table, TableBody, TableCell, TableHeader, TableRow } from '@/components/molecules/Table';
import type { TableColumnConfig, SummaryColumnConfig } from '@/features/receipt/types';
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

const toText = (value: unknown): string | number => {
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

const renderCell = (
    value: unknown,
    column: ColumnConfig,
    key: string,
    style?: React.CSSProperties,
    row?: DataItem
) => {
    const formattedValue = 'render' in column && column.render && row
        ? column.render(value, row)
        : column.format ? column.format(value) : value;
    const isNegative = !React.isValidElement(formattedValue) && isNegativeValue(value);
    const displayText = React.isValidElement(formattedValue) ? undefined : String(toText(formattedValue));

    const cellStyle: React.CSSProperties = {
        width: 'width' in column ? column.width : undefined,
        maxWidth: 'width' in column ? column.width : undefined,
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
        <TableCell key={key} {...props} title={displayText} data-negative={isNegative ? 'true' : undefined}>
            {toText(formattedValue)}
        </TableCell>
    );
};

const formatSummaryValue = (value: unknown, column: SummaryColumnConfig) =>
    column.format ? column.format(value) : value;

/**
 * スマホカード表示用: 明細1件の表示内容を組み立てる。
 * ラベルは TableColumnConfig.header、値は render / format を流用する。
 */
const formatCardValue = (
    value: unknown,
    column: TableColumnConfig,
    row: DataItem
): { node: React.ReactNode; rawValue: unknown; displayText: string | undefined } => {
    const formattedValue = column.render
        ? column.render(value, row)
        : column.format ? column.format(value) : value;
    const displayText = React.isValidElement(formattedValue) ? undefined : String(toText(formattedValue));
    return { node: React.isValidElement(formattedValue) ? formattedValue : toText(formattedValue), rawValue: value, displayText };
};

/**
 * スマホカード表示用: 集計カラムのラベルを列定義から解決する。
 * DomesticStock のように集計キーへ total_ プレフィックスが付く場合に対応する。
 */
const resolveSummaryLabel = (summaryKey: string, columns: TableColumnConfig[]): string => {
    const direct = columns.find(column => column.key === summaryKey);
    if (direct) return direct.header;
    const strippedKey = summaryKey.replace(/^total_/, '');
    const byStrippedKey = columns.find(column => column.key === strippedKey);
    if (byStrippedKey) return byStrippedKey.header;
    return summaryKey;
};

function renderCardItem<T extends DataItem>(
    item: T,
    columns: TableColumnConfig[],
    keyPrefix: string
) {
    return (
        <div
            key={keyPrefix}
            data-testid="receipt-card"
            className="rounded-lg border border-slate-950/10 bg-white px-3.5 py-2 shadow-sm"
        >
            <dl>
                {columns.map((column, colIndex) => {
                    const { node, rawValue, displayText } = formatCardValue(item[column.key], column, item);
                    const isNegative = !React.isValidElement(node) && isNegativeValue(rawValue);
                    return (
                        <div
                            key={`${keyPrefix}-${colIndex}`}
                            className="flex items-start justify-between gap-3 border-b border-slate-100 py-1.5 last:border-b-0"
                        >
                            <dt className="shrink-0 pt-0.5 text-xs text-slate-500">{column.header}</dt>
                            <dd
                                className="min-w-0 break-words text-right text-sm font-semibold tabular-nums text-slate-800"
                                title={displayText}
                                data-negative={isNegative ? 'true' : undefined}
                            >
                                {node}
                            </dd>
                        </div>
                    );
                })}
            </dl>
        </div>
    );
}

function renderGroupedCards<T extends DataItem, S extends SummaryItem>(
    summaryToRender: S[],
    groupedDataMap: Map<string, T[]>,
    columns: TableColumnConfig[],
    summaryColumns: SummaryColumnConfig[],
    formatGroupHeader: (key: string) => string
) {
    return summaryToRender.map((summaryItem, summaryIndex) => {
        const groupItems = groupedDataMap.get(summaryItem.filter) ?? [];
        const headerText = formatGroupHeader(summaryItem.filter);
        const itemCount = groupItems.length;

        const summaryValues = summaryColumns.map(column => ({
            key: column.key,
            label: resolveSummaryLabel(column.key, columns),
            value: formatSummaryValue(summaryItem[column.key], column),
            rawValue: summaryItem[column.key]
        }));

        return (
            <section key={`card-group-${summaryIndex}`} data-testid="receipt-card-group">
                <div className="rounded-lg border border-slate-300 bg-slate-100 px-3.5 py-2.5">
                    <div className="flex items-center">
                        <span className="text-sm font-medium text-slate-800">
                            {headerText}
                        </span>
                        <span className="ml-2 inline-flex items-center rounded bg-slate-600 px-2 py-0.5 text-xs font-medium text-white">
                            {itemCount}件
                        </span>
                    </div>
                    {summaryValues.length > 0 && (
                        <dl className="mt-2 border-t border-slate-200 pt-2">
                            {summaryValues.map((sv) => (
                                <div key={sv.key} className="flex items-start justify-between gap-3 py-0.5">
                                    <dt className="shrink-0 text-xs text-slate-600">{sv.label}</dt>
                                    <dd
                                        className="min-w-0 break-words text-right text-sm font-semibold tabular-nums text-slate-800"
                                        data-negative={!React.isValidElement(sv.value) && isNegativeValue(sv.rawValue) ? 'true' : undefined}
                                    >
                                        {React.isValidElement(sv.value) ? sv.value : toText(sv.value)}
                                    </dd>
                                </div>
                            ))}
                        </dl>
                    )}
                </div>
                <div className="mt-2 space-y-2">
                    {groupItems.map((item, itemIndex) =>
                        renderCardItem(item, columns, `card-item-${summaryIndex}-${itemIndex}`)
                    )}
                </div>
            </section>
        );
    });
}

function renderDataRows<T extends DataItem>(
    items: T[],
    columns: TableColumnConfig[],
    keyPrefix: string
) {
    return items.map((item, itemIndex) => (
        <TableRow key={`${keyPrefix}-${itemIndex}`}>
            {columns.map((column, colIndex) =>
                renderCell(item[column.key], column, `${keyPrefix}-${itemIndex}-${colIndex}`, undefined, item)
            )}
        </TableRow>
    ));
}

function renderGroupedRows<T extends DataItem, S extends SummaryItem>(
    summaryToRender: S[],
    groupedDataMap: Map<string, T[]>,
    columns: TableColumnConfig[],
    summaryColumns: SummaryColumnConfig[],
    formatGroupHeader: (key: string) => string
) {
    return summaryToRender.map((summaryItem, summaryIndex) => {
        const groupItems = groupedDataMap.get(summaryItem.filter) ?? [];
        const headerText = formatGroupHeader(summaryItem.filter);
        const itemCount = groupItems.length;

        const summaryValues = summaryColumns.map(column => ({
            key: column.key,
            value: formatSummaryValue(summaryItem[column.key], column),
            rawValue: summaryItem[column.key]
        }));

        const summaryBorderClass = summaryIndex > 0 && 'border-t-2 border-slate-300';
        const summaryLeftClass = cn('whitespace-normal bg-slate-100 text-slate-800 font-semibold border-l-2 border-slate-500', summaryBorderClass);
        const summaryValueClass = cn('bg-slate-100 text-slate-800 text-right font-semibold', summaryBorderClass);

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
                                {React.isValidElement(sv.value) ? sv.value : toText(sv.value)}
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
                            renderCell(item[column.key], column, `item-${summaryIndex}-${itemIndex}-${colIndex}`, undefined, item)
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
    // テーブル / カード内のクリックイベントをハンドル（銘柄コードリンク用）
    // PC テーブルとスマホカードで同一ロジックを使う
    const handleContentClick = (e: React.MouseEvent<HTMLElement>) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('security-code-link') && onSearch) {
            const searchValue = target.dataset.search;
            if (searchValue) {
                onSearch(searchValue);
            }
        }
    };

    // データをグループキーで事前にマップ化（O(n) → O(n+m) に削減）
    const groupedDataMap = new Map<string, T[]>();
    for (const item of data) {
        const key = getGroupKey(item);
        const group = groupedDataMap.get(key);
        if (group) {
            group.push(item);
        } else {
            groupedDataMap.set(key, [item]);
        }
    }
    const summaryToRender = summary.filter(summaryItem => groupedDataMap.has(summaryItem.filter));

    return (
        <>
            {/* PC幅 (sm以上): 従来のテーブル表示のまま変えない */}
            <div className="hidden sm:block">
                <Table
                    className="mb-0"
                    bordered
                    small
                    responsive
                    onClick={handleContentClick}
                >
                    <TableHeader>
                        <TableRow className="bg-slate-50 text-center">
                            {columns.map((column) => (
                                <TableCell
                                    as="th"
                                    className="text-center whitespace-nowrap"
                                    style={{ width: column.width, maxWidth: column.width }}
                                    key={column.header}
                                >
                                    {column.header}
                                </TableCell>
                            ))}
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {summaryToRender.length > 0
                            ? renderGroupedRows(summaryToRender, groupedDataMap, columns, summaryColumns, formatGroupHeader)
                            : renderDataRows(data, columns, 'item')
                        }
                    </TableBody>
                </Table>
            </div>
            {/* スマホ幅 (640px未満) のみ: 1取引=1カードの縦積み表示 */}
            <div className="sm:hidden" onClick={handleContentClick} data-testid="receipt-card-list">
                {summaryToRender.length > 0 ? (
                    <div className="space-y-4">
                        {renderGroupedCards(summaryToRender, groupedDataMap, columns, summaryColumns, formatGroupHeader)}
                    </div>
                ) : (
                    <div className="space-y-2">
                        {data.map((item, itemIndex) =>
                            renderCardItem(item, columns, `card-item-${itemIndex}`)
                        )}
                    </div>
                )}
            </div>
        </>
    );
}
