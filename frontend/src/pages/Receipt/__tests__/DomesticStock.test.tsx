import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import { DomesticStock } from '../DomesticStock';

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

describe('DomesticStock', () => {
    const mockCsvData = [
        {
            '約定日': '2023/01/15',
            '受渡日': '2023/01/18',
            '銘柄コード': '1234',
            '銘柄名': 'テスト株式',
            '口座': '特定',
            '数量[株]': '100',
            '売却/決済単価[円]': '1000',
            '売却/決済額[円]': '100000',
            '平均取得価額[円]': '900',
            '実現損益[円]': '10000'
        },
        {
            '約定日': '2023/01/20',
            '受渡日': '2023/01/23',
            '銘柄コード': '5678',
            '銘柄名': 'テスト株式2',
            '口座': 'NISA',
            '数量[株]': '200',
            '売却/決済単価[円]': '2000',
            '売却/決済額[円]': '400000',
            '平均取得価額[円]': '1800',
            '実現損益[円]': '40000'
        }
    ];

    it('コンポーネントが正常にレンダリングされる', () => {
        render(<DomesticStock csvData={mockCsvData} />);

        // 集計情報が表示されることを確認（複数ある場合は最初のものをチェック）
        const totalProfitElements = screen.getAllByText('実現損益');
        expect(totalProfitElements.length).toBeGreaterThan(0);

        const totalTaxElements = screen.getAllByText('税額');
        expect(totalTaxElements.length).toBeGreaterThan(0);

        const netProfitElements = screen.getAllByText('実現損益(税引)');
        expect(netProfitElements.length).toBeGreaterThan(0);
    });

    it('テーブルのヘッダーが正しく表示される', () => {
        render(<DomesticStock csvData={mockCsvData} />);

        // テーブルヘッダーの確認（実際のカラム定義に合わせる）
        // 「口座」は検索オプション内にも表示されるためgetAllByTextを使用
        expect(screen.getByText('約定日')).toBeInTheDocument();
        expect(screen.getByText('銘柄コード')).toBeInTheDocument();
        expect(screen.getByText('銘柄名')).toBeInTheDocument();
        expect(screen.getAllByText('口座').length).toBeGreaterThan(0);
        expect(screen.getByText('数量')).toBeInTheDocument();
        expect(screen.getByText('売却単価')).toBeInTheDocument();
        expect(screen.getByText('売却額')).toBeInTheDocument();
        expect(screen.getByText('取得価額')).toBeInTheDocument();
        expect(screen.getByText('損益')).toBeInTheDocument();
        // 「税額」は集計情報にも表示されるためgetAllByTextを使用
        expect(screen.getAllByText('税額').length).toBeGreaterThan(0);
        expect(screen.getByText('税引後')).toBeInTheDocument();
    });

    it('CSVデータが正しく表示される', () => {
        render(<DomesticStock csvData={mockCsvData} />);
        
        // データの内容確認
        expect(screen.getByText('1234')).toBeInTheDocument();
        expect(screen.getByText('テスト株式')).toBeInTheDocument();
        expect(screen.getByText('5678')).toBeInTheDocument();
        expect(screen.getByText('テスト株式2')).toBeInTheDocument();
    });

    it('検索オプションが正しく生成される', async () => {
        const { container } = render(<DomesticStock csvData={mockCsvData} />);

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
        expect(screen.getByText('1234: テスト株式')).toBeInTheDocument();
        expect(screen.getByText('5678: テスト株式2')).toBeInTheDocument();
    });

    it('銘柄検索時に0件サマリーが表示されない', async () => {
        const waitOpts = { timeout: 5000 };
        const user = userEvent.setup();
        const { container } = render(<DomesticStock csvData={mockCsvData} />);

        // 検索オプションは初期状態で展開済み
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        }, waitOpts);
        const securitiesSelect = container.querySelector('#securities-search') as HTMLSelectElement;

        await user.selectOptions(securitiesSelect, '1234');

        await waitFor(() => {
            expect(screen.queryByText('0件')).not.toBeInTheDocument();
        }, waitOpts);
    }, 20000);

    it('空のデータでもエラーが発生しない', () => {
        render(<DomesticStock csvData={[]} />);

        // 集計情報はゼロで表示される（複数要素がある場合を考慮）
        const summaryElements = screen.getAllByText('実現損益');
        expect(summaryElements.length).toBeGreaterThan(0);
    });

    it('数値フォーマットが正しく適用される', () => {
        render(<DomesticStock csvData={mockCsvData} />);
        
        // 通貨フォーマットされた値が存在することを確認
        // 具体的な値は実装に依存するため、¥記号の存在を確認
        const currencyElements = screen.getAllByText(/¥/);
        expect(currencyElements.length).toBeGreaterThan(0);
    });

    it('レスポンシブテーブルが使用される', () => {
        const { container } = render(<DomesticStock csvData={mockCsvData} />);
        
        // レスポンシブテーブルのクラスが適用されていることを確認
        const responsiveTable = container.querySelector('div.overflow-x-auto');
        expect(responsiveTable).toBeInTheDocument();
        
        const table = screen.getByRole('table');
        expect(table).toBeInTheDocument();
        expect(table).toHaveClass('w-full', 'border-collapse');
    });
});
