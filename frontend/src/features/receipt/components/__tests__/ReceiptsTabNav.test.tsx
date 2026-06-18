import { fireEvent, render, screen } from '@testing-library/react';
import { createRef } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { ReceiptsTabNav } from '../ReceiptsTabNav';

describe('ReceiptsTabNav', () => {
  const counts = {
    dividend: 3,
    domesticstock: 5,
    mutualfund: 2,
  };

  it('タブが3つ表示され、アクティブタブに aria-selected=true が設定される', () => {
    render(
      <ReceiptsTabNav
        receiptsType="domesticstock"
        tablistRef={createRef<HTMLDivElement>()}
        onTabChange={vi.fn()}
        onKeyDown={vi.fn()}
        counts={counts}
      />
    );

    const tabs = screen.getAllByRole('tab');

    expect(tabs).toHaveLength(3);
    expect(screen.getByRole('tab', { name: /配当金/ })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /国内株式/ })).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByRole('tab', { name: /投資信託/ })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /配当金/ })).toHaveAttribute('aria-selected', 'false');
    expect(screen.getByRole('tab', { name: /投資信託/ })).toHaveAttribute('aria-selected', 'false');
  });

  it('各タブに件数バッジが表示される', () => {
    render(
      <ReceiptsTabNav
        receiptsType="dividend"
        tablistRef={createRef<HTMLDivElement>()}
        onTabChange={vi.fn()}
        onKeyDown={vi.fn()}
        counts={counts}
      />
    );

    expect(screen.getByTestId('tab-count-dividend')).toHaveTextContent('3');
    expect(screen.getByTestId('tab-count-domesticstock')).toHaveTextContent('5');
    expect(screen.getByTestId('tab-count-mutualfund')).toHaveTextContent('2');
  });

  it('非選択タブと件数バッジは淡い背景でも十分なコントラストの文字色を使う', () => {
    render(
      <ReceiptsTabNav
        receiptsType="domesticstock"
        tablistRef={createRef<HTMLDivElement>()}
        onTabChange={vi.fn()}
        onKeyDown={vi.fn()}
        counts={counts}
      />
    );

    const inactiveTab = screen.getByRole('tab', { name: /配当金/ });

    expect(inactiveTab).toHaveClass('text-slate-800');
    expect(screen.getByTestId('tab-count-dividend')).toHaveClass('text-slate-700');
  });

  it('タブクリックで onTabChange コールバックが呼ばれる', () => {
    const onTabChange = vi.fn();

    render(
      <ReceiptsTabNav
        receiptsType="dividend"
        tablistRef={createRef<HTMLDivElement>()}
        onTabChange={onTabChange}
        onKeyDown={vi.fn()}
        counts={counts}
      />
    );

    fireEvent.click(screen.getByRole('tab', { name: /投資信託/ }));

    expect(onTabChange).toHaveBeenCalledWith('mutualfund');
    expect(onTabChange).toHaveBeenCalledTimes(1);
  });
});
