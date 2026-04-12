import type { components } from '@/generated/api';

import type { ReceiptBase, TaxCalculations } from './receipt';

export type DomesticStockApiData = components['schemas']['DomesticStock'];

export interface DomesticStockData
  extends Omit<
      DomesticStockApiData,
      'trade_date' | 'settlement_date' | 'id' | 'created_at' | 'updated_at'
    >,
    ReceiptBase {
  trade_date: Date;
  [key: string]: unknown;
}

export interface DomesticStockCalculations extends TaxCalculations {
  total_realized_profit_and_loss: number;
  total_realized_profit_and_loss_after_tax: number;
}

export interface DomesticStockSummary extends DomesticStockCalculations {
  filter: string;
  [key: string]: unknown;
}
