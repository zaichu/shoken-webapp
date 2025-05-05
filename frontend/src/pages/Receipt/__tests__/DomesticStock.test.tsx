import React from 'react';
import { render, screen } from '@testing-library/react';
import { DomesticStock } from '../DomesticStock';
import * as dataTransformer from '@/lib/utils/dataTransformer';

jest.mock('@/components/templates', () => ({
    ReceiptTemplate: ({ children, title, header, searchQuery, onSearch, searchOptions }) => (
        <div data-testid="receipt-template">
            <div data-testid="title">{title}</div>
            <div data-testid="header">{header}</div>
            <div data-testid="search-query">{searchQuery}</div>
            <div data-testid="search-options">
                {JSON.stringify(searchOptions)}
            </div>
            <div data-testid="content">{children}</div>
        </div>
    )
}));

jest.mock('@/components/molecules', () => ({
    ReceiptHeader: ({ items }) => (
        <div data-testid="receipt-header">
            {items.map((item, index) => (
                <div key={index} data-testid={`header-item-${index}`}>
                    <div data-testid={`title-${index}`}>{item.title}</div>
                    <div data-testid={`value-${index}`}>{item.format(item.value)}</div>
                </div>
            ))}
        </div>
    )
}));

jest.mock('@/components/organisms', () => ({
    ReceiptTable: ({ data, summary, columns, summaryColumns, getGroupKey }) => (
        <div data-testid="receipt-table">
            <div data-testid="data">{JSON.stringify(data)}</div>
            <div data-testid="summary">{JSON.stringify(summary)}</div>
            <div data-testid="columns">{JSON.stringify(columns)}</div>
            <div data-testid="summary-columns">{JSON.stringify(summaryColumns)}</div>
        </div>
    )
}));

jest.mock('@/lib/utils/dataTransformer', () => ({
    createSearchOptions: jest.fn(),
    filterDataBySearchQuery: jest.fn(),
    groupAndSummarizeData: jest.fn()
}));

