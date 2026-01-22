import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
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
    { header: '金額', key: 'amount', width: '120px', textAlign: 'right', format: (value) => `¥${value.toLocaleString()}` },
  ];

  const mockSummaryColumns: SummaryColumnConfig[] = [
    { key: 'name', colSpan: 2, textAlign: 'right', format: () => '合計:' },
    { key: 'amount', colSpan: 1, textAlign: 'right', format: (value) => `¥${value.toLocaleString()}` },
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

  it('HTMLタグが含まれる値が正しくレンダリングされる', () => {
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

    // ヘッダーの「リンク」を確認
    expect(screen.getAllByText('リンク')).toHaveLength(4); // ヘッダー1つ + データ3つ

    // リンク要素だけを取得
    const links = screen.getAllByRole('link');
    expect(links).toHaveLength(3);
    links.forEach(link => {
      expect(link).toHaveAttribute('href', 'https://example.com');
      expect(link).toHaveTextContent('リンク');
    });
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

    // グループAのデータの後にグループAのサマリーが表示される
    const table = screen.getByRole('table');
    const rows = table.querySelectorAll('tbody tr');
    
    // グループAのデータは最初の2行
    expect(rows[0]).toHaveTextContent('銘柄A');
    expect(rows[1]).toHaveTextContent('銘柄B');
    
    // グループAのサマリーは3行目
    expect(rows[2]).toHaveTextContent('合計:');
    expect(rows[2]).toHaveTextContent('¥3,000');
    
    // グループBのデータは4行目
    expect(rows[3]).toHaveTextContent('銘柄C');
    
    // グループBのサマリーは5行目
    expect(rows[4]).toHaveTextContent('合計:');
    expect(rows[4]).toHaveTextContent('¥3,000');
  });
});
