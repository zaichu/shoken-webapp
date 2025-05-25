/**
 * 保有株の情報を表すインターフェース
 */
export interface HoldingsData {
  /** 銘柄コード */
  security_code: string;
  /** 銘柄名 */
  security_name: string;
  /** 保有数量 */
  shares: number;
  /** 執行中数量 */
  executing_shares: number;
  /** 平均取得価額 */
  average_purchase_price: number;
  /** 取得総額 */
  total_purchase_amount: number;
  /** 現在値 */
  current_price: number;
  /** 前日比 */
  daily_change: number;
  /** 時価評価額 */
  market_value: number;
  /** 評価損益率（%） */
  profit_loss_rate: number;
  // インデックスシグネチャを追加して汎用的なアクセスを許可
  [key: string]: unknown;
}
