import { render, screen, within } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';

import { HomePage } from '../Home';

vi.mock('@/components/templates/Layout', () => ({
  Layout: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

describe('HomePage', () => {
  it('ページタイトルにホームを含む', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    expect(document.title).toContain('ホーム');
  });

  it('クイックアクションリンクを表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    const quickActionSection = screen
      .getByRole('heading', { name: 'クイックアクション' })
      .parentElement;

    expect(quickActionSection).not.toBeNull();

    const scoped = within(quickActionSection as HTMLElement);

    expect(scoped.getByRole('link', { name: '銘柄検索' })).toBeInTheDocument();
    expect(scoped.getByRole('link', { name: '資産管理' })).toBeInTheDocument();
    expect(scoped.getByRole('link', { name: '取引明細' })).toBeInTheDocument();
  });

  it('クイックアクションリンクの遷移先が正しい', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    const quickActionSection = screen
      .getByRole('heading', { name: 'クイックアクション' })
      .parentElement;

    expect(quickActionSection).not.toBeNull();

    const scoped = within(quickActionSection as HTMLElement);

    expect(scoped.getByRole('link', { name: '銘柄検索' })).toHaveAttribute('href', '/search');
    expect(scoped.getByRole('link', { name: '資産管理' })).toHaveAttribute('href', '/assetbalance');
    expect(scoped.getByRole('link', { name: '取引明細' })).toHaveAttribute('href', '/receipts');
  });
});
