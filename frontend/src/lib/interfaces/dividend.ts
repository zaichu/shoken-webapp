import { ReceiptBase } from './receipt';

export interface DividendData extends ReceiptBase {
    product: string;
    account: string;
    security_code: string;
    security_name: string;
    unit_price: number;
    shares: number;
    dividends_before_tax: number;
    taxes: number;
    net_amount_received: number;
}

export interface DividendCalculations {
    total_dividends_before_tax: number;
    total_taxes: number;
    total_net_amount_received: number;
}

export interface DividendSummary {
    filter: string;
    dividends_before_tax: number;
    taxes: number;
    net_amount_received: number;
}
