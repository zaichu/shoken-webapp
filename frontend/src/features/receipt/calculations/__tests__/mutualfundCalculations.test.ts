import { describe, expect, it } from 'vitest';

import type { MutualfundData } from '@/features/receipt/types';

import { calculateMutualfund } from '../mutualfundCalculations';

const createMutualfundData = (overrides: Partial<MutualfundData> = {}): MutualfundData => ({
  settlement_date: new Date(2024, 2, 5),
  trade_date: new Date(2024, 2, 1),
  fund_name: 'テスト投資信託',
  dividends: '再投資',
  account: '特定口座',
  shares: 1000,
  exchange_rate: 1,
  cancellation_unit_price_yen: 12000,
  cancellation_amount_yen: 1200000,
  average_acquisition_price_yen: 10000,
  realized_profit_and_loss: 200000,
  taxes: 40630,
  realized_profit_and_loss_after_tax: 159370,
  ...overrides,
});

describe('calculateMutualfund', () => {
  it('空配列の場合は全フィールド0を返す', () => {
    expect(calculateMutualfund([])).toEqual({
      total_realized_profit_and_loss: 0,
      total_taxes: 0,
      total_realized_profit_and_loss_after_tax: 0,
    });
  });

  it('1件の場合は値をそのまま返す', () => {
    const data = createMutualfundData({
      realized_profit_and_loss: 50000,
      taxes: 10157,
      realized_profit_and_loss_after_tax: 39843,
    });

    expect(calculateMutualfund([data])).toEqual({
      total_realized_profit_and_loss: 50000,
      total_taxes: 10157,
      total_realized_profit_and_loss_after_tax: 39843,
    });
  });

  it('複数件の場合は各フィールドを合計する', () => {
    const data = [
      createMutualfundData({
        fund_name: 'ファンドA',
        realized_profit_and_loss: 50000,
        taxes: 10157,
        realized_profit_and_loss_after_tax: 39843,
      }),
      createMutualfundData({
        fund_name: 'ファンドB',
        realized_profit_and_loss: -10000,
        taxes: 0,
        realized_profit_and_loss_after_tax: -10000,
      }),
      createMutualfundData({
        fund_name: 'ファンドC',
        realized_profit_and_loss: 25000,
        taxes: 5078,
        realized_profit_and_loss_after_tax: 19922,
      }),
    ];

    expect(calculateMutualfund(data)).toEqual({
      total_realized_profit_and_loss: 65000,
      total_taxes: 15235,
      total_realized_profit_and_loss_after_tax: 49765,
    });
  });
});
