import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import { StatItem, StatItemWithRate } from '../StatItem';

describe('StatItem', () => {
  it('renders title and value correctly', () => {
    render(
      <StatItem
        title="テストタイトル"
        value="テスト値"
      />
    );

    expect(screen.getByText('テストタイトル')).toBeInTheDocument();
    expect(screen.getByText('テスト値')).toBeInTheDocument();
  });

  it('applies custom className when provided', () => {
    const customClass = 'custom-col-class';
    render(
      <StatItem
        title="テストタイトル"
        value="テスト値"
        className={customClass}
      />
    );

    const container = screen.getByText('テストタイトル').closest('div');
    expect(container).toHaveClass(customClass);
  });

  it('applies default className when not provided', () => {
    render(
      <StatItem
        title="テストタイトル"
        value="テスト値"
      />
    );

    const container = screen.getByText('テストタイトル').closest('div');
    expect(container).toHaveClass('col');
  });

  it('renders React element as value', () => {
    const reactElement = <span data-testid="custom-element">カスタム要素</span>;
    render(
      <StatItem
        title="テストタイトル"
        value={reactElement}
      />
    );

    expect(screen.getByTestId('custom-element')).toBeInTheDocument();
    expect(screen.getByText('カスタム要素')).toBeInTheDocument();
  });
});

describe('StatItemWithRate', () => {
  it('renders value with rate correctly', () => {
    const mockFormat = vi.fn((value) => `¥${value.toLocaleString()}`);
    render(
      <StatItemWithRate
        title="収益"
        value={1000000}
        rate={15.5}
        format={mockFormat}
      />
    );

    expect(screen.getByText('収益')).toBeInTheDocument();
    expect(screen.getByText('¥1,000,000 (15.50%)')).toBeInTheDocument();
    expect(mockFormat).toHaveBeenCalledWith(1000000);
  });

  it('renders value without rate when rate is undefined', () => {
    const mockFormat = vi.fn((value) => `¥${value.toLocaleString()}`);
    render(
      <StatItemWithRate
        title="収益"
        value={1000000}
        format={mockFormat}
      />
    );

    expect(screen.getByText('¥1,000,000')).toBeInTheDocument();
    expect(screen.queryByText(/\(/)).not.toBeInTheDocument(); // パーセンテージが含まれていないことを確認
  });

  it('uses custom rateFormat when provided', () => {
    const mockFormat = vi.fn((value) => `¥${value.toLocaleString()}`);
    const mockRateFormat = vi.fn((rate) => `${rate}%`);
    
    render(
      <StatItemWithRate
        title="収益"
        value={1000000}
        rate={15.555}
        format={mockFormat}
        rateFormat={mockRateFormat}
      />
    );

    expect(screen.getByText('¥1,000,000 (15.555%)')).toBeInTheDocument();
    expect(mockRateFormat).toHaveBeenCalledWith(15.555);
  });

  it('uses default format when not provided', () => {
    render(
      <StatItemWithRate
        title="収益"
        value={1000000}
        rate={15.5}
      />
    );

    expect(screen.getByText('1000000 (15.50%)')).toBeInTheDocument();
  });

  it('passes className to StatItem', () => {
    const customClass = 'custom-col-span';
    render(
      <StatItemWithRate
        title="収益"
        value={1000000}
        className={customClass}
      />
    );

    const container = screen.getByText('収益').closest('div');
    expect(container).toHaveClass(customClass);
  });

  it('handles zero value correctly', () => {
    const mockFormat = vi.fn((value) => `¥${value.toLocaleString()}`);
    render(
      <StatItemWithRate
        title="収益"
        value={0}
        rate={0}
        format={mockFormat}
      />
    );

    expect(screen.getByText('¥0 (0.00%)')).toBeInTheDocument();
  });

  it('handles negative rate correctly', () => {
    const mockFormat = vi.fn((value) => `¥${value.toLocaleString()}`);
    render(
      <StatItemWithRate
        title="損失"
        value={1000000}
        rate={-15.5}
        format={mockFormat}
      />
    );

    expect(screen.getByText('¥1,000,000 (-15.50%)')).toBeInTheDocument();
  });
});
