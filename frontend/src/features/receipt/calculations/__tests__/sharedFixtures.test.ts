import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import type {
  DividendData,
  DomesticStockData,
  MutualfundData,
} from '@/features/receipt/types';

import { calculateDividends } from '../dividendCalculations';
import {
  calculateDailyData,
  calculateDomesticStock,
} from '../domesticStockCalculations';
import { calculateMutualfund } from '../mutualfundCalculations';

const fixtureDirectory = resolve(
  process.cwd(),
  '../frontend-leptos/tests/fixtures/receipts',
);

const readFixture = <T>(name: string): T =>
  JSON.parse(readFileSync(resolve(fixtureDirectory, name), 'utf8')) as T;

interface DomesticInputRow {
  trade_date: string;
  account: string;
  realized_profit_and_loss: number;
  taxes: number;
}

interface DomesticInputCase {
  name: string;
  rows: DomesticInputRow[];
}

interface ExpectedDomesticCase {
  name: string;
  daily: ReturnType<typeof calculateDailyData>;
  total: ReturnType<typeof calculateDomesticStock>;
  row_taxes_total: number;
}

interface SimpleInputCase<T> {
  name: string;
  rows: T[];
}

interface ExpectedCase<T> {
  name: string;
  total: T;
}

const domesticRow = (row: DomesticInputRow): DomesticStockData => ({
  trade_date: new Date(`${row.trade_date}T00:00:00`),
  settlement_date: new Date(`${row.trade_date}T00:00:00`),
  security_code: '',
  security_name: '',
  account: row.account,
  shares: 0,
  asked_price: 0,
  proceeds: 0,
  purchase_price: 0,
  realized_profit_and_loss: row.realized_profit_and_loss,
  taxes: row.taxes,
  realized_profit_and_loss_after_tax:
    row.realized_profit_and_loss - row.taxes,
});

describe('shared receipt calculation fixtures', () => {
  it('matches domestic daily and total calculations', () => {
    const input = readFixture<{ cases: DomesticInputCase[] }>(
      'domestic-input.json',
    );
    const expected = readFixture<{ cases: ExpectedDomesticCase[] }>(
      'domestic-expected.json',
    );

    expect(
      input.cases.map(({ name, rows }) => {
        const data = rows.map(domesticRow);
        const daily = calculateDailyData(data);
        return {
          name,
          daily,
          total: calculateDomesticStock(daily),
          row_taxes_total: data.reduce((sum, row) => sum + row.taxes, 0),
        };
      }),
    ).toEqual(expected.cases);
  });

  it('matches dividend calculations', () => {
    type Row = Pick<
      DividendData,
      'dividends_before_tax' | 'taxes' | 'net_amount_received'
    >;
    const input = readFixture<{ cases: SimpleInputCase<Row>[] }>(
      'dividend-input.json',
    );
    const expected = readFixture<{
      cases: ExpectedCase<ReturnType<typeof calculateDividends>>[];
    }>('dividend-expected.json');

    expect(
      input.cases.map(({ name, rows }) => ({
        name,
        total: calculateDividends(rows as DividendData[]),
      })),
    ).toEqual(expected.cases);
  });

  it('matches mutual fund calculations', () => {
    type Row = Pick<
      MutualfundData,
      | 'realized_profit_and_loss'
      | 'taxes'
      | 'realized_profit_and_loss_after_tax'
    >;
    const input = readFixture<{ cases: SimpleInputCase<Row>[] }>(
      'mutualfund-input.json',
    );
    const expected = readFixture<{
      cases: ExpectedCase<ReturnType<typeof calculateMutualfund>>[];
    }>('mutualfund-expected.json');

    expect(
      input.cases.map(({ name, rows }) => ({
        name,
        total: calculateMutualfund(rows as MutualfundData[]),
      })),
    ).toEqual(expected.cases);
  });
});
