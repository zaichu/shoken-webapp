import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { vi } from 'vitest';
import { Header } from '../Header';

vi.mock('@/features/auth/hooks/useAuth', () => ({
  useAuth: () => ({
    user: {
      name: 'テストユーザー',
      email: 'test@example.com',
      picture: null,
      picture_url: null,
    },
    login: vi.fn(),
    logout: vi.fn(),
    deleteAccount: vi.fn(),
    isAuthenticated: true,
    isLoading: false,
  }),
}));

describe('Header', () => {
  it('主要ナビゲーションリンクを表示する', () => {
    render(
      <MemoryRouter>
        <Header />
      </MemoryRouter>
    );

    expect(screen.getByRole('link', { name: '銘柄検索' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: '資産管理' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: '取引明細' })).toBeInTheDocument();
  });

  it('ログイン状態でユーザーアバターを表示する', () => {
    render(
      <MemoryRouter>
        <Header />
      </MemoryRouter>
    );

    expect(screen.getByLabelText('テストユーザー')).toBeInTheDocument();
  });
});
