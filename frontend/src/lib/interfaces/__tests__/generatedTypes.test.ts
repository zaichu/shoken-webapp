import { describe, expectTypeOf, it } from 'vitest';

import type { components } from '@/generated/api';
import type {
  AssetBalanceApiData,
  AssetBalanceData,
} from '@/lib/interfaces/assetBalance';
import type { DividendApiData, DividendData } from '@/lib/interfaces/dividend';
import type {
  DomesticStockApiData,
  DomesticStockData,
} from '@/lib/interfaces/domesticStock';
import type { MutualfundApiData, MutualfundData } from '@/lib/interfaces/mutualfund';

describe('generated interface bindings', () => {
  it('asset balance API and view data types are derived from generated schemas', () => {
    expectTypeOf<AssetBalanceApiData>().toEqualTypeOf<components['schemas']['AssetBalance']>();
    expectTypeOf<AssetBalanceData>().toEqualTypeOf<components['schemas']['AssetBalance']>();
  });

  it('receipt API data types are identical to generated schemas', () => {
    expectTypeOf<DividendApiData>().toEqualTypeOf<components['schemas']['Dividend']>();
    expectTypeOf<DomesticStockApiData>().toEqualTypeOf<components['schemas']['DomesticStock']>();
    expectTypeOf<MutualfundApiData>().toEqualTypeOf<components['schemas']['Mutualfund']>();
  });

  it('receipt view data keeps Date based fields for UI logic', () => {
    expectTypeOf<DividendData['settlement_date']>().toEqualTypeOf<Date>();
    expectTypeOf<DomesticStockData['trade_date']>().toEqualTypeOf<Date>();
    expectTypeOf<DomesticStockData['settlement_date']>().toEqualTypeOf<Date>();
    expectTypeOf<MutualfundData['trade_date']>().toEqualTypeOf<Date>();
    expectTypeOf<MutualfundData['settlement_date']>().toEqualTypeOf<Date>();
  });

  it('MutualfundData keeps dividends as a normalized string for the UI', () => {
    expectTypeOf<MutualfundData['dividends']>().toEqualTypeOf<string>();
  });
});
