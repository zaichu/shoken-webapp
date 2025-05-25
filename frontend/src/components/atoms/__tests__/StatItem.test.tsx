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

    expect(titleElement).toHaveClass('h6');
    expect(titleElement).toHaveClass('mb-0');
    expect(valueElement).toHaveClass('h4');
    expect(valueElement).toHaveClass('mb-0');
  });

  it('カードバリアントが適用される', () => {
    const { container } = render(<StatItem title="タイトル" value="値" variant="card" />);

    const containerElement = container.querySelector('.card.p-3');
    const titleElement = screen.getByText('タイトル');
    const valueElement = screen.getByText('値');

    expect(containerElement).toBeInTheDocument();
    expect(titleElement).toHaveClass('card-title', 'h6');
    expect(valueElement).toHaveClass('card-text', 'h4');
  });

  it('インラインバリアントが適用される', () => {
    const { container } = render(<StatItem title="タイトル" value="値" variant="inline" />);

    const containerElement = container.querySelector('.d-flex.justify-content-between.align-items-center');
    const titleElement = screen.getByText('タイトル');
    const valueElement = screen.getByText('値');

    expect(containerElement).toBeInTheDocument();
    expect(titleElement).toHaveClass('mb-0', 'text-muted');
    expect(valueElement).toHaveClass('mb-0', 'fw-bold');
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

    const cardElement = container.querySelector('.card.p-3');
    expect(cardElement).toBeInTheDocument();
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
