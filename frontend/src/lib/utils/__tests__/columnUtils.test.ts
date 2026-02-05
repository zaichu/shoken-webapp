import { describe, it, expect } from 'vitest';
import { reorderColumnsBySearch, ColumnReorderRule } from '../columnUtils';
import { TableColumnConfig } from '@/lib/interfaces/receipt';

interface TestItem {
    id: number;
    name: string;
    category: string;
    account: string;
}

const baseColumns: TableColumnConfig[] = [
    { key: 'date', header: '日付' },
    { key: 'code', header: 'コード' },
    { key: 'name', header: '名前' },
    { key: 'category', header: 'カテゴリ' },
    { key: 'account', header: '口座' },
    { key: 'amount', header: '金額' },
];

const testData: TestItem[] = [
    { id: 1, name: 'テスト1', category: '商品A', account: '特定口座' },
    { id: 2, name: 'テスト2', category: '商品B', account: 'NISA' },
];

describe('columnUtils', () => {
    describe('reorderColumnsBySearch', () => {
        it('検索クエリがない場合はbaseColumnsをそのまま返す', () => {
            const rules: ColumnReorderRule<TestItem>[] = [
                { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
            ];

            const result = reorderColumnsBySearch(baseColumns, testData, '', rules, 1);
            expect(result).toEqual(baseColumns);
        });

        it('マッチするルールがある場合は該当列を前面に配置する', () => {
            const rules: ColumnReorderRule<TestItem>[] = [
                { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
            ];

            const result = reorderColumnsBySearch(baseColumns, testData, '特定', rules, 1);

            // date(固定) + account(移動) + code, name, category, amount
            expect(result[0].key).toBe('date');
            expect(result[1].key).toBe('account');
            expect(result[2].key).toBe('code');
        });

        it('fixedColumnCountに応じて固定列数を変更できる', () => {
            const rules: ColumnReorderRule<TestItem>[] = [
                { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
            ];

            const result = reorderColumnsBySearch(baseColumns, testData, '特定', rules, 2);

            // date, code(固定2列) + account(移動) + name, category, amount
            expect(result[0].key).toBe('date');
            expect(result[1].key).toBe('code');
            expect(result[2].key).toBe('account');
            expect(result[3].key).toBe('name');
        });

        it('複数ルールがある場合は優先度順に評価される', () => {
            const rules: ColumnReorderRule<TestItem>[] = [
                { columnKey: 'category', match: (item, q) => item.category.toLowerCase().includes(q) },
                { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
            ];

            // 商品Aにマッチ → categoryが前面に
            const result = reorderColumnsBySearch(baseColumns, testData, '商品a', rules, 1);
            expect(result[1].key).toBe('category');
        });

        it('マッチするデータがない場合はbaseColumnsを返す', () => {
            const rules: ColumnReorderRule<TestItem>[] = [
                { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
            ];

            const result = reorderColumnsBySearch(baseColumns, testData, '存在しない', rules, 1);
            expect(result).toEqual(baseColumns);
        });

        it('存在しないcolumnKeyの場合はスキップして次のルールを評価', () => {
            const rules: ColumnReorderRule<TestItem>[] = [
                { columnKey: 'nonexistent', match: () => true },
                { columnKey: 'account', match: (item, q) => item.account.toLowerCase().includes(q) },
            ];

            const result = reorderColumnsBySearch(baseColumns, testData, '特定', rules, 1);
            expect(result[1].key).toBe('account');
        });
    });
});
