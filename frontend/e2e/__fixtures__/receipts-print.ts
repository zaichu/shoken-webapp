// 印刷検査用のモック。12桁の長い金額を複数列に並べ、印字幅に収まることを確かめる。
export function domesticStocksFixture(userId: string) {
  const created_at = '2024-03-01T00:00:00Z';
  const updated_at = '2024-03-01T00:00:00Z';
  return [
    {
      id: 'domestic-print',
      user_id: userId,
      trade_date: '2024-03-01',
      settlement_date: '2024-03-04',
      security_code: '8306',
      security_name: '三菱UFJフィナンシャル・グループ',
      account: '特定口座',
      shares: 123456,
      asked_price: 12345,
      proceeds: 123456789,
      purchase_price: 22345,
      realized_profit_and_loss: -123456789,
      taxes: 0,
      realized_profit_and_loss_after_tax: -123456789,
      created_at,
      updated_at,
    },
    {
      id: 'domestic-print-long',
      user_id: userId,
      trade_date: '2024-03-01',
      settlement_date: '2024-03-04',
      security_code: '9984',
      security_name: 'ソフトバンクグループ',
      account: 'NISA口座',
      shares: 9876543,
      asked_price: 1234567,
      proceeds: 123456789012,
      purchase_price: 2345678,
      realized_profit_and_loss: -876543210987,
      taxes: 12345678901,
      realized_profit_and_loss_after_tax: -765432109876,
      created_at,
      updated_at,
    },
  ];
}

// 印刷で3行を超える長さのファンド名。印字時にクランプされないことを確かめる
export const LONG_FUND_NAME =
  'eMAXIS Slim 全世界株式（オール・カントリー）・楽天・バンガード・ファンド（全世界株式）長期積立専用アクティブ運用特別受益権（為替ヘッジなし）＜子ファンド組込型＞ニッセイ外国株式インデックスファンド＜購入・換金手数料なし＞シリーズ';

export function mutualFundsFixture(userId: string) {
  const created_at = '2024-03-01T00:00:00Z';
  const updated_at = '2024-03-01T00:00:00Z';
  return [
    {
      id: 'mutual-fund-print',
      user_id: userId,
      trade_date: '2024-03-01',
      settlement_date: '2024-03-04',
      fund_name: 'eMAXIS Slim 全世界株式（オール・カントリー）',
      account: '特定口座',
      shares: '123456',
      exchange_rate: '1',
      cancellation_unit_price_yen: '12345',
      cancellation_amount_yen: '123456789',
      average_acquisition_price_yen: '22345',
      dividends: '0',
      realized_profit_and_loss: '-123456789',
      taxes: '0',
      realized_profit_and_loss_after_tax: '-123456789',
      created_at,
      updated_at,
    },
    {
      id: 'mutual-fund-print-long',
      user_id: userId,
      trade_date: '2024-03-01',
      settlement_date: '2024-03-04',
      fund_name: 'eMAXIS Slim 米国株式（S&P500）',
      account: 'NISA口座',
      shares: '9876543',
      exchange_rate: '1',
      cancellation_unit_price_yen: '1234567',
      cancellation_amount_yen: '234567890123',
      average_acquisition_price_yen: '2345678',
      dividends: '0',
      realized_profit_and_loss: '-345678901234',
      taxes: '1234567890',
      realized_profit_and_loss_after_tax: '-234567890123',
      created_at,
      updated_at,
    },
  ];
}
