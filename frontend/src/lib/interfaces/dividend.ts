import type { components } from '@/generated/api';

import type { ReceiptBase, TaxCalculations } from './receipt';

export type DividendApiData = components['schemas']['Dividend'];

export interface DividendData
  extends Omit<DividendApiData, 'settlement_date' | 'id' | 'created_at' | 'updated_at'>,
    ReceiptBase {
  [key: string]: unknown;
}

export interface DividendCalculations extends TaxCalculations {
  total_dividends_before_tax: number;
  total_net_amount_received: number;
}
