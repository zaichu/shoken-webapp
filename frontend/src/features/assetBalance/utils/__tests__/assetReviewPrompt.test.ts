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

  it('金融商品取引業者・投資顧問の助言ではないこと、売買指示・断定的推奨を避けることを明記する', () => {
    const result = generateAssetReviewPrompt(singleAsset);
    expect(result).toMatch(/金融商品取引業者|投資顧問/);
    expect(result).toMatch(/売買指示|断定的推奨/);
  });

  it('リサーチアシスタントの役割指定を含む', () => {
    const result = generateAssetReviewPrompt(singleAsset);
    expect(result).toMatch(/リサーチアシスタント/);
  });

  it('全銘柄コードと銘柄名を含む', () => {
    const result = generateAssetReviewPrompt(multiAssets);
    expect(result).toContain('7203');
    expect(result).toContain('トヨタ自動車');
    expect(result).toContain('6758');
    expect(result).toContain('ソニーグループ');
  });

  it('集計セクションを含まず、保有銘柄テーブルだけを入力情報として渡す', () => {
    const result = generateAssetReviewPrompt(multiAssets);
    expect(result).not.toContain('集計:');
    expect(result).not.toContain('保有銘柄数');
  });

  it('保有株数・平均取得単価をテーブルに含む', () => {
    const result = generateAssetReviewPrompt(singleAsset);
    expect(result).toMatch(/100/);           // shares
    expect(result).toMatch(/2,500|2500/);    // average_purchase_price
  });

  it('テーブルヘッダーに現在値・取得額・評価額・評価損益額・損益率・推定1株配当を含まない', () => {
    const result = generateAssetReviewPrompt(singleAsset);
    const headerLine = result.split('\n').find((line) => line.startsWith('銘柄コード'));
    expect(headerLine).toBeDefined();
    expect(headerLine).not.toContain('現在値');
    expect(headerLine).not.toContain('取得額');
    expect(headerLine).not.toContain('評価額');
    expect(headerLine).not.toContain('評価損益額');
    expect(headerLine).not.toContain('損益率');
    expect(headerLine).not.toContain('推定1株配当');
  });

  it('最新の公開情報を確認するよう外部AIへ指示する', () => {
    const result = generateAssetReviewPrompt(singleAsset);
    expect(result).toMatch(/公開情報/);
    expect(result).toMatch(/最新/);
    expect(result).toMatch(/株価|配当|業績|ニュース/);
  });

  it('投資目的・期間・リスク許容度等の不足情報を追加質問として列挙するよう指示する', () => {
    const result = generateAssetReviewPrompt(singleAsset);
    expect(result).toMatch(/投資目的/);
    expect(result).toMatch(/リスク許容度/);
    expect(result).toMatch(/追加質問/);
  });

  it('データが空の場合でもエラーを出さず空プロンプトを返す', () => {
    expect(() => generateAssetReviewPrompt([])).not.toThrow();
    const result = generateAssetReviewPrompt([]);
    expect(typeof result).toBe('string');
  });

  it('個人識別情報（メール・ユーザーID）を含まない', () => {
    const result = generateAssetReviewPrompt(multiAssets);
    expect(result).not.toMatch(/@/);
    expect(result).not.toMatch(/user_id|userId/);
  });

  it('外部AIへ不足情報の質問を促す文言を含む', () => {
    const result = generateAssetReviewPrompt(singleAsset);
    expect(result).toMatch(/質問|不足|情報/);
  });
});
