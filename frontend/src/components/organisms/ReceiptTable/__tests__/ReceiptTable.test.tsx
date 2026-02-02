import { render, screen } from '@testing-library/react';
import { ReceiptTable } from '../ReceiptTable';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';

// ResizeObserver のモック（class形式で定義）
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
global.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

describe('ReceiptTable', () => {
  const mockColumns: TableColumnConfig[] = [
    { header: '日付', key: 'date', width: '100px', textAlign: 'center' },
    { header: '銘柄', key: 'name', width: '200px', textAlign: 'left' },
    { header: '金額', key: 'amount', width: '120px', textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
  ];

  const mockSummaryColumns: SummaryColumnConfig[] = [
    { key: 'name', colSpan: 2, textAlign: 'right', format: () => '合計:' },
    { key: 'amount', colSpan: 1, textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
  ];

  const mockData = [
    { date: '2024-01-01', name: '銘柄A', amount: 1000, group: 'A' },
    { date: '2024-01-02', name: '銘柄B', amount: 2000, group: 'A' },
    { date: '2024-01-03', name: '銘柄C', amount: 3000, group: 'B' },
  ];

  const mockSummary = [
    { filter: 'A', amount: 3000, name: 'グループA' },
    { filter: 'B', amount: 3000, name: 'グループB' },
  ];

  const getGroupKey = (item: typeof mockData[0]) => item.group;

  it('データが正しくレンダリングされる', () => {
    render(
      <ReceiptTable
        data={mockData}
        summary={mockSummary}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    // ヘッダー
    expect(screen.getByText('日付')).toBeInTheDocument();
    expect(screen.getByText('銘柄')).toBeInTheDocument();
    expect(screen.getByText('金額')).toBeInTheDocument();

    // データ
    expect(screen.getByText('銘柄A')).toBeInTheDocument();
    expect(screen.getByText('銘柄B')).toBeInTheDocument();
    expect(screen.getByText('銘柄C')).toBeInTheDocument();
    expect(screen.getByText('¥1,000')).toBeInTheDocument();
    expect(screen.getByText('¥2,000')).toBeInTheDocument();
    
    // ¥3,000は複数あるので、getAllByTextを使用
    const threeThousandElements = screen.getAllByText('¥3,000');
    expect(threeThousandElements).toHaveLength(3); // データ1つ + サマリー2つ

    // サマリー
    expect(screen.getAllByText('合計:')).toHaveLength(2);
  });

  it('HTMLタグが含まれる文字列はエスケープされてテキストとして表示される（XSS対策）', () => {
    const columnsWithHtml: TableColumnConfig[] = [
      ...mockColumns,
      {
        header: 'リンク',
        key: 'link',
        width: '100px',
        textAlign: 'center',
        format: (value) => `<a href="${value}">リンク</a>`
      },
    ];

    const dataWithHtml = mockData.map(item => ({ ...item, link: 'https://example.com' }));

    render(
      <ReceiptTable
        data={dataWithHtml}
        summary={mockSummary}
        columns={columnsWithHtml}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    // ヘッダーの「リンク」のみ表示
    expect(screen.getByText('リンク')).toBeInTheDocument();

    // HTML文字列はリンクとしてレンダリングされず、テキストとして表示される
    expect(screen.queryAllByRole('link')).toHaveLength(0);

    // エスケープされたHTML文字列がテキストとして表示される
    expect(screen.getAllByText('<a href="https://example.com">リンク</a>')).toHaveLength(3);
  });

  it('空のデータでも正しくレンダリングされる', () => {
    render(
      <ReceiptTable
        data={[]}
        summary={[]}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    // ヘッダーは表示される
    expect(screen.getByText('日付')).toBeInTheDocument();
    expect(screen.getByText('銘柄')).toBeInTheDocument();
    expect(screen.getByText('金額')).toBeInTheDocument();

    // データ行は表示されない
    expect(screen.queryByText('銘柄A')).not.toBeInTheDocument();
  });

  it('グループごとにデータが正しくフィルタリングされる', () => {
    render(
      <ReceiptTable
        data={mockData}
        summary={mockSummary}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    // 実装ではサマリー行が先に表示され、その後にデータ行が続く
    const table = screen.getByRole('table');
    const rows = table.querySelectorAll('tbody tr');

    // グループAのサマリーが最初
    expect(rows[0]).toHaveTextContent('A');
    expect(rows[0]).toHaveTextContent('2件');
    expect(rows[0]).toHaveTextContent('¥3,000');

    // グループAのデータ行
    expect(rows[1]).toHaveTextContent('銘柄A');
    expect(rows[2]).toHaveTextContent('銘柄B');

    // グループBのサマリー
    expect(rows[3]).toHaveTextContent('B');
    expect(rows[3]).toHaveTextContent('1件');
    expect(rows[3]).toHaveTextContent('¥3,000');

    // グループBのデータ行
    expect(rows[4]).toHaveTextContent('銘柄C');
  });
});
