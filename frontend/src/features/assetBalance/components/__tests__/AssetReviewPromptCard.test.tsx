import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { AssetBalanceData } from '@/types/api';
import { AssetReviewPromptCard } from '../AssetReviewPromptCard';

function makeAsset(overrides: Partial<AssetBalanceData> = {}): AssetBalanceData {
  return {
    id: 'id-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '7203',
    security_name: 'トヨタ自動車',
    shares: 100,
    executing_shares: 0,
    average_purchase_price: 2500,
    total_purchase_amount: 250000,
    current_price: 2600,
    daily_change: 50,
    market_value: 260000,
    profit_loss_rate: 4.0,
    ...overrides,
  };
}

const singleAsset = [makeAsset()];

describe('AssetReviewPromptCard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('保有データありのときコピーボタンを表示する', () => {
    render(
      <AssetReviewPromptCard
        assetBalanceData={singleAsset}
        dividendPerShareMap={new Map()}
      />
    );
    expect(screen.getByRole('button', { name: /AI総評プロンプトをコピー/ })).toBeInTheDocument();
  });

  it('保有データ0件のときボタンをdisabledにする', () => {
    render(
      <AssetReviewPromptCard
        assetBalanceData={[]}
        dividendPerShareMap={new Map()}
      />
    );
    expect(screen.getByRole('button', { name: /AI総評プロンプトをコピー/ })).toBeDisabled();
  });

  it('コピー成功時に成功メッセージを表示する', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });

    render(
      <AssetReviewPromptCard
        assetBalanceData={singleAsset}
        dividendPerShareMap={new Map()}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /AI総評プロンプトをコピー/ }));

    await waitFor(() => {
      expect(screen.getByRole('status')).toHaveTextContent('コピーしました');
    });
    expect(writeText).toHaveBeenCalledTimes(1);
  });

  it('コピー成功時に writeText が1回呼ばれる', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });

    render(
      <AssetReviewPromptCard
        assetBalanceData={singleAsset}
        dividendPerShareMap={new Map()}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /AI総評プロンプトをコピー/ }));

    await waitFor(() => expect(writeText).toHaveBeenCalledTimes(1));
    const [calledText] = writeText.mock.calls[0] as [string];
    expect(calledText).toContain('トヨタ自動車');
  });

  it('Clipboard API 失敗時にエラーメッセージを表示する', async () => {
    const writeText = vi.fn().mockRejectedValue(new Error('denied'));
    Object.assign(navigator, { clipboard: { writeText } });

    render(
      <AssetReviewPromptCard
        assetBalanceData={singleAsset}
        dividendPerShareMap={new Map()}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /AI総評プロンプトをコピー/ }));

    await waitFor(() => {
      expect(screen.getByText(/コピーに失敗しました/)).toBeInTheDocument();
    });
  });

  it('aria-live 領域が存在する', () => {
    render(
      <AssetReviewPromptCard
        assetBalanceData={singleAsset}
        dividendPerShareMap={new Map()}
      />
    );
    const liveRegion = screen.getByRole('status');
    expect(liveRegion).toHaveAttribute('aria-live');
  });
});
