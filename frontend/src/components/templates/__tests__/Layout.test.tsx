import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Layout } from '../Layout';

vi.mock('@/components/organisms/Header', () => ({
  Header: () => <div data-testid="header" />,
}));
vi.mock('@/components/organisms/Footer', () => ({
  Footer: () => <div data-testid="footer" />,
}));

describe('Layout', () => {
  it('children が描画される', () => {
    render(<Layout><div>テストコンテンツ</div></Layout>);
    expect(screen.getByText('テストコンテンツ')).toBeInTheDocument();
  });

  it('id="main-content" の要素が存在する', () => {
    render(<Layout><div>コンテンツ</div></Layout>);
    expect(document.getElementById('main-content')).toBeInTheDocument();
  });

  it('スキップリンク「メインコンテンツへスキップ」が存在する', () => {
    render(<Layout><div>コンテンツ</div></Layout>);
    expect(screen.getByText('メインコンテンツへスキップ')).toBeInTheDocument();
  });

  it('Header と Footer がレンダリングされる', () => {
    render(<Layout><div>コンテンツ</div></Layout>);
    expect(screen.getByTestId('header')).toBeInTheDocument();
    expect(screen.getByTestId('footer')).toBeInTheDocument();
  });
});
