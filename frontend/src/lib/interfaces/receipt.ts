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

export interface SearchOption {
    value: string;
    label: string;
}

export interface TableColumnConfig {
    key: string;
    header: string;
    width?: string;
    textAlign?: 'left' | 'center' | 'right';
    format?: (value: any) => string;
}

export interface SummaryColumnConfig {
    key: string;
    colSpan?: number;
    textAlign?: 'left' | 'center' | 'right';
    format?: (value: any) => string;
}
