import type { AssetBalanceData } from '@/types/api';

function fmt(value: number): string {
  return value.toLocaleString('ja-JP');
}

function fmtRate(value: number): string {
  return `${value.toFixed(2)}%`;
}

export function generateAssetReviewPrompt(
  assets: AssetBalanceData[],
  dividendPerShareMap: Map<string, number>
): string {
  const totalPurchase = assets.reduce((sum, a) => sum + a.total_purchase_amount, 0);
  const totalMarketValue = assets.reduce((sum, a) => sum + a.market_value, 0);
  const totalProfitLoss = totalMarketValue - totalPurchase;
  const totalProfitLossRate =
    totalPurchase > 0 ? (totalProfitLoss / totalPurchase) * 100 : 0;

  const summaryLines = [
    `- 保有銘柄数: ${assets.length}銘柄`,
    `- 合計取得金額: ¥${fmt(totalPurchase)}`,
    `- 合計評価額: ¥${fmt(totalMarketValue)}`,
    `- 評価損益額: ¥${fmt(totalProfitLoss)}`,
    `- 評価損益率: ${fmtRate(totalProfitLossRate)}`,
  ].join('\n');

  const header = [
    '銘柄コード',
    '銘柄名',
    '保有株数',
    '平均取得単価',
    '現在値',
    '取得額',
    '評価額',
    '評価損益額',
    '損益率',
    '推定1株配当',
  ].join(' | ');

  const rows = assets.map((a) => {
    const profitLossAmount = a.market_value - a.total_purchase_amount;
    const div = dividendPerShareMap.has(a.security_code)
      ? `¥${dividendPerShareMap.get(a.security_code)}`
      : '不明';
    return [
      a.security_code,
      a.security_name,
      fmt(a.shares),
      `¥${fmt(a.average_purchase_price)}`,
      `¥${fmt(a.current_price)}`,
      `¥${fmt(a.total_purchase_amount)}`,
      `¥${fmt(a.market_value)}`,
      `¥${fmt(profitLossAmount)}`,
      fmtRate(a.profit_loss_rate),
      div,
    ].join(' | ');
  });

  return `あなたは日本株ポートフォリオをレビューする投資分析アシスタントです。
以下の保有銘柄データをもとに、ポートフォリオ全体の総評をしてください。

【重要な注意事項】
このプロンプトで求めるのは投資助言・売買推奨ではありません。
判断材料の整理・リスク観点・確認事項として回答してください。
確定的な予測や断定的な表現は避け、検討観点と質問リストを提示してください。

【お願いしたい分析項目】
1. ポートフォリオ全体の総評
2. 集中リスク・業種偏り・銘柄偏りの読み取り
3. 損益状況の読み取り（含み益/損の分布）
4. 配当観点のコメント（推定配当が不明な銘柄は不明として扱う）
5. 追加で確認すべきIR・決算・ニュース・指標
6. 売買指示ではなく、検討すべき観点と質問リスト

【保有銘柄データ】
集計:
${summaryLines}

各銘柄:
${header}
${rows.join('\n')}

不足している情報があれば、分析に必要な情報として質問してください。`;
}
