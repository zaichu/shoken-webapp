import { createSearchOptions, groupAndSummarizeData } from '../dataTransformer';

interface TestItemWithCode {
    code: string;
    name: string;
    value?: number;
}

interface TestItemWithCodeAndDate {
    code: string;
    name: string;
    date: Date | string;
}

interface TestItemWithDate {
    date: Date;
    category: string;
    amount: number;
    tax: number;
}

describe('createSearchOptions', () => {
    it('正しい検索オプションを生成する', () => {
        const testData: TestItemWithCode[] = [
            { code: '1234', name: 'テスト1' },
            { code: '5678', name: 'テスト2' },
            { code: '', name: 'テスト3' }
        ];

        const result = createSearchOptions(testData, 'code', 'name', true);
        
        expect(result).toEqual([
            { value: '1234', label: '1234: テスト1' },
            { value: '5678', label: '5678: テスト2' },
            { value: 'テスト3', label: 'テスト3' }
        ]);
    });

    it('プレフィックスなしで正しい検索オプションを生成する', () => {
        const testData: TestItemWithCode[] = [
            { code: '1234', name: 'テスト1' },
            { code: '5678', name: 'テスト2' }
        ];

        const result = createSearchOptions(testData, 'code', 'name', false);
        
        expect(result).toEqual([
            { value: '1234', label: 'テスト1' },
            { value: '5678', label: 'テスト2' }
        ]);
    });

    it('重複した値をフィルタリングする', () => {
        const testData: TestItemWithCode[] = [
            { code: '1234', name: 'テスト1' },
            { code: '1234', name: 'テスト1' },
            { code: '5678', name: 'テスト2' }
        ];

        const result = createSearchOptions(testData, 'code', 'name', true);

        expect(result).toEqual([
            { value: '1234', label: '1234: テスト1' },
            { value: '5678', label: '5678: テスト2' }
        ]);
    });

    it('dateFieldを指定すると同一valueで最新日付のlabelを採用する（古い名称が先）', () => {
        const testData: TestItemWithCodeAndDate[] = [
            { code: '9432', name: '日本電信電話', date: new Date('2024-06-21') },
            { code: '9432', name: 'ＮＴＴ', date: new Date('2026-06-01') },
            { code: '1234', name: 'テスト', date: new Date('2023-01-01') },
        ];

        const result = createSearchOptions(testData, 'code', 'name', true, 'date');

        expect(result).toEqual([
            { value: '1234', label: '1234: テスト' },
            { value: '9432', label: '9432: ＮＴＴ' },
        ]);
    });

    it('dateFieldを指定すると同一valueで最新日付のlabelを採用する（新しい名称が先）', () => {
        const testData: TestItemWithCodeAndDate[] = [
            { code: '9432', name: 'ＮＴＴ', date: new Date('2026-06-01') },
            { code: '9432', name: '日本電信電話', date: new Date('2024-06-21') },
            { code: '1234', name: 'テスト', date: new Date('2023-01-01') },
        ];

        const result = createSearchOptions(testData, 'code', 'name', true, 'date');

        expect(result).toEqual([
            { value: '1234', label: '1234: テスト' },
            { value: '9432', label: '9432: ＮＴＴ' },
        ]);
    });

    it('dateFieldを日付文字列で指定しても最新日付のlabelを採用する', () => {
        const testData: TestItemWithCodeAndDate[] = [
            { code: '9432', name: '日本電信電話', date: '2024-06-21' },
            { code: '9432', name: 'ＮＴＴ', date: '2026-06-01' },
        ];

        const result = createSearchOptions(testData, 'code', 'name', true, 'date');

        expect(result).toEqual([
            { value: '9432', label: '9432: ＮＴＴ' },
        ]);
    });

    it('dateFieldを指定しない場合は最初に出たlabelを保持する（後方互換）', () => {
        const testData: TestItemWithCodeAndDate[] = [
            { code: '9432', name: '日本電信電話', date: new Date('2024-06-21') },
            { code: '9432', name: 'ＮＴＴ', date: new Date('2026-06-01') },
        ];

        const result = createSearchOptions(testData, 'code', 'name', true);

        expect(result).toEqual([
            { value: '9432', label: '9432: 日本電信電話' },
        ]);
    });

    it('先頭itemの日付が不正文字列でも後続の有効な日付のlabelを採用する', () => {
        const testData: TestItemWithCodeAndDate[] = [
            { code: '9432', name: '日本電信電話', date: 'not-a-date' },
            { code: '9432', name: 'ＮＴＴ', date: '2026-06-01' },
        ];

        const result = createSearchOptions(testData, 'code', 'name', true, 'date');

        expect(result).toEqual([
            { value: '9432', label: '9432: ＮＴＴ' },
        ]);
    });
});

