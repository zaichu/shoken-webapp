import { describe, expect, it } from 'vitest';
import { calculateValuation, summarizeValuation } from '../valuation';

describe('評価損益', () => {
  it('金額から率を再計算する', () => {
    expect(calculateValuation(1180000, 1090000)).toEqual({ amount: 90000, rate: 90000 / 1090000 * 100 });
    expect(calculateValuation(0, 100)).toEqual({ amount: -100, rate: -100 });
    expect(calculateValuation(100, 0)).toEqual({ amount: 100, rate: null });
  });
  it.each([null, undefined, NaN, Infinity, '', '-'])('欠損 %s を0として計算しない', value => {
    expect(calculateValuation(value, 100)).toEqual({ amount: null, rate: null });
    expect(calculateValuation(100, value)).toEqual({ amount: null, rate: null });
  });
  it('欠損を含む合計は不完全とし金額と率を表示しない', () => {
    expect(summarizeValuation([{ market_value: 100, total_purchase_amount: 80 }, { market_value: null, total_purchase_amount: 20 }])).toEqual({ marketValue: null, amount: null, rate: null, incomplete: true });
  });
  it('合計金額から率を計算し単純平均しない', () => {
    expect(summarizeValuation([{ market_value: 200, total_purchase_amount: 100 }, { market_value: 900, total_purchase_amount: 900 }])).toEqual({ marketValue: 1100, amount: 100, rate: 10, incomplete: false });
  });
});
