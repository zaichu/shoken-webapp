import { describe, expect, it } from 'vitest';

import {
  sortDomesticStockByTradeDate,
  transformDBDomesticStock,
} from '../domesticStockParser';

const toRecord = (value: unknown): Record<string, unknown> =>
  value as Record<string, unknown>;

const createFilledInput = (shape: Record<string, unknown>) => {
  const input: Record<string, unknown> = {};
  let numberSeed = 1;
  let dateSeed = 1;

  for (const [key, value] of Object.entries(shape)) {
    if (value instanceof Date) {
      input[key] = `2024-02-${String(dateSeed).padStart(2, '0')}`;
      dateSeed += 1;
      continue;
    }

    if (typeof value === 'number') {
      input[key] = String(200 + numberSeed);
      numberSeed += 1;
      continue;
    }

    if (typeof value === 'string') {
      input[key] = `value-${key}`;
      continue;
    }

    throw new Error(`Unexpected field type for ${key}`);
  }

  return input;
};

const createZeroLikeInput = (shape: Record<string, unknown>) => {
  const input: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(shape)) {
    if (value instanceof Date) {
      input[key] = '2024-11-30';
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

describe('domesticStockParser', () => {
  describe('sortDomesticStockByTradeDate', () => {
    it('sorts domestic stock records by trade_date in ascending order', () => {
      const records = [
        transformDBDomesticStock({ trade_date: '2024-03-15' }),
        transformDBDomesticStock({ trade_date: '2024-01-10' }),
        transformDBDomesticStock({ trade_date: '2024-02-20' }),
      ];

      const sorted = sortDomesticStockByTradeDate(records).map((item) =>
        (toRecord(item).trade_date as Date).toISOString(),
      );

      expect(sorted).toEqual([
        new Date('2024-01-10').toISOString(),
        new Date('2024-02-20').toISOString(),
        new Date('2024-03-15').toISOString(),
      ]);
    });
  });

  describe('transformDBDomesticStock', () => {
    it('transforms record values into domestic stock data', () => {
      const emptyShape = toRecord(transformDBDomesticStock({}));
      const input = createFilledInput(emptyShape);
      const result = toRecord(transformDBDomesticStock(input));

      for (const [key, defaultValue] of Object.entries(emptyShape)) {
        if (defaultValue instanceof Date) {
          expect(result[key]).toEqual(new Date(String(input[key])));
          continue;
        }

        if (typeof defaultValue === 'number') {
          expect(result[key]).toBe(Number(input[key]));
          continue;
        }

        if (typeof defaultValue === 'string') {
          expect(result[key]).toBe(String(input[key]));
          continue;
        }

        throw new Error(`Unexpected field type for ${key}`);
      }
    });

    it('returns typed defaults when values are missing', () => {
      const result = toRecord(transformDBDomesticStock({}));

      expect(result).toHaveProperty('trade_date');
      expect(result.trade_date).toBeInstanceOf(Date);
      expect(result).toHaveProperty('settlement_date');
      expect(result.settlement_date).toBeInstanceOf(Date);

      for (const [key, value] of Object.entries(result)) {
        if (key === 'trade_date' || key === 'settlement_date') {
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
      const emptyShape = toRecord(transformDBDomesticStock({}));
      const input = createZeroLikeInput(emptyShape);
      const result = toRecord(transformDBDomesticStock(input));

      for (const [key, defaultValue] of Object.entries(emptyShape)) {
        if (defaultValue instanceof Date) {
          expect(result[key]).toEqual(new Date('2024-11-30'));
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
