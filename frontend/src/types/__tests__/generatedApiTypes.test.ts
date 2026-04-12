import { describe, expectTypeOf, it } from 'vitest';

import type { components } from '@/generated/api';
import type { AssetBalanceApiData, AssetBalanceData } from '@/types/api';

describe('generated api type bindings', () => {
  it('asset balance API and view data types are derived from generated schemas', () => {
    expectTypeOf<AssetBalanceApiData>().toEqualTypeOf<components['schemas']['AssetBalance']>();
    expectTypeOf<AssetBalanceData>().toEqualTypeOf<components['schemas']['AssetBalance']>();
  });
});
