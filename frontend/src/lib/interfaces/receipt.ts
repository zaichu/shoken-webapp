import { FormatFunction, TableColumnAlignment, SelectOption } from '../../types/common';

export interface ReceiptBase {
    settlement_date: Date;
}

export interface Calculations {
    [key: string]: number;
}

export interface ReceiptSummary {
    filter: string;
    [key: string]: string | number;
}

export type SearchOption = SelectOption;

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
