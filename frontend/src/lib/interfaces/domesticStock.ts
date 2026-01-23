import { ReceiptBase, TaxCalculations } from './receipt';

export interface DomesticStockData extends ReceiptBase {
    trade_date: Date;
    security_code: string;
    security_name: string;
    security_info: string; // 銘柄コード + 銘柄名の表示用HTML
    account: string;
    shares: number;
    asked_price: number;
    proceeds: number;
    purchase_price: number;
    realized_profit_and_loss: number;
    taxes: number;
    realized_profit_and_loss_after_tax: number;
    // インデックスシグネチャを追加して汎用的なアクセスを許可
    [key: string]: unknown;
}

export interface DomesticStockCalculations extends TaxCalculations {
    total_realized_profit_and_loss: number;
    total_realized_profit_and_loss_after_tax: number;
}

export interface DomesticStockSummary extends DomesticStockCalculations {
    filter: string;
    // インデックスシグネチャを追加して汎用的なアクセスを許可
    [key: string]: unknown;
}
