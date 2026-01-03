import { createSearchOptions, filterDataBySearchQuery, groupAndSummarizeData } from '../dataTransformer';

interface TestItemWithCode {
    code: string;
    name: string;
    value?: number;
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
});

describe('filterDataBySearchQuery', () => {
    const testData: TestItemWithCode[] = [
        { code: '1234', name: 'テスト1', value: 100 },
        { code: '5678', name: 'サンプル', value: 200 },
        { code: '9012', name: 'テスト2', value: 300 }
    ];

    it('検索クエリが空の場合はすべてのデータを返す', () => {
        const result = filterDataBySearchQuery(testData, '', ['code', 'name']);
        expect(result).toEqual(testData);
    });

    it('codeフィールドでフィルタリングする', () => {
        const result = filterDataBySearchQuery(testData, '123', ['code', 'name']);
        expect(result).toEqual([testData[0]]);
    });

    it('nameフィールドでフィルタリングする', () => {
        const result = filterDataBySearchQuery(testData, 'テスト', ['code', 'name']);
        expect(result).toEqual([testData[0], testData[2]]);
    });

    it('検索クエリに一致するデータがない場合は空の配列を返す', () => {
        const result = filterDataBySearchQuery(testData, 'ないデータ', ['code', 'name']);
        expect(result).toEqual([]);
    });
});

describe('groupAndSummarizeData', () => {
    const testData: TestItemWithDate[] = [
        { date: new Date('2023-01-01'), category: 'A', amount: 100, tax: 10 },
        { date: new Date('2023-01-02'), category: 'A', amount: 200, tax: 20 },
        { date: new Date('2023-02-01'), category: 'B', amount: 300, tax: 30 },
        { date: new Date('2023-02-15'), category: 'C', amount: 400, tax: 40 }
    ];

    it('日付の月でグループ化する', () => {
        const groupByFn = (item: TestItemWithDate) => {
            const date = item.date;
            return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}`;
        };

        const result = groupAndSummarizeData(testData, groupByFn, ['amount', 'tax']);
        
        // Symbol.for('key')を除外して比較
        const resultWithoutSymbol = result.map(item => {
            const symbolKey = Symbol.for('key');
            // eslint-disable-next-line @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any
            const { [symbolKey as any]: _, ...rest } = item;
            return rest;
        });
        
        expect(resultWithoutSymbol).toEqual([
            { filter: '2023-01', amount: 300, tax: 30 },
            { filter: '2023-02', amount: 700, tax: 70 }
        ]);
    });

    it('カテゴリでグループ化する', () => {
        const groupByFn = (item: TestItemWithDate) => item.category;
        const result = groupAndSummarizeData(testData, groupByFn, ['amount', 'tax']);
        
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
