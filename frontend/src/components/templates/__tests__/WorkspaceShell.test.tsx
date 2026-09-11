import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { WorkspaceShell } from '../WorkspaceShell';

describe('WorkspaceShell', () => {
  it('mainとrailをレンダリングする', () => {
    render(
      <WorkspaceShell
        main={<div>メインコンテンツ</div>}
        rail={<div>サイドバー</div>}
      />
    );

    expect(screen.getByText('メインコンテンツ')).toBeInTheDocument();
    expect(screen.getByText('サイドバー')).toBeInTheDocument();
  });

  it('testIdPrefixを指定するとdata-testidにプレフィックスが付く', () => {
    render(
      <WorkspaceShell
        main={<div>メイン</div>}
        rail={<div>サイド</div>}
        testIdPrefix="receipt"
      />
    );

    expect(screen.getByTestId('receipt-workspace')).toBeInTheDocument();
    expect(screen.getByTestId('receipt-main-stage')).toBeInTheDocument();
    expect(screen.getByTestId('receipt-utility-rail')).toBeInTheDocument();
  });

  it('testIdPrefixなしのデフォルトdata-testidが付く', () => {
    render(
      <WorkspaceShell
        main={<div>メイン</div>}
        rail={<div>サイド</div>}
      />
    );

    expect(screen.getByTestId('workspace')).toBeInTheDocument();
    expect(screen.getByTestId('main-stage')).toBeInTheDocument();
    expect(screen.getByTestId('utility-rail')).toBeInTheDocument();
  });

  it('mainClassNameがmainステージに適用される', () => {
    render(
      <WorkspaceShell
        main={<div>メイン</div>}
        rail={<div>サイド</div>}
        mainClassName="custom-main"
      />
    );

    expect(screen.getByTestId('main-stage')).toHaveClass('custom-main');
  });

  it('railClassNameがutility railに適用される', () => {
    render(
      <WorkspaceShell
        main={<div>メイン</div>}
        rail={<div>サイド</div>}
        railClassName="custom-rail"
      />
    );

    expect(screen.getByTestId('utility-rail')).toHaveClass('custom-rail');
  });

  it('railClassNameなしではutility railにクラスが付かない', () => {
    render(
      <WorkspaceShell
        main={<div>メイン</div>}
        rail={<div>サイド</div>}
      />
    );

    expect(screen.getByTestId('utility-rail').getAttribute('class')).toBeNull();
  });
});
