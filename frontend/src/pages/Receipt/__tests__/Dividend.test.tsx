import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import { Dividend } from '../Dividend';
import * as dataTransformer from '@/lib/utils/dataTransformer';
import * as jquantsApi from '@/lib/api/jquants';

// J-Quants APIフックのモック
vi.mock('@/lib/api/jquants', () => ({
    useJQuantsDividend: vi.fn()
}));

vi.mock('@/components/templates', () => ({
    ReceiptTemplate: ({ children, title, header, searchQuery, searchOptions, onSearch }) => (
        <div data-testid="receipt-template">
            <div data-testid="title">{title}</div>
            <div data-testid="header">{header}</div>
            <div data-testid="search-query">{searchQuery}</div>
            <div data-testid="search-options">
                {JSON.stringify(searchOptions)}
            </div>
            <select 
                data-testid="search-select" 
                aria-label="検索フィルター"
                value={searchQuery} 
                onChange={e => onSearch?.(e.target.value)} 
            >
                <option value="">全て表示</option>
                {searchOptions?.map((option, index) => (
                    <option key={index} value={option.value}>
                        {option.label}
                    </option>
                ))}
            </select>
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

vi.mock('@/components', () => ({
    NumberInputField: ({ label, value, onChange, disabled, helpText }) => (
        <div data-testid={`input-${label}`}>
            <label>{label}</label>
            <input 
                type="number" 
                value={value} 
                onChange={e => onChange?.(Number(e.target.value))}
                disabled={disabled}
                data-testid={`input-field-${label}`}
            />
            {helpText && <div data-testid={`help-text-${label}`}>{helpText}</div>}
        </div>
    ),
    StatItem: ({ title, value }) => (
        <div data-testid={`stat-${title}`}>
            <div>{title}</div>
            <div>{value}</div>
        </div>
    ),
    StatItemWithRate: ({ title, value, rate, format }) => (
        <div data-testid={`stat-with-rate-${title}`}>
            <div>{title}</div>
            <div>{format ? format(value) : value}</div>
            <div>{rate}%</div>
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

describe('Dividend', () => {
    const mockJQuantsHook = vi.mocked(jquantsApi.useJQuantsDividend);
    
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
        vi.mocked(dataTransformer.createSearchOptions).mockReset();
        vi.mocked(dataTransformer.filterDataBySearchQuery).mockReset();
        vi.mocked(dataTransformer.groupAndSummarizeData).mockReset();

        // モックの初期化
        vi.mocked(dataTransformer.createSearchOptions).mockReturnValue([
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
        
        vi.mocked(dataTransformer.filterDataBySearchQuery).mockReturnValue(mockDividendData);
        
        vi.mocked(dataTransformer.groupAndSummarizeData).mockReturnValue([
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
        
        // J-Quants APIフックのデフォルトモック
        mockJQuantsHook.mockReturnValue({
            dividendPerShare: 0,
            loading: false,
            error: null
        });
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

    describe('J-Quants APIを使用した配当情報の自動取得', () => {
        it('APIから配当情報が取得される場合', async () => {
            const user = userEvent.setup();
            
            // APIのモックを設定
            mockJQuantsHook.mockReturnValue({
                dividendPerShare: 50,
                loading: false,
                error: null
            });
            
            render(<Dividend csvData={mockCsvData} />);
            
            // 検索入力で銘柄を検索
            const searchSelect = screen.getByRole('combobox', { name: '検索フィルター' });
            await user.selectOptions(searchSelect, '1234');
            
            // APIフックが正しい引数で呼ばれることを確認
            expect(mockJQuantsHook).toHaveBeenCalledWith('1234', true);
            
            // 一株配当フィールドがAPIから取得した値で設定されていることを確認
            const dividendPerShareInput = screen.getByTestId('input-field-一株配当');
            expect(dividendPerShareInput).toHaveValue(50);
            
            // フィールドが無効化されていることを確認
            expect(dividendPerShareInput).toBeDisabled();
            
            // ヘルプテキストが表示されていることを確認
            expect(screen.getByTestId('help-text-一株配当')).toHaveTextContent('J-Quantsから自動取得');
        });
        
        it('APIがローディング中の場合', async () => {
            const user = userEvent.setup();
            
            // APIのモックを設定（ローディング中）
            mockJQuantsHook.mockReturnValue({
                dividendPerShare: 0,
                loading: true,
                error: null
            });
            
            render(<Dividend csvData={mockCsvData} />);
            
            // 検索入力で銘柄を検索
            const searchSelect = screen.getByRole('combobox', { name: '検索フィルター' });
            await user.selectOptions(searchSelect, '1234');
            
            // フィールドが無効化されていることを確認
            const dividendPerShareInput = screen.getByTestId('input-field-一株配当');
            expect(dividendPerShareInput).toBeDisabled();
            
            // ローディングメッセージが表示されていることを確認
            expect(screen.getByTestId('help-text-一株配当')).toHaveTextContent('データ取得中...');
        });
        
        it('APIでエラーが発生した場合', async () => {
            const user = userEvent.setup();
            
            // APIのモックを設定（エラー）
            mockJQuantsHook.mockReturnValue({
                dividendPerShare: 0,
                loading: false,
                error: 'APIエラーが発生しました'
            });
            
            render(<Dividend csvData={mockCsvData} />);
            
            // 検索入力で銘柄を検索
            const searchSelect = screen.getByRole('combobox', { name: '検索フィルター' });
            await user.selectOptions(searchSelect, '1234');
            
            // フィールドが有効であることを確認（手動入力可能）
            const dividendPerShareInput = screen.getByTestId('input-field-一株配当');
            expect(dividendPerShareInput).not.toBeDisabled();
            
            // エラーメッセージが表示されていることを確認
            expect(screen.getByTestId('help-text-一株配当')).toHaveTextContent('エラー: APIエラーが発生しました');
        });
        
        it('銘柄検索を解除すると手動入力可能になる', async () => {
            const user = userEvent.setup();
            
            // APIのモックを動的に変更できるように実装
            mockJQuantsHook.mockImplementation((code: string, enabled: boolean) => {
                if (!enabled || !code) {
                    return {
                        dividendPerShare: 0,
                        loading: false,
                        error: null
                    };
                }
                return {
                    dividendPerShare: 50,
                    loading: false,
                    error: null
                };
            });
            
            render(<Dividend csvData={mockCsvData} />);
            
            // 検索入力
            const searchSelect = screen.getByRole('combobox', { name: '検索フィルター' });
            await user.selectOptions(searchSelect, '1234');
            
            // 自動設定されていることを確認
            const dividendPerShareInput = screen.getByTestId('input-field-一株配当');
            expect(dividendPerShareInput).toBeDisabled();
            expect(dividendPerShareInput).toHaveValue(50);
            
            // 検索クリア
            await user.selectOptions(searchSelect, '');
            
            // DividendInfoコンポーネントがアンマウントされ、配当入力フィールド自体が表示されないことを確認
            expect(screen.queryByTestId('input-field-一株配当')).not.toBeInTheDocument();
            
            // 代わりにheaderが表示されることを確認
            expect(screen.getByTestId('receipt-header')).toBeInTheDocument();
        });
    });
});
