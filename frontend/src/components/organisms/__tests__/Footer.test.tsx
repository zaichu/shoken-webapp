import { render, screen } from '@testing-library/react';
import { Footer } from '../Footer';

describe('Footer', () => {
  it('プライバシーポリシーリンクを表示する', () => {
    render(<Footer />);

    expect(screen.getByRole('link', { name: 'プライバシーポリシー（新しいタブで開く）' })).toBeInTheDocument();
  });

  it('利用規約リンクを表示する', () => {
    render(<Footer />);

    expect(screen.getByRole('link', { name: '利用規約（新しいタブで開く）' })).toBeInTheDocument();
  });

  it('Cookie ポリシーリンクを表示する', () => {
    render(<Footer />);

    expect(screen.getByRole('link', { name: 'Cookie ポリシー（新しいタブで開く）' })).toBeInTheDocument();
  });

  it('コピーライトを表示する', () => {
    render(<Footer />);

    expect(screen.getByText('© 2026 shoken-webapp')).toBeInTheDocument();
  });
});