describe('groupAndSummarizeData', () => {
    const testData: TestItemWithDate[] = [
        { date: new Date('2023-01-01'), category: 'A', amount: 100, tax: 10 },
        { date: new Date('2023-01-02'), category: 'A', amount: 200, tax: 20 },
        { date: new Date('2023-02-01'), category: 'B', amount: 300, tax: 30 },
        { date: new Date('2023-02-15'), category: 'C', amount: 400, tax: 40 }
    ];

    it('日付の月でグループ化して新しい順に返す', () => {
        const groupByFn = (item: TestItemWithDate) => {
            const date = item.date;
            return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}`;
        };

        const result = groupAndSummarizeData(testData, groupByFn, ['amount', 'tax'], 'desc');
        
        // Symbol.for('key')を除外して比較
        const resultWithoutSymbol = result.map(item => {
            const symbolKey = Symbol.for('key');
            // eslint-disable-next-line @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any
            const { [symbolKey as any]: _, ...rest } = item;
            return rest;
        });
        
        expect(resultWithoutSymbol).toEqual([
            { filter: '2023-02', amount: 700, tax: 70 },
            { filter: '2023-01', amount: 300, tax: 30 }
        ]);
    });

    it('カテゴリでグループ化する', () => {
        const groupByFn = (item: TestItemWithDate) => item.category;
        const result = groupAndSummarizeData([...testData].reverse(), groupByFn, ['amount', 'tax']);
        
        // Symbol.for('key')を除外して比較
        const resultWithoutSymbol = result.map(item => {
            const symbolKey = Symbol.for('key');
            // eslint-disable-next-line @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any
            const { [symbolKey as any]: _, ...rest } = item;
            return rest;
        });
        
        expect(resultWithoutSymbol).toEqual([
            { filter: 'A', amount: 300, tax: 30 },
            { filter: 'B', amount: 300, tax: 30 },
            { filter: 'C', amount: 400, tax: 40 }
        ]);
    });

    it.each([
        ['asc', ['A', 'B', 'C']],
        ['desc', ['C', 'B', 'A']],
    ] as const)('order=%s で入力順によらずカテゴリを並べる', (order, expected) => {
        const input = [testData[2], testData[0], testData[3], testData[1]];
        const original = [...input];
        const result = groupAndSummarizeData(input, item => item.category, ['amount', 'tax'], order);

        expect(result.map(item => item.filter)).toEqual(expected);
        expect(result.find(item => item.filter === 'A')).toMatchObject({ amount: 300, tax: 30 });
        expect(input).toEqual(original);
    });

    it('検索クエリありでデータをグループ化する', () => {
        const groupByFn = () => 'search';
        const result = groupAndSummarizeData(testData, groupByFn, ['amount', 'tax']);
        
        // Symbol.for('key')を除外して比較
        const resultWithoutSymbol = result.map(item => {
            const symbolKey = Symbol.for('key');
            // eslint-disable-next-line @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any
            const { [symbolKey as any]: _, ...rest } = item;
            return rest;
        });
        
        expect(resultWithoutSymbol).toEqual([
            { filter: 'search', amount: 1000, tax: 100 }
        ]);
    });
});
