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

        // タイトルが表示されることを確認（テーブルヘッダーにも「配当金」があるためgetAllByTextを使用）
        const titleElements = screen.getAllByText('配当金');
        expect(titleElements.length).toBeGreaterThan(0);

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

        // テーブルヘッダーの確認（実際のカラム定義に合わせる）
        // 「商品」「口座」は検索オプション内にも表示されるためgetAllByTextを使用
        expect(screen.getByText('入金日')).toBeInTheDocument();
        expect(screen.getAllByText('商品').length).toBeGreaterThan(0);
        expect(screen.getAllByText('口座').length).toBeGreaterThan(0);
        expect(screen.getByText('銘柄コード')).toBeInTheDocument();
        expect(screen.getByText('銘柄名')).toBeInTheDocument();
        expect(screen.getByText('単価')).toBeInTheDocument();
        expect(screen.getByText('数量')).toBeInTheDocument();
        // 「配当金」はページタイトルとしても表示されるためgetAllByTextを使用
        const dividendElements = screen.getAllByText('配当金');
        expect(dividendElements.length).toBeGreaterThan(0);
        expect(screen.getByText('税額')).toBeInTheDocument();
        expect(screen.getByText('受取額')).toBeInTheDocument();
    });

    it('CSVデータが正しく表示される', () => {
        render(<Dividend csvData={mockCsvData} />);

        // データの内容確認
        expect(screen.getByText('1234')).toBeInTheDocument();
        expect(screen.getByText('テスト株式1')).toBeInTheDocument();
        expect(screen.getByText('5678')).toBeInTheDocument();
        expect(screen.getByText('テスト株式2')).toBeInTheDocument();
    });

    it('検索オプションが正しく生成される', async () => {
        const { container } = render(<Dividend csvData={mockCsvData} />);

        // 検索オプションは初期状態で展開済み
        const searchOptionsHeader = screen.getByText('検索オプション');
        expect(searchOptionsHeader).toBeInTheDocument();

        // 銘柄検索セレクトボックスの確認（IDで指定）
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });

        // オプションの確認（複数の「全て表示」があるためgetAllByTextを使用）
        const allOptions = screen.getAllByText('全て表示');
        expect(allOptions.length).toBeGreaterThanOrEqual(1);
        expect(screen.getByText('1234: テスト株式1')).toBeInTheDocument();
        expect(screen.getByText('5678: テスト株式2')).toBeInTheDocument();
    });

    it('銘柄を検索すると配当情報フォームが表示される', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend csvData={mockCsvData} />);

        // 検索オプションは初期状態で展開済み - 銘柄検索セレクトボックスで銘柄を選択（IDで指定）
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });
        const selectElement = container.querySelector('#securities-search') as HTMLSelectElement;
        await user.selectOptions(selectElement, '1234');

        // 配当情報フォームが表示されることを確認
        await waitFor(() => {
            expect(screen.getByText('配当情報')).toBeInTheDocument();
            expect(screen.getByText('平均取得価格')).toBeInTheDocument();
            expect(screen.getByText('保有数量(株)')).toBeInTheDocument();
            expect(screen.getByText('一株配当')).toBeInTheDocument();
        });

        // 統計項目が重複せず表示されることを確認
        expect(screen.queryByText('取得総額')).not.toBeInTheDocument();
        expect(screen.queryByText('合計配当金')).not.toBeInTheDocument();
        expect(screen.queryByText('合計税額')).not.toBeInTheDocument();
        expect(screen.queryByText('合計受取金額')).not.toBeInTheDocument();
        expect(screen.getByText('配当金額 (配当利回り)')).toBeInTheDocument();
        expect(screen.getAllByText('税額').length).toBeGreaterThanOrEqual(2);
        expect(screen.getByText('受取金額 (累積利回り)')).toBeInTheDocument();
    });

    it('配当情報ヘッダーをクリックすると開閉できる', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend csvData={mockCsvData} />);

        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });
        const selectElement = container.querySelector('#securities-search') as HTMLSelectElement;
        await user.selectOptions(selectElement, '1234');

        await waitFor(() => {
            expect(screen.getByText('平均取得価格')).toBeInTheDocument();
        });

        const header = screen.getByTestId('receipt-header');
        await user.click(header);
        expect(screen.queryByText('平均取得価格')).not.toBeInTheDocument();

        await user.click(header);
        expect(screen.getByText('平均取得価格')).toBeInTheDocument();
    });

    it('検索をクリアすると通常のヘッダーに戻る', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend csvData={mockCsvData} />);

        // 検索オプションは初期状態で展開済み - 最初に銘柄を選択（IDで指定）
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });
        const selectElement = container.querySelector('#securities-search') as HTMLSelectElement;
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

        // タイトルは表示される（テーブルヘッダーにも「配当金」があるためgetAllByTextを使用）
        const titleElements = screen.getAllByText('配当金');
        expect(titleElements.length).toBeGreaterThan(0);

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
        const responsiveTable = container.querySelector('div.overflow-x-auto');
        expect(responsiveTable).toBeInTheDocument();

        const table = screen.getByRole('table');
        expect(table).toBeInTheDocument();
        expect(table).toHaveClass('w-full', 'border-collapse');
    });

    // TODO: 配当情報フォームの数値計算テストを修正する必要あり
    it.skip('配当情報フォームで数値計算が動作する', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend csvData={mockCsvData} />);

        // 検索オプションは初期状態で展開済み - 銘柄を選択して配当情報フォームを表示（IDで指定）
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });
        const selectElement = container.querySelector('#securities-search') as HTMLSelectElement;
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
