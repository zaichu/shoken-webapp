// J-Quants API レスポンスの型定義

export interface JQuantsTokenResponse {
  refresh_token: string;
}

export interface JQuantsStatementData {
  DisclosedDate: string;
  DisclosedTime: string;
  LocalCode: string;
  DisclosureNumber: string;
  TypeOfDocument: string;
  TypeOfCurrentPeriod: string;
  CurrentPeriodStartDate: string;
  CurrentPeriodEndDate: string;
  CurrentFiscalYearStartDate: string;
  CurrentFiscalYearEndDate: string;
  NextFiscalYearStartDate: string;
  NextFiscalYearEndDate: string;
  NetSales: string;
  OperatingProfit: string;
  OrdinaryProfit: string;
  Profit: string;
  EarningsPerShare: string;
  DilutedEarningsPerShare: string;
  TotalAssets: string;
  Equity: string;
  EquityToAssetRatio: string;
  BookValuePerShare: string;
  CashFlowsFromOperatingActivities: string;
  CashFlowsFromInvestingActivities: string;
  CashFlowsFromFinancingActivities: string;
  CashAndEquivalents: string;
  ResultDividendPerShare1stQuarter: string;
  ResultDividendPerShare2ndQuarter: string;
  ResultDividendPerShare3rdQuarter: string;
  ResultDividendPerShareFiscalYearEnd: string;
  ResultDividendPerShareAnnual: string;
  DistributionsPerUnit_REIT: string;
  ResultTotalDividendPaidAnnual: string;
  ResultPayoutRatioAnnual: string;
  ForecastDividendPerShare1stQuarter: string;
  ForecastDividendPerShare2ndQuarter: string;
  ForecastDividendPerShare3rdQuarter: string;
  ForecastDividendPerShareFiscalYearEnd: string;
  ForecastDividendPerShareAnnual: string;
  ForecastDistributionsPerUnit_REIT: string;
  ForecastTotalDividendPaidAnnual: string;
  ForecastPayoutRatioAnnual: string;
  NextYearForecastDividendPerShare1stQuarter: string;
  NextYearForecastDividendPerShare2ndQuarter: string;
  NextYearForecastDividendPerShare3rdQuarter: string;
  NextYearForecastDividendPerShareFiscalYearEnd: string;
  NextYearForecastDividendPerShareAnnual: string;
  NextYearForecastDistributionsPerUnit_REIT: string;
  NextYearForecastPayoutRatioAnnual: string;
  ForecastNetSales2ndQuarter: string;
  ForecastOperatingProfit2ndQuarter: string;
  ForecastOrdinaryProfit2ndQuarter: string;
  ForecastProfit2ndQuarter: string;
  ForecastEarningsPerShare2ndQuarter: string;
  NextYearForecastNetSales2ndQuarter: string;
  NextYearForecastOperatingProfit2ndQuarter: string;
  NextYearForecastOrdinaryProfit2ndQuarter: string;
  NextYearForecastProfit2ndQuarter: string;
  NextYearForecastEarningsPerShare2ndQuarter: string;
  ForecastNetSales: string;
  ForecastOperatingProfit: string;
  ForecastOrdinaryProfit: string;
  ForecastProfit: string;
  ForecastEarningsPerShare: string;
  NextYearForecastNetSales: string;
  NextYearForecastOperatingProfit: string;
  NextYearForecastOrdinaryProfit: string;
  NextYearForecastProfit: string;
  NextYearForecastEarningsPerShare: string;
  MaterialChangesInSubsidiaries: string;
  SignificantChangesInTheScopeOfConsolidation: string;
  ChangesBasedOnRevisionsOfAccountingStandard: string;
  ChangesOtherThanOnesBasedOnRevisionsOfAccountingStandard: string;
  ChangesInAccountingEstimates: string;
  RetrospectiveRestatement: string;
  NumberOfIssuedAndOutstandingSharesAtTheEndOfFiscalYearIncludingTreasuryStock: string;
  NumberOfTreasuryStockAtTheEndOfFiscalYear: string;
  AverageNumberOfShares: string;
  NonConsolidatedNetSales: string;
  NonConsolidatedOperatingProfit: string;
  NonConsolidatedOrdinaryProfit: string;
  NonConsolidatedProfit: string;
  NonConsolidatedEarningsPerShare: string;
  NonConsolidatedTotalAssets: string;
  NonConsolidatedEquity: string;
  NonConsolidatedEquityToAssetRatio: string;
  NonConsolidatedBookValuePerShare: string;
  ForecastNonConsolidatedNetSales2ndQuarter: string;
  ForecastNonConsolidatedOperatingProfit2ndQuarter: string;
  ForecastNonConsolidatedOrdinaryProfit2ndQuarter: string;
  ForecastNonConsolidatedProfit2ndQuarter: string;
  ForecastNonConsolidatedEarningsPerShare2ndQuarter: string;
  NextYearForecastNonConsolidatedNetSales2ndQuarter: string;
  NextYearForecastNonConsolidatedOperatingProfit2ndQuarter: string;
  NextYearForecastNonConsolidatedOrdinaryProfit2ndQuarter: string;
  NextYearForecastNonConsolidatedProfit2ndQuarter: string;
  NextYearForecastNonConsolidatedEarningsPerShare2ndQuarter: string;
  ForecastNonConsolidatedNetSales: string;
  ForecastNonConsolidatedOperatingProfit: string;
  ForecastNonConsolidatedOrdinaryProfit: string;
  ForecastNonConsolidatedProfit: string;
  ForecastNonConsolidatedEarningsPerShare: string;
  NextYearForecastNonConsolidatedNetSales: string;
  NextYearForecastNonConsolidatedOperatingProfit: string;
  NextYearForecastNonConsolidatedOrdinaryProfit: string;
  NextYearForecastNonConsolidatedProfit: string;
  NextYearForecastNonConsolidatedEarningsPerShare: string;
}

/**
 * J-Quants API V2 決算サマリーレスポンス
 * V2では fins/statements → fins/summary に変更され、
 * レスポンスフィールド名も statements → fin_summary に変更
 */
export interface JQuantsStatementsResponse {
  fin_summary: JQuantsStatementData[];
  pagination_key?: string;
}

export interface JQuantsDividendResponse {
  dividend_per_share: number;
}

export interface JQuantsError {
  message: string;
  status: number;
}
