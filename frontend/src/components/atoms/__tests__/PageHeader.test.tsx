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
});
