import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import { calculateValuation, summarizeValuation } from '../valuation';
import { calculatePercentage, safeAdd } from '@/lib/utils/formatters';

const fixtureDirectory = resolve(
  process.cwd(),
  '../frontend-leptos/tests/fixtures/asset_balance',
);

const readFixture = <T>(name: string): T =>
  JSON.parse(readFileSync(resolve(fixtureDirectory, name), 'utf8')) as T;

interface ValuationCase {
  name: string;
  market_value: unknown;
  purchase_amount: unknown;
  expected: { amount: number | null; rate: number | null };
}

interface SummaryCase {
  name: string;
  items: { market_value: unknown; total_purchase_amount: unknown }[];
  expected: {
    marketValue: number | null;
    amount: number | null;
    rate: number | null;
    incomplete: boolean;
  };
}

interface CompositionCase {
  name: string;
  values: number[];
  expected_percentages: number[];
}

interface KpiHolding {
  security_code: string;
  shares: number;
  total_purchase_amount: number | null;
}

interface KpiCase {
  name: string;
  holdings: KpiHolding[];
  dividends_per_share: Record<string, number>;
  expected: {
    total_purchase_amount: number;
    total_annual_dividends: number | null;
    dividend_yield: number | null;
    holdings_count: number;
  };
}

interface ValuationFixture {
  valuation_cases: ValuationCase[];
  summary_cases: SummaryCase[];
  composition_cases: CompositionCase[];
  kpi_cases: KpiCase[];
}

/**
 * AssetPortfolioSummary と同じ式で年間配当と配当利回りを求める。
 * 合計取得総額の積み上げには本番と同じ safeAdd を使う。
 */
function calculatePortfolioDividends(
  holdings: KpiHolding[],
  dividendsPerShare: Record<string, number>,
  totalPurchaseAmount: number,
): { totalAnnualDividends: number | null; dividendYield: number | null } {
  const map = new Map(Object.entries(dividendsPerShare));
  if (map.size === 0) {
    return { totalAnnualDividends: null, dividendYield: null };
  }
  let total = 0;
  holdings.forEach((item) => {
    const perShare = map.get(item.security_code);
    if (perShare !== undefined) {
      total += perShare * (item.shares || 0);
    }
  });
  if (total === 0) {
    return { totalAnnualDividends: null, dividendYield: null };
  }
  const yieldValue =
    totalPurchaseAmount > 0 ? (total / totalPurchaseAmount) * 100 : null;
  return { totalAnnualDividends: total, dividendYield: yieldValue };
}

describe('shared assetBalance valuation fixtures', () => {
  const fixture = readFixture<ValuationFixture>('valuation.json');

  it('matches single-holding valuation', () => {
    expect(
      fixture.valuation_cases.map(
        ({ name, market_value, purchase_amount }) => ({
          name,
          ...calculateValuation(market_value, purchase_amount),
        }),
      ),
    ).toEqual(
      fixture.valuation_cases.map(({ name, expected }) => ({
        name,
        ...expected,
      })),
    );
  });

  it('matches valuation summaries', () => {
    expect(
      fixture.summary_cases.map(({ name, items }) => ({
        name,
        ...summarizeValuation(items),
      })),
    ).toEqual(
      fixture.summary_cases.map(({ name, expected }) => ({
        name,
        ...expected,
      })),
    );
  });

  it('matches composition percentages', () => {
    expect(
      fixture.composition_cases.map(({ name, values }) => {
        const total = values.reduce((sum, value) => sum + value, 0);
        return {
          name,
          percentages: values.map((value) =>
            calculatePercentage(value, total),
          ),
        };
      }),
    ).toEqual(
      fixture.composition_cases.map(({ name, expected_percentages }) => ({
        name,
        percentages: expected_percentages,
      })),
    );
  });

  it('matches portfolio KPI totals', () => {
    expect(
      fixture.kpi_cases.map(({ name, holdings, dividends_per_share }) => {
        const totalPurchaseAmount = holdings.reduce(
          (sum, item) => safeAdd(sum, item.total_purchase_amount || 0),
          0,
        );
        const { totalAnnualDividends, dividendYield } =
          calculatePortfolioDividends(
            holdings,
            dividends_per_share,
            totalPurchaseAmount,
          );
        return {
          name,
          total_purchase_amount: totalPurchaseAmount,
          total_annual_dividends: totalAnnualDividends,
          dividend_yield: dividendYield,
          holdings_count: holdings.length,
        };
      }),
    ).toEqual(
      fixture.kpi_cases.map(({ name, expected }) => ({
        name,
        ...expected,
      })),
    );
  });
});
