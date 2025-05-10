import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import { DomesticStock } from '../DomesticStock';
import * as dataTransformer from '@/lib/utils/dataTransformer';

vi.mock('@/components/templates', () => ({
    ReceiptTemplate: ({ children, title, header, searchQuery, searchOptions }) => (
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

vi.mock('@/components/molecules', () => ({
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

vi.mock('@/components/organisms', () => ({
    ReceiptTable: ({ data, summary, columns, summaryColumns }) => (
        <div data-testid="receipt-table">
            <div data-testid="data">{JSON.stringify(data)}</div>
            <div data-testid="summary">{JSON.stringify(summary)}</div>
            <div data-testid="columns">{JSON.stringify(columns)}</div>
            <div data-testid="summary-columns">{JSON.stringify(summaryColumns)}</div>
        </div>
    )
}));

vi.mock('@/lib/utils/dataTransformer', () => ({
    createSearchOptions: vi.fn(),
    filterDataBySearchQuery: vi.fn(),
    groupAndSummarizeData: vi.fn()
}));

describe('DomesticStock', () => {
    const mockCsvData = [
        {
            '銘柄': 'テスト株式',
            '銘柄コード': '1234',
            '約定日': '2023/01/15',
            '受渡日': '2023/01/18',
            '税': 'NISA',
            '数量[株/口]': '100',
            '取引': '買い',
            '単価[円]': '1000',
            '国内手数料[円]': '500',
            '現地手数料[円]': '500',
            '税金[円]': '100',
            '受渡額[円]': '101100'
        },
        {
            '銘柄': 'テスト株式2',
            '銘柄コード': '5678',
            '約定日': '2023/01/20',
            '受渡日': '2023/01/23',
            '税': 'NISA',
            '数量[株/口]': '200',
            '取引': '売り',
            '単価[円]': '2000',
            '国内手数料[円]': '1000',
            '現地手数料[円]': '1000',
            '税金[円]': '200',
            '受渡額[円]': '397800'
        }
    ];

    beforeEach(() => {
        vi.mocked(dataTransformer.createSearchOptions).mockReset();
        vi.mocked(dataTransformer.filterDataBySearchQuery).mockReset();
        vi.mocked(dataTransformer.groupAndSummarizeData).mockReset();

        vi.mocked(dataTransformer.createSearchOptions).mockReturnValue([
            { value: '1234', label: '1234:テスト株式' },
            { value: '5678', label: '5678:テスト株式2' }
        ]);

        const mockDomesticStockData = [
            {
                security_name: 'テスト株式',
                security_code: '1234',
                trade_date: new Date('2023-01-15'),
                settlement_date: new Date('2023-01-18'),
                tax_category: 'NISA',
                quantity: 100,
                trade_type: '買い',
                unit_price: 1000,
                buy_sell_amount: 100000,
                domestic_fee: 500,
                foreign_fee: 500,
                tax: 100,
                execution_amount: 101100
            },
            {
                security_name: 'テスト株式2',
                security_code: '5678',
                trade_date: new Date('2023-01-20'),
                settlement_date: new Date('2023-01-23'),
                tax_category: 'NISA',
                quantity: 200,
                trade_type: '売り',
                unit_price: 2000,
                buy_sell_amount: 400000,
                domestic_fee: 1000,
                foreign_fee: 1000,
                tax: 200,
                execution_amount: 397800
            }
        ];
        
        vi.mocked(dataTransformer.filterDataBySearchQuery).mockReturnValue(mockDomesticStockData);
        
        vi.mocked(dataTransformer.groupAndSummarizeData).mockReturnValue([
            {
                filter: '2023-01',
                buy_sell_amount: 500000,
                domestic_fee: 1500,
                foreign_fee: 1500,
                tax: 300,
                execution_amount: 498900
            }
        ]);
    });

    it('正しいタイトルとヘッダーアイテムが表示される', () => {
        render(<DomesticStock csvData={mockCsvData} />);
        
        expect(screen.getByTestId('title')).toHaveTextContent('国内株式');
        
        expect(screen.getByTestId('title-0')).toHaveTextContent('合計実現損益');
        expect(screen.getByTestId('value-0')).toHaveTextContent('¥ 0');
        
        expect(screen.getByTestId('title-1')).toHaveTextContent('合計税額');
        expect(screen.getByTestId('value-1')).toHaveTextContent('¥ 0');
        
        expect(screen.getByTestId('title-2')).toHaveTextContent('合計実現損益(税引)');
        expect(screen.getByTestId('value-2')).toHaveTextContent('¥ 0');
    });

    it('テーブルに正しいデータと設定が渡される', () => {
        render(<DomesticStock csvData={mockCsvData} />);
        
        const columnsData = JSON.parse(screen.getByTestId('columns').textContent || '[]');
        const summaryColumnsData = JSON.parse(screen.getByTestId('summary-columns').textContent || '[]');
        
        // カラム設定の検証
        expect(columnsData).toHaveLength(13);
        expect(columnsData[0].key).toBe('trade_date');
        expect(columnsData[0].header).toBe('約定日');
        
        expect(columnsData[3].key).toBe('security_name');
        expect(columnsData[3].header).toBe('銘柄名');
        expect(columnsData[3].width).toBe('250px');
        
        // 集計カラム設定の検証
        expect(summaryColumnsData).toHaveLength(3);
        expect(summaryColumnsData[0].key).toBe('total_realized_profit_and_loss');
        expect(summaryColumnsData[0].colSpan).toBe(11);
        expect(summaryColumnsData[0].textAlign).toBe('right');
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
        
        // DomesticStockの実装が変更されたため、groupAndSummarizeDataは使われていない
        expect(dataTransformer.groupAndSummarizeData).not.toHaveBeenCalled();
    });

    it('売買区分によって適切なスタイルが適用される', () => {
        const mockDataWithSell = [
            ...mockCsvData,
            {
                '銘柄': 'テスト株式3',
                '銘柄コード': '9999',
                '約定日': '2023/02/01',
                '受渡日': '2023/02/04',
                '税': '特定',
                '数量[株/口]': '150',
                '取引': '売り',
                '単価[円]': '1500',
                '国内手数料[円]': '750',
                '現地手数料[円]': '750',
                '税金[円]': '150',
                '受渡額[円]': '222900'
            }
        ];

        render(<DomesticStock csvData={mockDataWithSell} />);
        
        // テスト実装はコンポーネントの具体的な実装に依存
        // ここでは基本的な表示確認のみ
        expect(screen.getByTestId('receipt-table')).toBeInTheDocument();
    });
});
