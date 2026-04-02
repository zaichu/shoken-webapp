import { render, screen } from '@testing-library/react';
import { EmptyState } from '../EmptyState';

describe('EmptyState', () => {
  it('title を表示する', () => {
    render(<EmptyState title="データがありません" />);

    expect(screen.getByRole('heading', { level: 3, name: 'データがありません' })).toBeInTheDocument();
  });

  it('description を表示する', () => {
    render(<EmptyState title="データがありません" description="CSV を読み込んでください" />);

    expect(screen.getByText('CSV を読み込んでください')).toBeInTheDocument();
  });

  it('action を表示する', () => {
    render(<EmptyState title="データがありません" action={<button>再読み込み</button>} />);

    expect(screen.getByRole('button', { name: '再読み込み' })).toBeInTheDocument();
  });

  it('headingLevel が h1 のとき h1 を使う', () => {
    render(<EmptyState title="トップ見出し" headingLevel="h1" />);

    expect(screen.getByRole('heading', { level: 1, name: 'トップ見出し' })).toBeInTheDocument();
  });

  it('icon を表示する', () => {
    render(<EmptyState title="データなし" icon={<span data-testid="empty-icon">📭</span>} />);

    expect(screen.getByTestId('empty-icon')).toBeInTheDocument();
  });
});
