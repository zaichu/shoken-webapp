import { render, screen } from '@testing-library/react';
import { Dividend } from '../Dividend';
import * as dataTransformer from '@/lib/utils/dataTransformer';

jest.mock('@/components/templates', () => ({
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
    ReceiptTable: ({ data, summary, columns, summaryColumns }) => (
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

describe('Dividend', () => {
    const mockCsvData = [
        {
            '入金日': '2023/01/15',
            '商品': '株式',
            '口座': '特定口座',
            '銘柄コード': '1234',
            '銘柄': 'テスト株式1',
            '単価[円/現地通貨]': '100',
            '数量[株/口]': '10',
            '配当・分配金合計（税引前）[円/現地通貨]': '1000',
            '税額合計[円/現地通貨]': '200',
            '受取金額[円/現地通貨]': '800'
        },
        {
            '入金日': '2023/02/15',
            '商品': '株式',
            '口座': '特定口座',
            '銘柄コード': '5678',
            '銘柄': 'テスト株式2',
            '単価[円/現地通貨]': '200',
            '数量[株/口]': '20',
            '配当・分配金合計（税引前）[円/現地通貨]': '4000',
            '税額合計[円/現地通貨]': '800',
            '受取金額[円/現地通貨]': '3200'
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
            { value: '5678', label: '5678:テスト株式2' }
        ]);

        const mockDividendData = [
            {
                settlement_date: new Date('2023-01-15'),
                product: '株式',
                account: '特定口座',
                security_code: '1234',
                security_name: 'テスト株式1',
                unit_price: 100,
                shares: 10,
                dividends_before_tax: 1000,
                taxes: 200,
                net_amount_received: 800
            },
            {
                settlement_date: new Date('2023-02-15'),
                product: '株式',
                account: '特定口座',
                security_code: '5678',
                security_name: 'テスト株式2',
                unit_price: 200,
                shares: 20,
                dividends_before_tax: 4000,
                taxes: 800,
                net_amount_received: 3200
            }
        ];
        
        (dataTransformer.filterDataBySearchQuery as jest.Mock).mockReturnValue(mockDividendData);
        
        (dataTransformer.groupAndSummarizeData as jest.Mock).mockReturnValue([
            {
                filter: '2023-01',
                dividends_before_tax: 1000,
                taxes: 200,
                net_amount_received: 800
            },
            {
                filter: '2023-02',
                dividends_before_tax: 4000,
                taxes: 800,
                net_amount_received: 3200
            }
        ]);
    });

    it('正しいタイトルとヘッダーアイテムが表示される', () => {
        render(<Dividend csvData={mockCsvData} />);
        
        expect(screen.getByTestId('title')).toHaveTextContent('配当金');
        
        expect(screen.getByTestId('title-0')).toHaveTextContent('合計配当金');
        expect(screen.getByTestId('value-0')).toHaveTextContent('¥ 5,000');
        
        expect(screen.getByTestId('title-1')).toHaveTextContent('合計税額');
        expect(screen.getByTestId('value-1')).toHaveTextContent('¥ 1,000');
        
        expect(screen.getByTestId('title-2')).toHaveTextContent('合計受取金額');
        expect(screen.getByTestId('value-2')).toHaveTextContent('¥ 4,000');
    });

    it('テーブルに正しいデータと設定が渡される', () => {
        render(<Dividend csvData={mockCsvData} />);
        
        const columnsData = JSON.parse(screen.getByTestId('columns').textContent || '[]');
        const summaryColumnsData = JSON.parse(screen.getByTestId('summary-columns').textContent || '[]');
        
        // カラム設定の検証
        expect(columnsData).toHaveLength(13);
        expect(columnsData[0].key).toBe('settlement_date');
        expect(columnsData[0].header).toBe('入金日');
        
        expect(columnsData[4].key).toBe('security_name');
        expect(columnsData[4].header).toBe('銘柄名');
        expect(columnsData[4].width).toBe('250px');
        
        // 集計カラム設定の検証
        expect(summaryColumnsData).toHaveLength(3);
        expect(summaryColumnsData[0].key).toBe('dividends_before_tax');
        expect(summaryColumnsData[0].colSpan).toBe(11);
        expect(summaryColumnsData[0].textAlign).toBe('right');
    });

    it('ユーティリティ関数が正しく呼び出される', () => {
        render(<Dividend csvData={mockCsvData} />);
        
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
            ['dividends_before_tax', 'taxes', 'net_amount_received']
        );
    });
});
