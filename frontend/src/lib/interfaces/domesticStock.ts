import { ReceiptBase } from './receipt';

export interface DomesticStockData extends ReceiptBase {
    trade_date: Date;
    security_code: string;
    security_name: string;
    account: string;
    shares: number;
    asked_price: number;
    proceeds: number;
    purchase_price: number;
    realized_profit_and_loss: number;
}

export interface DomesticStockCalculations {
    total_realized_profit_and_loss: number;
    total_taxes: number;
    total_realized_profit_and_loss_after_tax: number;
}

export interface DomesticStockSummary extends DomesticStockCalculations {
    filter: string;
}
