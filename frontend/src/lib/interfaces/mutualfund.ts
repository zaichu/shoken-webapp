import { ReceiptBase } from './receipt';

/**
 * 投資信託データの基本構造
 */
export interface MutualfundData extends ReceiptBase {
    trade_date: Date;                       // 約定日
    settlement_date: Date;                  // 受渡日
    fund_name: string;                      // ファンド名
    dividends: string;                      // 分配金
    account: string;                        // 口座
    shares: number;                         // 数量[口]
    exchange_rate: number;                  // 為替レート[円]
    cancellation_unit_price_yen: number;    // 解約単価[円]
    cancellation_amount_yen: number;        // 解約額[円]
    average_acquisition_price_yen: number;  // 平均取得価額[円]
    realized_profit_and_loss: number;       // 実現損益[円]
    taxes: number;                          // 税額
    realized_profit_and_loss_after_tax: number; // 実現損益(税引)[円]
    // インデックスシグネチャを追加して汎用的なアクセスを許可
    [key: string]: unknown;
}

/**
 * 投資信託の集計計算結果
 */
export interface MutualfundCalculations {
    total_realized_profit_and_loss: number;           // 合計実現損益
    total_taxes: number;                              // 合計税額
    total_realized_profit_and_loss_after_tax: number; // 合計実現損益(税引)
}

/**
 * 投資信託の集計サマリー
 */
export interface MutualfundSummary extends MutualfundCalculations {
    filter: string;                            // 集計単位（日付や検索クエリ）
    // インデックスシグネチャを追加して汎用的なアクセスを許可
    [key: string]: unknown;
}
