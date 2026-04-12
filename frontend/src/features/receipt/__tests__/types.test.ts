import { describe, expectTypeOf, it } from 'vitest';

import type { components } from '@/generated/api';
import type {
  DividendApiData,
  DividendData,
  DomesticStockApiData,
  DomesticStockData,
  MutualfundApiData,
  MutualfundData,
} from '@/features/receipt/types';

describe('receipt view model types', () => {
  it('API data types are derived from generated schemas', () => {
    expectTypeOf<DividendApiData>().toEqualTypeOf<components['schemas']['Dividend']>();
    expectTypeOf<DomesticStockApiData>().toEqualTypeOf<components['schemas']['DomesticStock']>();
    expectTypeOf<MutualfundApiData>().toEqualTypeOf<components['schemas']['Mutualfund']>();
  });

  it('view data keeps Date based fields for UI logic', () => {
    expectTypeOf<DividendData['settlement_date']>().toEqualTypeOf<Date>();
    expectTypeOf<DomesticStockData['trade_date']>().toEqualTypeOf<Date>();
    expectTypeOf<DomesticStockData['settlement_date']>().toEqualTypeOf<Date>();
    expectTypeOf<MutualfundData['trade_date']>().toEqualTypeOf<Date>();
    expectTypeOf<MutualfundData['settlement_date']>().toEqualTypeOf<Date>();
  });

  it('mutualfund view data keeps dividends as a normalized string for the UI', () => {
    expectTypeOf<MutualfundData['dividends']>().toEqualTypeOf<string>();
  });
});
