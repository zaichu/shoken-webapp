import { ReceiptBase, TaxCalculations } from './receipt';

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
    // インデックスシグネチャを追加して汎用的なアクセスを許可
    [key: string]: unknown;
}

export interface DividendCalculations extends TaxCalculations {
    total_dividends_before_tax: number;
    total_net_amount_received: number;
}
