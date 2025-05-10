import { render, screen } from '@testing-library/react';
import { DomesticStock } from '../DomesticStock';
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
            '売却/決済単価[円]': '1000',
            '売却/決済額[円]': '100000',
            '平均取得価額[円]': '900',
            '実現損益[円]': '10000'
        },
        {
            '約定日': '2023/01/15', // 同日
            '受渡日': '2023/01/18',
            '銘柄コード': 5678,
            '銘柄名': 'テスト株式2',
            '口座': '特定口座',
            '数量[株]': 200,
            '売却/決済単価[円]': '500',
            '売却/決済額[円]': '100000',
            '平均取得価額[円]': '450',
            '実現損益[円]': '10000'
        },
        {
            '約定日': '2023/02/15',
            '受渡日': '2023/02/18',
            '銘柄コード': 9012,
            '銘柄名': 'テスト株式3',
            '口座': '特定口座',
            '数量[株]': 300,
            '売却/決済単価[円]': '800',
            '売却/決済額[円]': '240000',
            '平均取得価額[円]': '700',
            '実現損益[円]': '30000'
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
                trade_date: new Date('2023-01-15'),
                settlement_date: new Date('2023-01-18'),
                security_code: '1234',
                security_name: 'テスト株式1',
                account: '特定口座',
                shares: 100,
                asked_price: 1000,
                proceeds: 100000,
                purchase_price: 900,
                realized_profit_and_loss: 10000
            },
            {
                trade_date: new Date('2023-01-15'),
                settlement_date: new Date('2023-01-18'),
                security_code: '5678',
                security_name: 'テスト株式2',
                account: '特定口座',
                shares: 200,
                asked_price: 500,
                proceeds: 100000,
                purchase_price: 450,
                realized_profit_and_loss: 10000
            },
            {
                trade_date: new Date('2023-02-15'),
                settlement_date: new Date('2023-02-18'),
                security_code: '9012',
                security_name: 'テスト株式3',
                account: '特定口座',
                shares: 300,
                asked_price: 800,
                proceeds: 240000,
                purchase_price: 700,
                realized_profit_and_loss: 30000
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
        
        expect(screen.getByTestId('title')).toHaveTextContent('国内株式');
        
        expect(screen.getByTestId('title-0')).toHaveTextContent('合計実現損益');
        expect(screen.getByTestId('value-0')).toHaveTextContent('¥ 50,000');
        
        expect(screen.getByTestId('title-1')).toHaveTextContent('合計税額');
        expect(screen.getByTestId('value-1')).toHaveTextContent('¥ 10,158');
        
        expect(screen.getByTestId('title-2')).toHaveTextContent('合計実現損益(税引)');
        expect(screen.getByTestId('value-2')).toHaveTextContent('¥ 39,842');
    });

    it('テーブルに正しいデータと設定が渡される', () => {
        render(<DomesticStock csvData={mockCsvData} />);
        
        const columnsData = JSON.parse(screen.getByTestId('columns').textContent || '[]');
        const summaryColumnsData = JSON.parse(screen.getByTestId('summary-columns').textContent || '[]');
        
        // カラム設定の検証
        expect(columnsData).toHaveLength(13);
        expect(columnsData[0].key).toBe('trade_date');
        expect(columnsData[0].header).toBe('約定日');
        
        expect(columnsData[1].key).toBe('settlement_date');
        expect(columnsData[1].header).toBe('受渡日');
        
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
    });
});
