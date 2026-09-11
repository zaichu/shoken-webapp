import { render, screen } from '@testing-library/react';
import { PageHeader } from '../PageHeader';

describe('PageHeader', () => {
  it('title を h1 として表示する', () => {
    render(<PageHeader title="資産管理" />);

    expect(screen.getByRole('heading', { level: 1, name: '資産管理' })).toBeInTheDocument();
  });

  it('description を表示する', () => {
    render(<PageHeader title="資産管理" description="保有資産の一覧です" />);

    expect(screen.getByText('保有資産の一覧です')).toBeInTheDocument();
  });

  it('actions を表示する', () => {
    render(<PageHeader title="資産管理" actions={<button>追加</button>} />);

    expect(screen.getByRole('button', { name: '追加' })).toBeInTheDocument();
  });

  it('eyebrow を指定した場合だけ表示する', () => {
    const { rerender } = render(<PageHeader title="資産管理" />);

    expect(screen.queryByText('Portfolio')).not.toBeInTheDocument();
    expect(screen.queryByText('Workspace')).not.toBeInTheDocument();

    rerender(<PageHeader title="資産管理" eyebrow="Portfolio" />);

    expect(screen.getByText('Portfolio')).toBeInTheDocument();
  });

  it('スマホ幅ではタイトルを縮小し、eyebrow と description を非表示にする', () => {
    render(<PageHeader title="取引明細" eyebrow="Transactions" description="配当金・国内株式・投資信託の取引明細を管理します。" />);

    expect(screen.getByRole('heading', { level: 1 })).toHaveClass('max-sm:text-lg');
    expect(screen.getByText('Transactions')).toHaveClass('max-sm:hidden');
    expect(screen.getByText('配当金・国内株式・投資信託の取引明細を管理します。')).toHaveClass(
      'max-sm:hidden'
    );
  });
});
