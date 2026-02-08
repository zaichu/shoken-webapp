import { TableColumnConfig } from '@/lib/interfaces/receipt';

/**
 * 列の前面配置ルール
 */
export interface ColumnReorderRule<T> {
    /** 前面に配置する列のkey */
    columnKey: string;
    /** データがこのルールに該当するか判定する関数 */
    match: (item: T, query: string) => boolean;
}

/**
 * 検索クエリに応じて特定の列を前面に配置する
 * @param baseColumns ベースの列定義
 * @param data フィルタ済みデータ
 * @param searchQuery 検索クエリ
 * @param rules 前面配置ルールの配列（優先度順）
 * @param fixedColumnCount 先頭の固定列数（この列の後ろに移動列を挿入）
 */
export function reorderColumnsBySearch<T>(
    baseColumns: TableColumnConfig[],
    data: T[],
    searchQuery: string,
    rules: ColumnReorderRule<T>[],
    fixedColumnCount: number
): TableColumnConfig[] {
    if (!searchQuery) return baseColumns;

    const query = searchQuery.toLowerCase();

    for (const rule of rules) {
        if (data.some(item => rule.match(item, query))) {
            const targetCol = baseColumns.find(col => col.key === rule.columnKey);
            if (!targetCol) continue;

            const fixedCols = baseColumns.slice(0, fixedColumnCount);
            const otherCols = baseColumns.filter(col => col.key !== rule.columnKey);
            return [...fixedCols, targetCol, ...otherCols.slice(fixedColumnCount)];
        }
    }

    return baseColumns;
}
