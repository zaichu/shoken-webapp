import { describe, expect, it } from 'vitest';

import type { DividendData } from '@/lib/interfaces/dividend';

import { calculateDividends } from '../dividendCalculations';

const createDividendData = (overrides: Partial<DividendData> = {}): DividendData => ({
  settlement_date: new Date(2024, 2, 5),
  product: '配当金',
  account: '特定口座',
  security_code: '1234',
  security_name: 'テスト銘柄',
  unit_price: 100,
  shares: 10,
  dividends_before_tax: 1000,
  taxes: 203,
  net_amount_received: 797,
  ...overrides,
});

describe('calculateDividends', () => {
  it('空配列の場合は全フィールド0を返す', () => {
    expect(calculateDividends([])).toEqual({
      total_dividends_before_tax: 0,
      total_taxes: 0,
      total_net_amount_received: 0,
    });
  });

  it('1件の場合は値をそのまま返す', () => {
    const data = createDividendData({
      dividends_before_tax: 5000,
      taxes: 1015,
      net_amount_received: 3985,
    });

    expect(calculateDividends([data])).toEqual({
      total_dividends_before_tax: 5000,
      total_taxes: 1015,
      total_net_amount_received: 3985,
    });
  });

  it('複数件の場合は各フィールドを合計する', () => {
    const data = [
      createDividendData({
        security_code: '1111',
        dividends_before_tax: 1000,
        taxes: 203,
        net_amount_received: 797,
      }),
      createDividendData({
        security_code: '2222',
        dividends_before_tax: 2500,
        taxes: 507,
        net_amount_received: 1993,
      }),
      createDividendData({
        security_code: '3333',
        dividends_before_tax: 1800,
        taxes: 365,
        net_amount_received: 1435,
      }),
    ];

    expect(calculateDividends(data)).toEqual({
      total_dividends_before_tax: 5300,
      total_taxes: 1075,
      total_net_amount_received: 4225,
    });
  });

  it('マイナス値を含んでも正しく加算する', () => {
    const data = [
      createDividendData({
        dividends_before_tax: 1000,
        taxes: 200,
        net_amount_received: 800,
      }),
      createDividendData({
        security_code: '9999',
        dividends_before_tax: -150,
        taxes: -30,
        net_amount_received: -120,
      }),
    ];

    expect(calculateDividends(data)).toEqual({
      total_dividends_before_tax: 850,
      total_taxes: 170,
      total_net_amount_received: 680,
    });
  });
});
