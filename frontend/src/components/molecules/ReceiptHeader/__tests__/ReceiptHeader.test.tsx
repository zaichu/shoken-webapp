import { render, screen } from '@testing-library/react';
import { ReceiptHeader } from '../ReceiptHeader';

describe('ReceiptHeader', () => {
  const defaultItems = [
    {
      title: 'テスト項目1',
      value: 1000,
      format: (value: number) => `¥${value.toLocaleString()}`,
    },
    {
      title: 'テスト項目2',
      value: 50,
      format: (value: number) => `${value}件`,
    },
  ];

  it('正しくレンダリングされる', () => {
    render(<ReceiptHeader items={defaultItems} />);
    
    expect(screen.getByText('集計情報')).toBeInTheDocument();
    expect(screen.getByText('テスト項目1')).toBeInTheDocument();
    expect(screen.getByText('¥1,000')).toBeInTheDocument();
    expect(screen.getByText('テスト項目2')).toBeInTheDocument();
    expect(screen.getByText('50件')).toBeInTheDocument();
  });

  it('空の配列でも正しくレンダリングされる', () => {
    render(<ReceiptHeader items={[]} />);
    
    expect(screen.getByText('集計情報')).toBeInTheDocument();
  });

  it('itemsの配列数に応じて適切な数の項目がレンダリングされる', () => {
    const items = [
      ...defaultItems,
      {
        title: 'テスト項目3',
        value: 200,
        format: (value: number) => `${value}%`,
      },
    ];

    const { container } = render(<ReceiptHeader items={items} />);
    
    const columns = container.querySelectorAll('.stat-grid > div');
    expect(columns).toHaveLength(3);
  });

  it('異なるフォーマット関数が正しく適用される', () => {
    const items = [
      {
        title: '通貨',
        value: 1500,
        format: (value: number) => `¥${value}`,
      },
      {
        title: 'パーセント',
        value: 75,
        format: (value: number) => `${value}%`,
      },
      {
        title: '数量',
        value: 10,
        format: (value: number) => `${value}個`,
      },
    ];

    render(<ReceiptHeader items={items} />);
    
    expect(screen.getByText('¥1500')).toBeInTheDocument();
    expect(screen.getByText('75%')).toBeInTheDocument();
    expect(screen.getByText('10個')).toBeInTheDocument();
  });
});