describe('DomesticStock', () => {
    // 同じ日付の取引を含むテストデータ
    const mockCsvData = [
        {
            '約定日': '2023/01/15',
            '受渡日': '2023/01/18',
            '銘柄コード': 1234,
            '銘柄名': 'テスト株式1',
            '口座': '特定口座',
            '数量[株]': 100,
            '売却/決済単価': '1000',
            '売却/決済額': '100000',
            '平均取得価額': '900',
            '実現損益': '10000'
        },
        {
            '約定日': '2023/01/15', // 同日
            '受渡日': '2023/01/18',
            '銘柄コード': 5678,
            '銘柄名': 'テスト株式2',
            '口座': '特定口座',
            '数量[株]': 200,
            '売却/決済単価': '500',
            '売却/決済額': '100000',
            '平均取得価額': '450',
            '実現損益': '10000'
        },
        {
            '約定日': '2023/02/15',
            '受渡日': '2023/02/18',
            '銘柄コード': 9012,
            '銘柄名': 'テスト株式3',
            '口座': '特定口座',
            '数量[株]': 300,
            '売却/決済単価': '800',
            '売却/決済額': '240000',
            '平均取得価額': '700',
            '実現損益': '30000'
        }
    ];

    beforeEach(() => {
        // モックをリセット
        (dataTransformer.createSearchOptions as jest.Mock).mockReset();
        (dataTransformer.filterDataBySearchQuery as jest.Mock).mockReset();
        (dataTransformer.groupAndSummarizeData as jest.Mock).mockReset();

        // モックの初期化
        (dataTransformer.createSearchOptions as jest.Mock).mockReturnValue([
            { value: '1234', label: '1234:テスト株式1' },
            { value: '5678', label: '5678:テスト株式2' },
            { value: '9012', label: '9012:テスト株式3' }
        ]);

        // 日別で処理された結果
        const mockDomesticStockData = [
            {
                settlement_date: new Date('2023-01-15'),
                delivery_date: new Date('2023-01-18'),
                product: '国内株式',
                account: '特定口座',
                security_code: '1234',
                security_name: 'テスト株式1',
                quantity: 100,
                unit_price: 1000,
                amount: 100000,
                average_cost: 900,
                realized_pl: 10000,
                tax: 2031, // 日別合計20000*0.20315の半分
                realized_pl_after_tax: 7969
            },
            {
                settlement_date: new Date('2023-01-15'),
                delivery_date: new Date('2023-01-18'),
                product: '国内株式',
                account: '特定口座',
                security_code: '5678',
                security_name: 'テスト株式2',
                quantity: 200,
                unit_price: 500,
                amount: 100000,
                average_cost: 450,
                realized_pl: 10000,
                tax: 2032, // 日別合計20000*0.20315の半分（端数調整）
                realized_pl_after_tax: 7968
            },
            {
                settlement_date: new Date('2023-02-15'),
                delivery_date: new Date('2023-02-18'),
                product: '国内株式',
                account: '特定口座',
                security_code: '9012',
                security_name: 'テスト株式3',
                quantity: 300,
                unit_price: 800,
                amount: 240000,
                average_cost: 700,
                realized_pl: 30000,
                tax: 6095, // 30000*0.20315
                realized_pl_after_tax: 23905
            }
        ];
        
        (dataTransformer.filterDataBySearchQuery as jest.Mock).mockReturnValue(mockDomesticStockData);
        
        // 月次集計結果
        (dataTransformer.groupAndSummarizeData as jest.Mock).mockReturnValue([
            {
                filter: '2023-01',
                amount: 200000,
                realized_pl: 20000,
                tax: 4063,
                realized_pl_after_tax: 15937
            },
            {
                filter: '2023-02',
                amount: 240000,
                realized_pl: 30000,
                tax: 6095,
                realized_pl_after_tax: 23905
            }
        ]);
    });

    it('正しいタイトルとヘッダーアイテムが表示される', () => {
        render(<DomesticStock csvData={mockCsvData} />);
        
        expect(screen.getByTestId('title')).toHaveTextContent('国内株式実現損益');
        
        expect(screen.getByTestId('title-0')).toHaveTextContent('取引金額合計');
        expect(screen.getByTestId('value-0')).toHaveTextContent('¥440,000');
        
        expect(screen.getByTestId('title-1')).toHaveTextContent('実現損益合計');
        expect(screen.getByTestId('value-1')).toHaveTextContent('¥50,000');
        
        expect(screen.getByTestId('title-2')).toHaveTextContent('税額合計');
        expect(screen.getByTestId('value-2')).toHaveTextContent('¥10,158');
        
        expect(screen.getByTestId('title-3')).toHaveTextContent('実現損益(税引)合計');
        expect(screen.getByTestId('value-3')).toHaveTextContent('¥39,842');
    });

    it('テーブルに正しいデータと設定が渡される', () => {
        render(<DomesticStock csvData={mockCsvData} />);
        
        const columnsData = JSON.parse(screen.getByTestId('columns').textContent || '[]');
        const summaryColumnsData = JSON.parse(screen.getByTestId('summary-columns').textContent || '[]');
        
        // カラム設定の検証
        expect(columnsData).toHaveLength(10);
        expect(columnsData[0].key).toBe('settlement_date');
        expect(columnsData[0].header).toBe('約定日');
        
        expect(columnsData[1].key).toBe('delivery_date');
        expect(columnsData[1].header).toBe('受渡日');
        
        expect(columnsData[3].key).toBe('security_name');
        expect(columnsData[3].header).toBe('銘柄名');
        expect(columnsData[3].width).toBe('250px');
        
        // 集計カラム設定の検証
        expect(summaryColumnsData).toHaveLength(3);
        expect(summaryColumnsData[0].key).toBe('realized_pl');
        expect(summaryColumnsData[0].colSpan).toBe(10);
        expect(summaryColumnsData[0].textAlign).toBe('right');
        
        expect(summaryColumnsData[1].key).toBe('tax');
        expect(summaryColumnsData[2].key).toBe('realized_pl_after_tax');
    });

    it('ユーティリティ関数が正しく呼び出される', () => {
        render(<DomesticStock csvData={mockCsvData} />);
        
        // createSearchOptionsが正しく呼び出されることを確認
        expect(dataTransformer.createSearchOptions).toHaveBeenCalledWith(
            expect.any(Array),
            'security_code',
            'security_name',
            true
        );
        
        // filterDataBySearchQueryが正しく呼び出されることを確認
        expect(dataTransformer.filterDataBySearchQuery).toHaveBeenCalledWith(
            expect.any(Array),
            '',
            ['security_code', 'security_name']
        );
        
        // groupAndSummarizeDataが正しく呼び出されることを確認
        expect(dataTransformer.groupAndSummarizeData).toHaveBeenCalledWith(
            expect.any(Array),
            expect.any(Function),
            ['amount', 'realized_pl', 'tax', 'realized_pl_after_tax'],
            ''
        );
    });

    // 日別税金計算のテスト
    it('日付ごとに税金が計算される', () => {
        // createSearchOptionsのモックを一時的に無効化して実際のデータを使用
        (dataTransformer.createSearchOptions as jest.Mock).mockImplementation(
            (data, valueField, labelField, prefix) => {
                return data.map(item => ({
                    value: item[valueField] || '',
                    label: prefix ? `${item[valueField]}:${item[labelField]}` : item[labelField]
                }));
            }
        );
        
        // filterDataBySearchQueryのモックを無効化
        (dataTransformer.filterDataBySearchQuery as jest.Mock).mockImplementation(
            (data, query, fields) => data
        );
        
        // groupAndSummarizeDataのモックを無効化
        (dataTransformer.groupAndSummarizeData as jest.Mock).mockImplementation(
            (data, groupByFn, sumFields, searchQuery) => {
                const groups = new Map();
                
                data.forEach(item => {
                    const key = groupByFn(item);
                    if (!groups.has(key)) {
                        const group = { filter: key };
                        sumFields.forEach(field => group[field] = 0);
                        groups.set(key, group);
                    }
                    
                    const group = groups.get(key);
                    sumFields.forEach(field => {
                        group[field] += item[field] || 0;
                    });
                });
                
                return Array.from(groups.values());
            }
        );
        
        const { container } = render(<DomesticStock csvData={mockCsvData} />);
        
        // データが日付ごとに処理されていることを確認
        const dataContent = screen.getByTestId('data').textContent;
        const data = JSON.parse(dataContent || '[]');
        
        // 同じ日の取引の税金が日別で計算されていることを確認
        const jan15Items = data.filter(item => 
            new Date(item.settlement_date).toISOString().split('T')[0] === '2023-01-15'
        );
        
        expect(jan15Items).toHaveLength(2);
        
        // 同日の取引の税金合計が正しいか確認
        const jan15TaxSum = jan15Items.reduce((sum, item) => sum + item.tax, 0);
        const jan15ExpectedTax = Math.floor(20000 * 0.20315); // 一日の合計利益に対する税金
        
        expect(jan15TaxSum).toBeCloseTo(jan15ExpectedTax, 0);
        
        // サマリーの税金が正しく集計されていることを確認
        const summaryContent = screen.getByTestId('summary').textContent;
        const summary = JSON.parse(summaryContent || '[]');
        
        // 月次集計の検証
        expect(summary.length).toBeGreaterThan(0);
        
        // 税引後の利益が正しく計算されていることを確認（実現損益 - 税金）
        summary.forEach(item => {
            expect(item.realized_pl_after_tax).toBeCloseTo(item.realized_pl - item.tax, 0);
        });
    });
});
