import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';

import { NotFoundPage } from '../NotFound';

vi.mock('@/components/templates/Layout', () => ({
  Layout: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

describe('NotFoundPage', () => {
  it('404タイトルを表示する', () => {
    render(
      <MemoryRouter>
        <NotFoundPage />
      </MemoryRouter>
    );

    expect(
      screen.getByRole('heading', { name: '404 - ページが見つかりません' })
    ).toBeInTheDocument();
  });

  it('ホームに戻るボタンを表示する', () => {
    render(
      <MemoryRouter>
        <NotFoundPage />
      </MemoryRouter>
    );

    expect(screen.getByRole('button', { name: 'ホームに戻る' })).toBeInTheDocument();
  });

  it('関連リンクを表示する', () => {
    render(
      <MemoryRouter>
        <NotFoundPage />
      </MemoryRouter>
    );

    expect(screen.getByRole('link', { name: 'ホーム' })).toHaveAttribute('href', '/');
    expect(screen.getByRole('link', { name: '銘柄検索' })).toHaveAttribute('href', '/search');
    expect(screen.getByRole('link', { name: '取引明細' })).toHaveAttribute('href', '/receipts');
  });
});
