import { describe, it, expect } from 'vitest';
import {
    filterByConfig,
    FilterConfig,
    createYearOptions,
    getUniqueValues,
    matchesYear,
    matchesYearMonth,
} from '../searchUtils';

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

    describe('dateField（日付検索）', () => {
        it('yearSearchで年度検索できる', () => {
            const config: FilterConfig<TestItem> = {
                dateField: item => item.date,
                yearSearch: true,
            };
            const result = filterByConfig(testData, '2023', config);
            expect(result).toHaveLength(2);
        });

        it('yearMonthSearchで年月検索できる', () => {
            const config: FilterConfig<TestItem> = {
                dateField: item => item.date,
                yearMonthSearch: true,
            };
            const result = filterByConfig(testData, '2023-01', config);
            expect(result).toHaveLength(1);
            expect(result[0].code).toBe('1234');
        });

        it('dateSearchで日付検索できる', () => {
            const config: FilterConfig<TestItem> = {
                dateField: item => item.date,
                dateSearch: true,
            };
            const result = filterByConfig(testData, '2023-02-20', config);
            expect(result).toHaveLength(1);
            expect(result[0].code).toBe('5678');
        });
    });

    describe('amountFields（金額検索）', () => {
        it('金額の部分一致で検索できる', () => {
            const config: FilterConfig<TestItem> = {
                amountFields: [item => item.amount],
            };
            // 100, 200 contains '00'
            const result = filterByConfig(testData, '00', config);
            expect(result).toHaveLength(3);
        });

        it('金額の完全一致でも検索できる', () => {
            const config: FilterConfig<TestItem> = {
                amountFields: [item => item.amount],
            };
            const result = filterByConfig(testData, '100', config);
            expect(result).toHaveLength(1);
            expect(result[0].amount).toBe(100);
        });
    });
});

describe('createYearOptions', () => {
    it('重複排除した年オプションを年昇順で返す', () => {
        const data = [
            { date: new Date('2024-03-01') },
            { date: new Date('2023-12-15') },
            { date: new Date('2024-07-01') },
        ];

        const options = createYearOptions(data, item => item.date);

        expect(options).toEqual([
            { value: '2023', label: '2023年' },
            { value: '2024', label: '2024年' },
        ]);
    });

    it('空配列の場合は空を返す', () => {
        expect(createYearOptions([], () => new Date())).toEqual([]);
    });
});

describe('getUniqueValues', () => {
    it('重複と空文字を除いたユニーク値を返す', () => {
        const data = [
            { category: '特定' },
            { category: 'NISA' },
            { category: '特定' },
            { category: '' },
        ];

        const values = getUniqueValues(data, item => item.category);

        expect(values).toEqual(['特定', 'NISA']);
    });

    it('空配列の場合は空を返す', () => {
        expect(getUniqueValues([], (item: { v: string }) => item.v)).toEqual([]);
    });
});

describe('matchesYear', () => {
    it('年が一致する場合 true を返す', () => {
        expect(matchesYear(new Date('2024-06-15'), '2024')).toBe(true);
    });

    it('年が一致しない場合 false を返す', () => {
        expect(matchesYear(new Date('2024-06-15'), '2023')).toBe(false);
    });
});

describe('matchesYearMonth', () => {
    it('年月が YYYY-MM 形式で一致する場合 true を返す', () => {
        expect(matchesYearMonth(new Date('2024-03-15'), '2024-03')).toBe(true);
    });

    it('1桁の月はゼロ埋めして比較する', () => {
        expect(matchesYearMonth(new Date('2024-01-05'), '2024-01')).toBe(true);
        expect(matchesYearMonth(new Date('2024-01-05'), '2024-1')).toBe(false);
    });

    it('年月が一致しない場合 false を返す', () => {
        expect(matchesYearMonth(new Date('2024-03-15'), '2024-04')).toBe(false);
    });
});
