import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect } from 'vitest';
import { ErrorPage } from '../ErrorPage';

function renderErrorPage(props?: React.ComponentProps<typeof ErrorPage>) {
  return render(
    <MemoryRouter>
      <ErrorPage {...props} />
    </MemoryRouter>
  );
}

describe('ErrorPage', () => {
  it('デフォルト props でタイトルとメッセージが表示される', () => {
    renderErrorPage();

    expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();
    expect(
      screen.getByText('アプリケーションで問題が発生しました。もう一度お試しください。')
    ).toBeInTheDocument();
  });

  it('カスタム title と message が反映される', () => {
    renderErrorPage({ title: '404 Not Found', message: 'ページが見つかりません' });

    expect(screen.getByText('404 Not Found')).toBeInTheDocument();
    expect(screen.getByText('ページが見つかりません')).toBeInTheDocument();
  });

  it('showHomeButton=true（デフォルト）のとき「ホームに戻る」ボタンが表示される', () => {
    renderErrorPage();

    expect(screen.getByRole('button', { name: 'ホームに戻る' })).toBeInTheDocument();
  });

  it('showHomeButton=false のとき「ホームに戻る」ボタンが非表示', () => {
    renderErrorPage({ showHomeButton: false });

    expect(screen.queryByRole('button', { name: 'ホームに戻る' })).not.toBeInTheDocument();
  });

  it('showRelatedLinks=true のとき「ホーム」「銘柄検索」「取引明細」の3リンクが表示される', () => {
    renderErrorPage({ showRelatedLinks: true });

    expect(screen.getByRole('link', { name: 'ホーム' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: '銘柄検索' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: '取引明細' })).toBeInTheDocument();
  });

  it('showRelatedLinks=false（デフォルト）のとき関連リンクが非表示', () => {
    renderErrorPage();

    expect(screen.queryByRole('link', { name: '銘柄検索' })).not.toBeInTheDocument();
    expect(screen.queryByRole('link', { name: '取引明細' })).not.toBeInTheDocument();
  });
});
