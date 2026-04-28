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

  it('ダッシュボードヘッダーを表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    expect(screen.getByRole('heading', { level: 1, name: '証券Web' })).toBeInTheDocument();
    expect(screen.getByText('日々の資産・取引を確認する')).toBeInTheDocument();
  });

  it('ステータスストリップに4件のタイルを表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    expect(screen.getAllByText('銘柄検索').length).toBeGreaterThan(0);
    expect(screen.getAllByText('資産管理').length).toBeGreaterThan(0);
    expect(screen.getAllByText('取引明細').length).toBeGreaterThan(0);
    expect(screen.getByText('CSV取込')).toBeInTheDocument();
  });

  it('ステータスストリップの各タイルにセカンダリ行を表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    expect(screen.getByText('検索')).toBeInTheDocument();
    expect(screen.getByText('一覧確認')).toBeInTheDocument();
    expect(screen.getByText('明細確認')).toBeInTheDocument();
    expect(screen.getByText('CSV反映')).toBeInTheDocument();
  });

  it('CSV取込タイルはリンクではなく、各ページから取込可能と表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    expect(screen.getByText('各ページから取込可能')).toBeInTheDocument();
    // /csv へのリンクは存在しない
    const allLinks = screen.getAllByRole('link');
    expect(allLinks.every((l) => l.getAttribute('href') !== '/csv')).toBe(true);
  });

  it('ストリップの銘柄検索・資産管理・取引明細タイルがリンクになっている', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    const searchLinks = screen.getAllByRole('link').filter((l) => l.getAttribute('href') === '/search');
    expect(searchLinks.length).toBeGreaterThan(0);

    const assetLinks = screen.getAllByRole('link').filter((l) => l.getAttribute('href') === '/assetbalance');
    expect(assetLinks.length).toBeGreaterThan(0);

    const receiptLinks = screen.getAllByRole('link').filter((l) => l.getAttribute('href') === '/receipts');
    expect(receiptLinks.length).toBeGreaterThan(0);
  });

  it('次に行う操作セクションのリンクを表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    const nextSection = screen
      .getByRole('heading', { name: '次に行う操作' })
      .parentElement;

    expect(nextSection).not.toBeNull();

    const scoped = within(nextSection as HTMLElement);

    expect(scoped.getByRole('link', { name: '銘柄検索' })).toHaveAttribute('href', '/search');
    expect(scoped.getByRole('link', { name: '資産管理' })).toHaveAttribute('href', '/assetbalance');
    expect(scoped.getByRole('link', { name: '取引明細' })).toHaveAttribute('href', '/receipts');
  });

  it('データ確認フローセクションのステップを表示する', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>
    );

    const flowSection = screen
      .getByRole('heading', { name: 'データ確認フロー' })
      .parentElement;

    expect(flowSection).not.toBeNull();

    const scoped = within(flowSection as HTMLElement);

    expect(scoped.getByText('証券会社からCSVをダウンロード')).toBeInTheDocument();
    expect(scoped.getByText('各ページのCSV取込から反映')).toBeInTheDocument();
    expect(scoped.getByText('資産・配当金・取引明細を確認')).toBeInTheDocument();
  });
});
