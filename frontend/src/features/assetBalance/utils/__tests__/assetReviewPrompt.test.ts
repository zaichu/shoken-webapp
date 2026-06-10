import { describe, it, expect } from 'vitest';
import type { AssetBalanceData } from '@/types/api';
import { generateAssetReviewPrompt } from '../assetReviewPrompt';

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

describe('generateAssetReviewPrompt', () => {
  const singleAsset = [makeAsset()];
  const multiAssets = [
    makeAsset(),
    makeAsset({
      id: 'id-2',
      security_code: '6758',
      security_name: 'ソニーグループ',
      shares: 50,
      average_purchase_price: 12000,
      total_purchase_amount: 600000,
      current_price: 11500,
      daily_change: -100,
      market_value: 575000,
      profit_loss_rate: -4.17,
    }),
  ];

  it('投資助言ではなく判断材料整理である旨を含む', () => {
    const result = generateAssetReviewPrompt(singleAsset, new Map());
    expect(result).toMatch(/投資助言|売買推奨/);
    expect(result).toMatch(/判断材料/);
  });

  it('ポートフォリオレビューの役割指定を含む', () => {
    const result = generateAssetReviewPrompt(singleAsset, new Map());
    expect(result).toMatch(/ポートフォリオ/);
  });

  it('全銘柄コードと銘柄名を含む', () => {
    const result = generateAssetReviewPrompt(multiAssets, new Map());
    expect(result).toContain('7203');
    expect(result).toContain('トヨタ自動車');
    expect(result).toContain('6758');
    expect(result).toContain('ソニーグループ');
  });

  it('集計値（銘柄数・合計取得金額・合計評価額・評価損益額）を含む', () => {
    const result = generateAssetReviewPrompt(multiAssets, new Map());
    // 銘柄数
    expect(result).toMatch(/2\s*銘柄|銘柄数.+2/);
    // 合計取得金額: 250,000 + 600,000 = 850,000
    expect(result).toMatch(/850,000|850000/);
    // 合計評価額: 260,000 + 575,000 = 835,000
    expect(result).toMatch(/835,000|835000/);
    // 評価損益額: 835,000 - 850,000 = -15,000
    expect(result).toMatch(/-15,000|-15000/);
  });

  it('dividendPerShareMap がある銘柄は配当値を含む', () => {
    const divMap = new Map([['7203', 80]]);
    const result = generateAssetReviewPrompt(singleAsset, divMap);
    expect(result).toMatch(/80/);
  });

  it('dividendPerShareMap にない銘柄は「不明」と表示する', () => {
    const result = generateAssetReviewPrompt(singleAsset, new Map());
    expect(result).toMatch(/不明/);
  });

  it('データが空の場合でもエラーを出さず空プロンプトを返す', () => {
    expect(() => generateAssetReviewPrompt([], new Map())).not.toThrow();
    const result = generateAssetReviewPrompt([], new Map());
    expect(typeof result).toBe('string');
  });

  it('個人識別情報（メール・ユーザーID）を含まない', () => {
    const result = generateAssetReviewPrompt(multiAssets, new Map());
    expect(result).not.toMatch(/@/);
    expect(result).not.toMatch(/user_id|userId/);
  });

  it('保有株数・平均取得単価・現在値・取得額・評価額・損益率を含む', () => {
    const result = generateAssetReviewPrompt(singleAsset, new Map());
    expect(result).toMatch(/100/);   // shares
    expect(result).toMatch(/2,500|2500/); // average_purchase_price
    expect(result).toMatch(/2,600|2600/); // current_price
    expect(result).toMatch(/250,000|250000/); // total_purchase_amount
    expect(result).toMatch(/260,000|260000/); // market_value
    expect(result).toMatch(/4\.0|4\.00/);  // profit_loss_rate
  });

  it('各銘柄の評価損益額（market_value - total_purchase_amount）を含む', () => {
    // トヨタ: 260,000 - 250,000 = 10,000
    const result = generateAssetReviewPrompt(singleAsset, new Map());
    expect(result).toMatch(/10,000|10000/);
  });

  it('外部AIへ不足情報の質問を促す文言を含む', () => {
    const result = generateAssetReviewPrompt(singleAsset, new Map());
    expect(result).toMatch(/質問|不足|情報/);
  });
});
