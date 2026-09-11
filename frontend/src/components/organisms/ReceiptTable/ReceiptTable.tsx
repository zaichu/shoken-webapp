import React, { useId, useState } from 'react';
import { Table, TableBody, TableCell, TableHeader, TableRow } from '@/components/molecules/Table';
import type { TableColumnConfig, SummaryColumnConfig } from '@/features/receipt/types';
import { cn } from '@/lib/utils/classNames';

type DataItem = Record<string, unknown>;
type SummaryItem = Record<string, unknown> & { filter: string };
type ColumnConfig = TableColumnConfig | SummaryColumnConfig;

interface ReceiptTableProps<T extends DataItem, S extends SummaryItem> {
    primaryKey: string;
    nameKey: string;
    dateKey?: string;
    accountKey?: string;
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

const stripTotalPrefix = (key: string): string => key.replace(/^total_/, '');

/**
 * スマホのグループ見出しに常時表示する主要集計列を決める。
 * primaryKey と一致（total_ プレフィックス差異を吸収）する列を優先し、
 * 見つからなければ末尾列（受取額/税引後の想定）を使う。
 * ReceiptTable は汎用表示に徹し、ドメイン分岐は持たない。
 */
const resolvePrimarySummaryIndex = (
    summaryColumns: SummaryColumnConfig[],
    primaryKey: string
): number => {
    if (summaryColumns.length === 0) return -1;
    const normalizedPrimary = stripTotalPrefix(primaryKey);
    const matched = summaryColumns.findIndex(
        column => stripTotalPrefix(column.key) === normalizedPrimary
    );
    return matched >= 0 ? matched : summaryColumns.length - 1;
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

interface MobileSummaryFields {
    primaryKey: string;
    nameKey: string;
    dateKey?: string;
    accountKey?: string;
}

function ReceiptCard({ item, columns, fields }: {
    item: DataItem;
    columns: TableColumnConfig[];
    fields: MobileSummaryFields;
}) {
    const [expanded, setExpanded] = useState(false);
    const id = useId();
    const name = String(toText(item[fields.nameKey]));
    const primaryColumn = columns.find(column => column.key === fields.primaryKey);
    // ボタン内には文字列だけを置き、列の render が返すリンクやボタンを入れ子にしない。
    const amount = primaryColumn?.format
        ? toText(primaryColumn.format(item[fields.primaryKey]))
        : toText(item[fields.primaryKey]);
    // 日付は Date 型で渡るため、列定義の format（formatJPDate）を通す。
    // グループ見出しに年月があるので先頭の年だけ落として MM/DD にする。
    const dateColumn = columns.find(column => column.key === fields.dateKey);
    const date = fields.dateKey
        ? String(
              toText(
                  dateColumn?.format
                      ? dateColumn.format(item[fields.dateKey])
                      : item[fields.dateKey]
              )
          ).replace(/^\d{4}\//, '')
        : '';
    const account = fields.accountKey ? toText(item[fields.accountKey]) : '';

    return (
        <div data-testid="receipt-card" className="rounded-lg border border-slate-300 bg-white">
            <button
                id={`${id}-button`}
                type="button"
                aria-label={`${name} ${amount}`}
                aria-expanded={expanded}
                aria-controls={`${id}-details`}
                onClick={() => setExpanded(previous => !previous)}
                className="flex min-h-[90px] w-full flex-col justify-center gap-2 rounded-lg px-3 py-3 text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-700"
            >
                <span className="flex w-full items-baseline gap-2">
                    <span className="min-w-0 flex-1 truncate text-base font-semibold text-slate-950">{name}</span>
                    <span className={cn('min-w-[11ch] shrink-0 whitespace-nowrap text-right font-mono text-base font-semibold tabular-nums', isNegativeValue(item[fields.primaryKey]) ? 'text-red-800' : 'text-slate-950')}>
                        {amount}
                    </span>
                </span>
                <span className="flex w-full items-center gap-2 text-xs text-slate-600" aria-hidden="true">
                    <span className="shrink-0">{date}</span>
                    <span className="min-w-0 flex-1 truncate">{account}</span>
                    <svg className={cn('h-4 w-4 shrink-0', expanded && 'rotate-180')} viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
                        <path d="m6 9 6 6 6-6" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                    </svg>
                </span>
            </button>
            <div id={`${id}-details`} role="region" aria-labelledby={`${id}-button`} hidden={!expanded} className="border-t border-slate-300 px-3 py-2">
                {expanded && <dl>
                    {columns.map(column => {
                        const { node, rawValue, displayText } = formatCardValue(item[column.key], column, item);
                        return (
                            <div key={column.key} className="flex items-start justify-between gap-3 border-b border-slate-100 py-1.5 last:border-b-0">
                                <dt className="shrink-0 pt-0.5 text-xs text-slate-600">{column.header}</dt>
                                <dd className={cn('min-w-0 break-words text-right text-sm font-semibold tabular-nums', isNegativeValue(rawValue) ? 'text-red-800' : 'text-slate-800')} title={displayText}>
                                    {node}
                                </dd>
                            </div>
                        );
                    })}
                </dl>}
            </div>
        </div>
    );
}

/**
 * スマホ幅のグループ見出し1個分。集計3行を1行に集約し、詳細はタップで展開する。
 * 折り畳み時: 年月・件数・主要集計（受取額/税引後）のみで min-h 44px に収める。
 * 背景は slate-700（白カードとのコントラスト約10:1）で境界を明確にする。
 * PC幅のテーブル表示には一切触らない。
 */
function MobileReceiptGroup<T extends DataItem, S extends SummaryItem>({ summaryItem, groupItems, headerText, columns, summaryColumns, primaryKey, fields }: {
    summaryItem: S;
    groupItems: T[];
    headerText: string;
    columns: TableColumnConfig[];
    summaryColumns: SummaryColumnConfig[];
    primaryKey: string;
    fields: MobileSummaryFields;
}) {
    const [expanded, setExpanded] = useState(false);
    const id = useId();
    const itemCount = groupItems.length;

    const summaryValues = summaryColumns.map(column => ({
        key: column.key,
        label: resolveSummaryLabel(column.key, columns),
        value: formatSummaryValue(summaryItem[column.key], column),
        rawValue: summaryItem[column.key]
    }));

    if (summaryValues.length === 0) {
        return (
            <section data-testid="receipt-card-group">
                <div className="flex min-h-[44px] items-center rounded-lg bg-slate-700 px-3 py-2">
                    <span className="text-sm font-semibold text-white">{headerText}</span>
                    <span className="ml-2 inline-flex items-center rounded bg-white/20 px-2 py-0.5 text-xs font-medium text-white">
                        {itemCount}件
                    </span>
                </div>
                <div className="mt-2 space-y-2">
                    {groupItems.map((item, itemIndex) =>
                        <ReceiptCard key={String(item.id ?? `card-item-${itemIndex}`)} item={item} columns={columns} fields={fields} />
                    )}
                </div>
            </section>
        );
    }

    const primaryIndex = resolvePrimarySummaryIndex(summaryColumns, primaryKey);
    const primary = summaryValues[primaryIndex] ?? summaryValues[summaryValues.length - 1];
    if (!primary) {
        return null;
    }
    const primaryText = React.isValidElement(primary.value) ? '' : String(toText(primary.value));

    return (
        <section data-testid="receipt-card-group">
            <div className="overflow-hidden rounded-lg border border-slate-700">
                <button
                    id={`${id}-group-button`}
                    type="button"
                    aria-label={`${headerText} ${itemCount}件 ${primary.label} ${primaryText}`}
                    aria-expanded={expanded}
                    aria-controls={`${id}-group-details`}
                    onClick={() => setExpanded(previous => !previous)}
                    className="flex min-h-[44px] w-full items-center gap-2 bg-slate-700 px-3 py-2 text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
                >
                    <span className="min-w-0 flex-1 truncate text-sm font-semibold text-white">
                        {headerText}
                        <span className="ml-2 inline-flex items-center rounded bg-white/20 px-2 py-0.5 text-xs font-medium text-white">
                            {itemCount}件
                        </span>
                    </span>
                    <span className="flex shrink-0 items-baseline gap-1 whitespace-nowrap" aria-hidden="true">
                        <span className="text-xs text-slate-200">{primary.label}</span>
                        <span className="font-mono text-sm font-semibold tabular-nums text-white">
                            {React.isValidElement(primary.value) ? primary.value : toText(primary.value)}
                        </span>
                        <svg className={cn('h-4 w-4 shrink-0 self-center text-slate-200', expanded && 'rotate-180')} viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
                            <path d="m6 9 6 6 6-6" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                        </svg>
                    </span>
                </button>
                <div
                    id={`${id}-group-details`}
                    role="region"
                    aria-labelledby={`${id}-group-button`}
                    hidden={!expanded}
                    className="border-t border-slate-200 bg-white px-3 py-1"
                >
                    {expanded && (
                        <dl>
                            {summaryValues.map((sv) => (
                                <div key={sv.key} className="flex items-start justify-between gap-3 border-b border-slate-100 py-1.5 last:border-b-0">
                                    <dt className="shrink-0 pt-0.5 text-xs text-slate-600">{sv.label}</dt>
                                    <dd
                                        className={cn('min-w-0 break-words text-right text-sm font-semibold tabular-nums', !React.isValidElement(sv.value) && isNegativeValue(sv.rawValue) ? 'text-red-800' : 'text-slate-800')}
                                        data-negative={!React.isValidElement(sv.value) && isNegativeValue(sv.rawValue) ? 'true' : undefined}
                                    >
                                        {React.isValidElement(sv.value) ? sv.value : toText(sv.value)}
                                    </dd>
                                </div>
                            ))}
                        </dl>
                    )}
                </div>
            </div>
            <div className="mt-2 space-y-2">
                {groupItems.map((item, itemIndex) =>
                    <ReceiptCard key={String(item.id ?? `card-item-${itemIndex}`)} item={item} columns={columns} fields={fields} />
                )}
            </div>
        </section>
    );
}

function renderGroupedCards<T extends DataItem, S extends SummaryItem>(
    summaryToRender: S[],
    groupedDataMap: Map<string, T[]>,
    columns: TableColumnConfig[],
    summaryColumns: SummaryColumnConfig[],
    formatGroupHeader: (key: string) => string,
    fields: MobileSummaryFields
) {
    return summaryToRender.map((summaryItem, summaryIndex) => {
        const groupItems = groupedDataMap.get(summaryItem.filter) ?? [];
        const headerText = formatGroupHeader(summaryItem.filter);

        return (
            <MobileReceiptGroup
                key={`card-group-${summaryIndex}`}
                summaryItem={summaryItem}
                groupItems={groupItems}
                headerText={headerText}
                columns={columns}
                summaryColumns={summaryColumns}
                primaryKey={fields.primaryKey}
                fields={fields}
            />
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
    primaryKey,
    nameKey,
    dateKey,
    accountKey,
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
                        {renderGroupedCards(summaryToRender, groupedDataMap, columns, summaryColumns, formatGroupHeader, { primaryKey, nameKey, dateKey, accountKey })}
                    </div>
                ) : (
                    <div className="space-y-2">
                        {data.map((item, itemIndex) =>
                            <ReceiptCard key={String(item.id ?? `card-item-${itemIndex}`)} item={item} columns={columns} fields={{ primaryKey, nameKey, dateKey, accountKey }} />
                        )}
                    </div>
                )}
            </div>
        </>
    );
}
