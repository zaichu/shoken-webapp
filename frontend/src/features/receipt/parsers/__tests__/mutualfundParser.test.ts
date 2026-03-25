import { describe, expect, it } from 'vitest';

import {
  sortMutualfundByTradeDate,
  transformDBMutualfund,
} from '../mutualfundParser';

const toRecord = (value: unknown): Record<string, unknown> =>
  value as Record<string, unknown>;

const createZeroLikeInput = (shape: Record<string, unknown>) => {
  const input: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(shape)) {
    if (value instanceof Date) {
      input[key] = '2024-10-31';
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

describe('mutualfundParser', () => {
  describe('sortMutualfundByTradeDate', () => {
    it('sorts mutualfund records by trade_date in ascending order', () => {
      const records = [
        transformDBMutualfund({ trade_date: '2024-03-12' }),
        transformDBMutualfund({ trade_date: '2024-01-05' }),
        transformDBMutualfund({ trade_date: '2024-02-18' }),
      ];

      const sorted = sortMutualfundByTradeDate(records).map((item) =>
        (toRecord(item).trade_date as Date).toISOString(),
      );

      expect(sorted).toEqual([
        new Date('2024-01-05').toISOString(),
        new Date('2024-02-18').toISOString(),
        new Date('2024-03-12').toISOString(),
      ]);
    });
  });

  describe('transformDBMutualfund', () => {
    it('transforms record values into mutualfund data', () => {
      const input: Record<string, unknown> = {
        trade_date: '2024-03-01',
        settlement_date: '2024-03-05',
        fund_name: 'test-fund',
        dividends: '0',
        account: 'tokutei',
        shares: '1000',
        exchange_rate: '150',
        cancellation_unit_price_yen: '12000',
        cancellation_amount_yen: '12000000',
        average_acquisition_price_yen: '11000',
        realized_profit_and_loss: '1000000',
        taxes: '203150',
        realized_profit_and_loss_after_tax: '796850',
      };
      const result = toRecord(transformDBMutualfund(input));

      expect(result.trade_date).toEqual(new Date('2024-03-01'));
      expect(result.settlement_date).toEqual(new Date('2024-03-05'));
      expect(result.fund_name).toBe('test-fund');
      expect(result.dividends).toBe('0');
      expect(result.account).toBe('tokutei');
      expect(result.shares).toBe(1000);
      expect(result.exchange_rate).toBe(150);
      expect(result.cancellation_unit_price_yen).toBe(12000);
      expect(result.cancellation_amount_yen).toBe(12000000);
      expect(result.average_acquisition_price_yen).toBe(11000);
      expect(result.realized_profit_and_loss).toBe(1000000);
      expect(result.taxes).toBe(203150);
      expect(result.realized_profit_and_loss_after_tax).toBe(796850);
    });

    it('returns typed defaults when values are missing', () => {
      const result = toRecord(transformDBMutualfund({}));

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
      }
    });

    it('keeps zero-like values after transformation', () => {
      const emptyShape = toRecord(transformDBMutualfund({}));
      const input = createZeroLikeInput(emptyShape);
      const result = toRecord(transformDBMutualfund(input));

      expect(result.trade_date).toEqual(new Date('2024-10-31'));
      expect(result.settlement_date).toEqual(new Date('2024-10-31'));
      expect(result.fund_name).toBe('');
      expect(result.dividends).toBe('');
      expect(result.account).toBe('');

      for (const [key, defaultValue] of Object.entries(emptyShape)) {
        if (
          key === 'trade_date' ||
          key === 'settlement_date' ||
          key === 'fund_name' ||
          key === 'dividends' ||
          key === 'account'
        ) {
          continue;
        }

        if (defaultValue instanceof Date) {
          expect(result[key as keyof typeof result]).toEqual(
            new Date('2024-10-31'),
          );
          continue;
        }

        if (typeof defaultValue === 'number') {
          expect(result[key as keyof typeof result]).toBe(0);
          continue;
        }

        if (typeof defaultValue === 'string') {
          expect(result[key as keyof typeof result]).toBe('');
          continue;
        }

        throw new Error(`Unexpected field type for ${key}`);
      }
    });
  });
});
