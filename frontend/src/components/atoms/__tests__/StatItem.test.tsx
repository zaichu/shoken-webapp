import { render, screen } from '@testing-library/react';
import { StatItem, StatItemWithRate } from '../StatItem';

describe('StatItem', () => {
  it('基本的な統計アイテムを表示する', () => {
    render(<StatItem title="売上高" value="¥ 1,000,000" />);

    expect(screen.getByText('売上高')).toBeInTheDocument();
    expect(screen.getByText('¥ 1,000,000')).toBeInTheDocument();
  });

  it('ReactNodeを値として表示する', () => {
    const value = <span data-testid="custom-value">カスタム値</span>;
    render(<StatItem title="カスタム項目" value={value} />);

    expect(screen.getByText('カスタム項目')).toBeInTheDocument();
    expect(screen.getByTestId('custom-value')).toBeInTheDocument();
  });

  it('デフォルトバリアントが適用される', () => {
    render(<StatItem title="タイトル" value="値" />);

    const titleElement = screen.getByText('タイトル');
    const valueElement = screen.getByText('値');

    expect(titleElement).toHaveClass('text-base', 'font-semibold');
    expect(valueElement).toHaveClass('text-xl', 'font-bold');
  });

  it('カードバリアントが適用される', () => {
    const { container } = render(<StatItem title="タイトル" value="値" variant="card" />);

    const containerElement = container.firstChild as HTMLElement;
    const titleElement = screen.getByText('タイトル');
    const valueElement = screen.getByText('値');

    expect(containerElement).toBeInTheDocument();
    expect(containerElement).toHaveClass('bg-white', 'rounded-lg', 'p-3');
    expect(titleElement).toHaveClass('text-base', 'font-semibold', 'text-secondary');
    expect(valueElement).toHaveClass('text-xl', 'font-bold');
  });

  it('インラインバリアントが適用される', () => {
    const { container } = render(<StatItem title="タイトル" value="値" variant="inline" />);

    const containerElement = container.firstChild as HTMLElement;
    const titleElement = screen.getByText('タイトル');
    const valueElement = screen.getByText('値');

    expect(containerElement).toBeInTheDocument();
    expect(containerElement).toHaveClass('flex', 'justify-between', 'items-center');
    expect(titleElement).toHaveClass('text-secondary');
    expect(valueElement).toHaveClass('font-bold');
  });

  it('カスタムクラス名が適用される', () => {
    const { container } = render(
      <StatItem
        title="タイトル"
        value="値"
        className="custom-container"
        titleClassName="custom-title"
        valueClassName="custom-value"
      />
    );

    const containerElement = container.querySelector('.custom-container');
    const titleElement = screen.getByText('タイトル');
    const valueElement = screen.getByText('値');

    expect(containerElement).toBeInTheDocument();
    expect(titleElement).toHaveClass('custom-title');
    expect(valueElement).toHaveClass('custom-value');
  });
});

describe('StatItemWithRate', () => {
  it('基本的な統計アイテム（レート付き）を表示する', () => {
    render(<StatItemWithRate title="売上高" value={1000000} rate={15.5} />);

    expect(screen.getByText('売上高')).toBeInTheDocument();
    expect(screen.getByText('1,000,000 (15.50%)')).toBeInTheDocument();
  });

  it('レートなしで表示する', () => {
    render(<StatItemWithRate title="売上高" value={1000000} />);

    expect(screen.getByText('売上高')).toBeInTheDocument();
    expect(screen.getByText('1,000,000')).toBeInTheDocument();
    expect(screen.queryByText(/\(/)).not.toBeInTheDocument();
  });

  it('showRateがfalseの場合はレートを表示しない', () => {
    render(<StatItemWithRate title="売上高" value={1000000} rate={15.5} showRate={false} />);

    expect(screen.getByText('1,000,000')).toBeInTheDocument();
  });

  it('カスタムフォーマット関数が適用される', () => {
    const format = (value: number) => `$${value}`;
    const rateFormat = (rate: number) => `${rate}%`;

    render(
      <StatItemWithRate title="売上高" value={1000} rate={10} format={format} rateFormat={rateFormat} />
    );

    expect(screen.getByText('$1000 (10%)')).toBeInTheDocument();
  });

  it('variantが適用される', () => {
    const { container } = render(
      <StatItemWithRate title="売上高" value={1000} rate={10} variant="card" />
    );

    const cardElement = container.firstChild as HTMLElement;
    expect(cardElement).toBeInTheDocument();
    expect(cardElement).toHaveClass('bg-white', 'rounded-lg', 'p-3');
  });

  it('カスタムクラス名が適用される', () => {
    const { container } = render(
      <StatItemWithRate title="売上高" value={1000} className="custom-stat" />
    );

    const containerElement = container.querySelector('.custom-stat');
    expect(containerElement).toBeInTheDocument();
  });

  it('0の値も正しく表示される', () => {
    render(<StatItemWithRate title="利益" value={0} rate={0} />);

    expect(screen.getByText('0 (0.00%)')).toBeInTheDocument();
  });

  it('負の値も正しく表示される', () => {
    render(<StatItemWithRate title="損失" value={-1000} rate={-5.5} />);
    expect(screen.getByText('-1,000 (-5.50%)')).toBeInTheDocument();
  });
});
