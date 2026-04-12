import type { components } from '@/generated/api';

import type { ReceiptBase, TaxCalculations } from './receipt';

export type MutualfundApiData = components['schemas']['Mutualfund'];

export interface MutualfundData
  extends Omit<
      MutualfundApiData,
      'trade_date' | 'settlement_date' | 'dividends' | 'id' | 'created_at' | 'updated_at'
    >,
    ReceiptBase {
  trade_date: Date;
  dividends: string;
  [key: string]: unknown;
}

export interface MutualfundCalculations extends TaxCalculations {
  total_realized_profit_and_loss: number;
  total_realized_profit_and_loss_after_tax: number;
}
