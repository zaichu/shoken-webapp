import type { components } from '@/generated/api';
import type { FormatFunction, TableColumnAlignment } from '@/types/common';

export type DividendApiData = components['schemas']['Dividend'];
export type DomesticStockApiData = components['schemas']['DomesticStock'];
export type MutualfundApiData = components['schemas']['Mutualfund'];

export interface ReceiptBase {
  settlement_date: Date;
}

export interface TaxCalculations {
  total_taxes: number;
}

export interface TableColumnConfig {
  key: string;
  header: string;
  width?: string;
  textAlign?: TableColumnAlignment;
  format?: FormatFunction;
}

export interface SummaryColumnConfig {
  key: string;
  colSpan?: number;
  textAlign?: TableColumnAlignment;
  format?: FormatFunction;
}

export interface DividendData
  extends Omit<DividendApiData, 'settlement_date' | 'id' | 'created_at' | 'updated_at'>,
    ReceiptBase {
  [key: string]: unknown;
}

export interface DividendCalculations extends TaxCalculations {
  total_dividends_before_tax: number;
  total_net_amount_received: number;
}

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
