import { FormatFunction, TableColumnAlignment } from '../../types/common';

export interface ReceiptBase {
    settlement_date: Date;
}

/**
 * 税金計算を含む共通インターフェース
 */
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
