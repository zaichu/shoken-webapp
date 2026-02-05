import { describe, it, expect } from 'vitest';
import { filterByConfig, FilterConfig } from '../searchUtils';

interface TestItem {
    code: string;
    name: string;
    category: string;
    date: Date;
    amount: number;
}

const testData: TestItem[] = [
    { code: '1234', name: 'テスト商品1', category: 'A', date: new Date('2023-01-15'), amount: 100 },
    { code: '5678', name: 'サンプル品', category: 'B', date: new Date('2023-02-20'), amount: 200 },
    { code: '9012', name: 'テスト商品2', category: 'A', date: new Date('2024-01-10'), amount: 300 },
];

describe('filterByConfig', () => {
    describe('stringFields（完全一致）', () => {
        it('完全一致する場合にマッチする', () => {
            const config: FilterConfig<TestItem> = {
                stringFields: [item => item.code],
            };
            const result = filterByConfig(testData, '1234', config);
            expect(result).toHaveLength(1);
            expect(result[0].code).toBe('1234');
        });

        it('部分一致ではマッチしない', () => {
            const config: FilterConfig<TestItem> = {
                stringFields: [item => item.code],
            };
            const result = filterByConfig(testData, '123', config);
            expect(result).toHaveLength(0);
        });
    });

    describe('partialStringFields（部分一致）', () => {
        it('部分一致する場合にマッチする', () => {
            const config: FilterConfig<TestItem> = {
                partialStringFields: [item => item.name],
            };
            const result = filterByConfig(testData, 'テスト', config);
            expect(result).toHaveLength(2);
        });

        it('大文字小文字を区別しない', () => {
            const config: FilterConfig<TestItem> = {
                partialStringFields: [item => item.category],
            };
            const result = filterByConfig(testData, 'a', config);
            expect(result).toHaveLength(2);
        });

        it('複数フィールドで検索できる', () => {
            const config: FilterConfig<TestItem> = {
                partialStringFields: [
                    item => item.code,
                    item => item.name,
                ],
            };
            // コード「12」を含む商品
            const result = filterByConfig(testData, '12', config);
            expect(result).toHaveLength(2); // 1234とテスト商品1・2
        });
    });

    describe('複合条件', () => {
        it('stringFieldsとpartialStringFieldsを組み合わせられる', () => {
            const config: FilterConfig<TestItem> = {
                stringFields: [item => item.code],
                partialStringFields: [item => item.name],
            };
            // コード完全一致 または 名前部分一致
            const result = filterByConfig(testData, 'テスト', config);
            expect(result).toHaveLength(2); // テスト商品1, テスト商品2
        });

        it('クエリが空の場合は全データを返す', () => {
            const config: FilterConfig<TestItem> = {
                partialStringFields: [item => item.name],
            };
            const result = filterByConfig(testData, '', config);
            expect(result).toHaveLength(3);
        });
    });
});
