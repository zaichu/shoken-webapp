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

  it('フィーチャーカードを3件表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    expect(screen.getByRole('heading', { name: '銘柄検索' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: '資産管理' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: '取引明細' })).toBeInTheDocument();
  });

  it('フィーチャーカードの説明文を表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    expect(screen.getByText('日本の株式銘柄情報を検索できます。')).toBeInTheDocument();
    expect(screen.getByText('保有している銘柄の一覧と評価額を確認できます。')).toBeInTheDocument();
    expect(screen.getByText('配当金や分配金の記録を管理できます。')).toBeInTheDocument();
  });

  it('フィーチャーカードのリンク先が正しい', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    // フィーチャーカードはクイックアクションと同名リンクがあるため、すべてのリンクを取得して確認する
    const links = screen.getAllByRole('link', { name: /銘柄検索/ });
    expect(links.some((l) => l.getAttribute('href') === '/search')).toBe(true);

    const assetLinks = screen.getAllByRole('link', { name: /資産管理/ });
    expect(assetLinks.some((l) => l.getAttribute('href') === '/assetbalance')).toBe(true);

    const receiptLinks = screen.getAllByRole('link', { name: /取引明細/ });
    expect(receiptLinks.some((l) => l.getAttribute('href') === '/receipts')).toBe(true);
  });

  it('はじめかたセクションのSTEPを表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    expect(screen.getByText('STEP 1')).toBeInTheDocument();
    expect(screen.getByText('証券会社からCSVをダウンロード')).toBeInTheDocument();
    expect(screen.getByText('STEP 2')).toBeInTheDocument();
    expect(screen.getByText('各ページでCSVファイルを取り込み')).toBeInTheDocument();
    expect(screen.getByText('STEP 3')).toBeInTheDocument();
    expect(screen.getByText('配当金や資産管理の情報を確認')).toBeInTheDocument();
  });
});
