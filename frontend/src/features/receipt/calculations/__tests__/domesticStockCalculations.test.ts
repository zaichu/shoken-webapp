import { describe, expect, it } from 'vitest';

import type { DomesticStockData, DomesticStockSummary } from '@/lib/interfaces/domesticStock';

import {
  calculateDailyData,
  calculateDomesticStock,
} from '../domesticStockCalculations';

const createDomesticStockData = (
  overrides: Partial<DomesticStockData> = {},
): DomesticStockData => ({
  settlement_date: new Date(2024, 2, 4),
  trade_date: new Date(2024, 2, 1),
  security_code: '1234',
  security_name: 'テスト株',
  account: '特定口座',
  shares: 100,
  asked_price: 1200,
  proceeds: 120000,
  purchase_price: 100000,
  realized_profit_and_loss: 20000,
  taxes: 0,
  realized_profit_and_loss_after_tax: 20000,
  ...overrides,
});

const createDailySummary = (
  overrides: Partial<DomesticStockSummary> = {},
): DomesticStockSummary => ({
  filter: '2024-03-01',
  total_realized_profit_and_loss: 1000,
  total_taxes: 203,
  total_realized_profit_and_loss_after_tax: 797,
  ...overrides,
});

describe('calculateDailyData', () => {
  it('同一日付の複数件を1件のサマリーに集約する', () => {
    const result = calculateDailyData([
      createDomesticStockData({
        security_code: '1111',
        realized_profit_and_loss: 1000,
        proceeds: 10000,
      }),
      createDomesticStockData({
        security_code: '2222',
        realized_profit_and_loss: 2000,
        proceeds: 20000,
      }),
    ]);

    expect(result).toEqual([
      {
        filter: '2024-03-01',
        total_realized_profit_and_loss: 3000,
        total_taxes: 609,
        total_realized_profit_and_loss_after_tax: 2391,
      },
    ]);
  });

  it('特定口座で利益がある場合は税金を計算する', () => {
    const result = calculateDailyData([
      createDomesticStockData({
        realized_profit_and_loss: 10000,
      }),
    ]);

    expect(result[0]).toEqual({
      filter: '2024-03-01',
      total_realized_profit_and_loss: 10000,
      total_taxes: 2031,
      total_realized_profit_and_loss_after_tax: 7969,
    });
  });

  it('特定口座で損失の場合は税金が0になる', () => {
    const result = calculateDailyData([
      createDomesticStockData({
        realized_profit_and_loss: -10000,
      }),
    ]);

    expect(result[0]).toEqual({
      filter: '2024-03-01',
      total_realized_profit_and_loss: -10000,
      total_taxes: 0,
      total_realized_profit_and_loss_after_tax: -10000,
    });
  });

  it('NISA口座は利益があっても税金が0になる', () => {
    const result = calculateDailyData([
      createDomesticStockData({
        account: 'NISA口座',
        realized_profit_and_loss: 10000,
      }),
    ]);

    expect(result[0]).toEqual({
      filter: '2024-03-01',
      total_realized_profit_and_loss: 10000,
      total_taxes: 0,
      total_realized_profit_and_loss_after_tax: 10000,
    });
  });

  it('特定口座とNISA口座が混在する場合は特定口座分のみ課税する', () => {
    const result = calculateDailyData([
      createDomesticStockData({
        account: '特定口座',
        realized_profit_and_loss: 10000,
      }),
      createDomesticStockData({
        account: 'NISA口座',
        security_code: '5678',
        realized_profit_and_loss: 5000,
      }),
    ]);

    expect(result[0]).toEqual({
      filter: '2024-03-01',
      total_realized_profit_and_loss: 15000,
      total_taxes: 2031,
      total_realized_profit_and_loss_after_tax: 12969,
    });
  });

  it('別日付のデータは日付昇順のサマリーで返す', () => {
    const result = calculateDailyData([
      createDomesticStockData({
        trade_date: new Date(2024, 2, 2),
        settlement_date: new Date(2024, 2, 5),
        realized_profit_and_loss: 2000,
      }),
      createDomesticStockData({
        trade_date: new Date(2024, 2, 1),
        settlement_date: new Date(2024, 2, 4),
        security_code: '5678',
        realized_profit_and_loss: 1000,
      }),
    ]);

    expect(result).toEqual([
      {
        filter: '2024-03-01',
        total_realized_profit_and_loss: 1000,
        total_taxes: 203,
        total_realized_profit_and_loss_after_tax: 797,
      },
      {
        filter: '2024-03-02',
        total_realized_profit_and_loss: 2000,
        total_taxes: 406,
        total_realized_profit_and_loss_after_tax: 1594,
      },
    ]);
  });
});

describe('calculateDomesticStock', () => {
  it('空配列の場合は全フィールド0を返す', () => {
    expect(calculateDomesticStock([])).toEqual({
      total_realized_profit_and_loss: 0,
      total_taxes: 0,
      total_realized_profit_and_loss_after_tax: 0,
    });
  });

  it('複数サマリーの各フィールドを合計する', () => {
    const result = calculateDomesticStock([
      createDailySummary({
        total_realized_profit_and_loss: 10000,
        total_taxes: 2031,
        total_realized_profit_and_loss_after_tax: 7969,
      }),
      createDailySummary({
        filter: '2024-03-02',
        total_realized_profit_and_loss: -3000,
        total_taxes: 0,
        total_realized_profit_and_loss_after_tax: -3000,
      }),
      createDailySummary({
        filter: '2024-03-03',
        total_realized_profit_and_loss: 5000,
        total_taxes: 1015,
        total_realized_profit_and_loss_after_tax: 3985,
      }),
    ]);

    expect(result).toEqual({
      total_realized_profit_and_loss: 12000,
      total_taxes: 3046,
      total_realized_profit_and_loss_after_tax: 8954,
    });
  });
});
