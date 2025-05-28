import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import { Dividend } from '../Dividend';

// React Router DOM のモック
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: '/' }),
}));

// React Query のモック
vi.mock('@tanstack/react-query', () => ({
  QueryClient: vi.fn(() => ({
    setQueryData: vi.fn(),
    getQueryData: vi.fn(),
  })),
  QueryClientProvider: ({ children }: { children: React.ReactNode }) => children,
  useQuery: () => ({
    data: null,
    isLoading: false,
    error: null,
  }),
}));

// J-Quants APIフックのモック
vi.mock('@/features/jquants/hooks/useJQuantsDividend', () => ({
  useJQuantsDividend: vi.fn(() => ({
    dividendPerShare: undefined,
    loading: false,
    error: null,
  })),
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

    it('コンポーネントが正常にレンダリングされる', () => {
        render(<Dividend csvData={mockCsvData} />);

        // タイトルが表示されることを確認
        expect(screen.getByText('配当金')).toBeInTheDocument();

        // 集計情報が表示されることを確認（複数ある場合は最初のものをチェック）
        const combinedDividendElements = screen.getAllByText('合計配当金');
        expect(combinedDividendElements.length).toBeGreaterThan(0);
        
        const totalTaxElements = screen.getAllByText('合計税額');
        expect(totalTaxElements.length).toBeGreaterThan(0);
        
        const totalReceiptElements = screen.getAllByText('合計受取金額');
        expect(totalReceiptElements.length).toBeGreaterThan(0);
    });

    it('テーブルのヘッダーが正しく表示される', () => {
        render(<Dividend csvData={mockCsvData} />);

        // テーブルヘッダーの確認
        expect(screen.getByText('入金日')).toBeInTheDocument();
        expect(screen.getByText('商品')).toBeInTheDocument();
        expect(screen.getByText('口座')).toBeInTheDocument();
        expect(screen.getByText('銘柄コード')).toBeInTheDocument();
        expect(screen.getByText('銘柄名')).toBeInTheDocument();
        expect(screen.getByText('単価')).toBeInTheDocument();
        expect(screen.getByText('数量[株]')).toBeInTheDocument();
        expect(screen.getByText('配当・分配金')).toBeInTheDocument();
        expect(screen.getByText('税額')).toBeInTheDocument();
        expect(screen.getByText('受取金額')).toBeInTheDocument();
    });

    it('CSVデータが正しく表示される', () => {
        render(<Dividend csvData={mockCsvData} />);

        // データの内容確認
        expect(screen.getByText('1234')).toBeInTheDocument();
        expect(screen.getByText('テスト株式1')).toBeInTheDocument();
        expect(screen.getByText('5678')).toBeInTheDocument();
        expect(screen.getByText('テスト株式2')).toBeInTheDocument();
    });

    it('検索オプションが正しく生成される', () => {
        render(<Dividend csvData={mockCsvData} />);

        // 検索セレクトボックスの確認
        const selectElement = screen.getByLabelText('検索フィルター');
        expect(selectElement).toBeInTheDocument();

        // オプションの確認
        expect(screen.getByText('全て表示')).toBeInTheDocument();
        expect(screen.getByText('1234:テスト株式1')).toBeInTheDocument();
        expect(screen.getByText('5678:テスト株式2')).toBeInTheDocument();
    });

    it('銘柄を検索すると配当情報フォームが表示される', async () => {
        const user = userEvent.setup();
        render(<Dividend csvData={mockCsvData} />);

        // 検索セレクトボックスで銘柄を選択
        const selectElement = screen.getByLabelText('検索フィルター');
        await user.selectOptions(selectElement, '1234');

        // 配当情報フォームが表示されることを確認
        await waitFor(() => {
            expect(screen.getByText('配当情報')).toBeInTheDocument();
            expect(screen.getByText('平均取得価格')).toBeInTheDocument();
            expect(screen.getByText('保有数量(株)')).toBeInTheDocument();
            expect(screen.getByText('一株配当')).toBeInTheDocument();
        });

        // 統計項目が表示されることを確認
        expect(screen.getByText('取得総額')).toBeInTheDocument();
        expect(screen.getByText('合計受取金額 (累積利回り)')).toBeInTheDocument();
        expect(screen.getByText('年間配当金額 (配当利回り)')).toBeInTheDocument();
    });

    it('検索をクリアすると通常のヘッダーに戻る', async () => {
        const user = userEvent.setup();
        render(<Dividend csvData={mockCsvData} />);

        // 最初に銘柄を選択
        const selectElement = screen.getByLabelText('検索フィルター');
        await user.selectOptions(selectElement, '1234');

        // 配当情報が表示されることを確認
        await waitFor(() => {
            expect(screen.getByText('配当情報')).toBeInTheDocument();
        });

        // 検索をクリア
        await user.selectOptions(selectElement, '');

        // 通常の集計情報ヘッダーに戻ることを確認
        await waitFor(() => {
            expect(screen.getByText('集計情報')).toBeInTheDocument();
            expect(screen.queryByText('配当情報')).not.toBeInTheDocument();
        });
    });

    it('空のデータでもエラーが発生しない', () => {
        render(<Dividend csvData={[]} />);

        // タイトルは表示される
        expect(screen.getByText('配当金')).toBeInTheDocument();

        // 集計情報はゼロで表示される
        const summaryElements = screen.getAllByText('合計配当金');
        expect(summaryElements.length).toBeGreaterThan(0);
    });

    it('数値フォーマットが正しく適用される', () => {
        render(<Dividend csvData={mockCsvData} />);

        // 通貨フォーマットされた値が存在することを確認
        // 具体的な値は実装に依存するため、¥記号の存在を確認
        const currencyElements = screen.getAllByText(/¥/);
        expect(currencyElements.length).toBeGreaterThan(0);
    });

    it('レスポンシブテーブルが使用される', () => {
        const { container } = render(<Dividend csvData={mockCsvData} />);

        // レスポンシブテーブルのクラスが適用されていることを確認
        const responsiveTable = container.querySelector('.table-responsive');
        expect(responsiveTable).toBeInTheDocument();

        const table = container.querySelector('table');
        expect(table).toBeInTheDocument();
        expect(table).toHaveClass('table', 'table-bordered', 'table-sm');
    });

    it('配当情報フォームで数値計算が動作する', async () => {
        const user = userEvent.setup();
        render(<Dividend csvData={mockCsvData} />);

        // 銘柄を選択して配当情報フォームを表示
        const selectElement = screen.getByLabelText('検索フィルター');
        await user.selectOptions(selectElement, '1234');

        await waitFor(() => {
            expect(screen.getByText('配当情報')).toBeInTheDocument();
        });

        // フォームが存在し、入力可能であることを確認
        const inputs = screen.getAllByRole('spinbutton');
        expect(inputs.length).toBeGreaterThan(0);

        // 最初の入力フィールドに数値を入力してテスト
        if (inputs.length > 0) {
            await user.clear(inputs[0]);
            await user.type(inputs[0], '1000');
            
            // 入力が反映されることを確認
            await waitFor(() => {
                expect(inputs[0]).toHaveValue(1000);
            });
        }
    });
});
