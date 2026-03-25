import { describe, expect, it } from 'vitest';

import {
  sortDividendBySettlementDate,
  transformDBDividend,
} from '../dividendParser';

const toRecord = (value: unknown): Record<string, unknown> =>
  value as Record<string, unknown>;

const createZeroLikeInput = (shape: Record<string, unknown>) => {
  const input: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(shape)) {
    if (value instanceof Date) {
      input[key] = '2024-12-31';
      continue;
    }

    if (typeof value === 'number') {
      input[key] = 0;
      continue;
    }

    if (typeof value === 'string') {
      input[key] = '';
      continue;
    }

    throw new Error(`Unexpected field type for ${key}`);
  }

  return input;
};

describe('dividendParser', () => {
  describe('sortDividendBySettlementDate', () => {
    it('sorts dividends by settlement_date in ascending order', () => {
      const records = [
        transformDBDividend({ settlement_date: '2024-03-31' }),
        transformDBDividend({ settlement_date: '2024-01-31' }),
        transformDBDividend({ settlement_date: '2024-02-29' }),
      ];

      const sorted = sortDividendBySettlementDate(records).map((item) =>
        (toRecord(item).settlement_date as Date).toISOString(),
      );

      expect(sorted).toEqual([
        new Date('2024-01-31').toISOString(),
        new Date('2024-02-29').toISOString(),
        new Date('2024-03-31').toISOString(),
      ]);
    });
  });

  describe('transformDBDividend', () => {
    it('transforms record values into dividend data', () => {
      const input: Record<string, unknown> = {
        settlement_date: '2024-01-31',
        product: '株式数比例配分方式',
        account: '特定',
        security_code: '7203',
        security_name: 'トヨタ自動車',
        unit_price: '45',
        shares: '100',
        dividends_before_tax: '4500',
        taxes: '913',
        net_amount_received: '3587',
      };
      const result = toRecord(transformDBDividend(input));

      expect(result.settlement_date).toEqual(new Date('2024-01-31'));
      expect(result.product).toBe('株式数比例配分方式');
      expect(result.account).toBe('特定');
      expect(result.security_code).toBe('7203');
      expect(result.security_name).toBe('トヨタ自動車');
      expect(result.unit_price).toBe(45);
      expect(result.shares).toBe(100);
      expect(result.dividends_before_tax).toBe(4500);
      expect(result.taxes).toBe(913);
      expect(result.net_amount_received).toBe(3587);
    });

    it('returns typed defaults when values are missing', () => {
      const result = toRecord(transformDBDividend({}));

      expect(result).toHaveProperty('settlement_date');
      expect(result.settlement_date).toBeInstanceOf(Date);

      for (const [key, value] of Object.entries(result)) {
        if (key === 'settlement_date') {
          continue;
        }

        expect(typeof value === 'number' || typeof value === 'string').toBe(
          true,
        );

        if (typeof value === 'number') {
          expect(value).toBe(0);
        }
      }
    });

    it('keeps zero-like values after transformation', () => {
      const emptyShape = toRecord(transformDBDividend({}));
      const input = createZeroLikeInput(emptyShape);
      const result = toRecord(transformDBDividend(input));

      for (const [key, defaultValue] of Object.entries(emptyShape)) {
        if (defaultValue instanceof Date) {
          expect(result[key]).toEqual(new Date('2024-12-31'));
          continue;
        }

        if (typeof defaultValue === 'number') {
          expect(result[key]).toBe(0);
          continue;
        }

        if (typeof defaultValue === 'string') {
          expect(result[key]).toBe('');
          continue;
        }

        throw new Error(`Unexpected field type for ${key}`);
      }
    });
  });
});
