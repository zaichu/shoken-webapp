// J-Quants API レスポンスの型定義

export interface JQuantsTokenResponse {
  refresh_token: string;
}

/**
 * J-Quants API V2 決算サマリーデータ
 * V2では省略形フィールド名を使用
 */
export interface JQuantsStatementData {
  // 基本情報
  DiscDate: string; // 開示日
  DiscTime?: string; // 開示時刻
  Code: string; // 銘柄コード
  DiscNum?: string; // 開示番号
  DocType: string; // 書類種別
  CurPeriod: string; // 当期種別
  CurStartDate: string; // 当期開始日
  CurEndDate: string; // 当期終了日
  CurFYStartDate: string; // 当会計年度開始日
  CurFYEndDate: string; // 当会計年度終了日
  NxFYStartDate?: string; // 翌会計年度開始日
  NxFYEndDate?: string; // 翌会計年度終了日

  // 業績（連結）
  Sales?: string; // 売上高
  OpProfit?: string; // 営業利益
  OrdProfit?: string; // 経常利益
  Profit?: string; // 当期純利益
  EPS?: string; // 1株当たり当期純利益
  DilutedEPS?: string; // 希薄化後EPS
  TotalAssets?: string; // 総資産
  Equity?: string; // 純資産
  EquityRatio?: string; // 自己資本比率
  BPS?: string; // 1株当たり純資産

  // キャッシュフロー
  CashFlowOp?: string; // 営業CF
  CashFlowInv?: string; // 投資CF
  CashFlowFin?: string; // 財務CF
  CashEq?: string; // 現金及び現金同等物

  // 配当実績
  Div1Q?: string; // 1Q配当実績
  Div2Q?: string; // 2Q配当実績
  Div3Q?: string; // 3Q配当実績
  DivYrEnd?: string; // 期末配当実績
  DivAnn?: string; // 年間配当実績
  DistREIT?: string; // REIT分配金
  TotalDivPaidAnn?: string; // 配当支払総額実績
  PayoutRatioAnn?: string; // 配当性向実績

  // 配当予想（今期）
  FDiv1Q?: string; // 1Q配当予想
  FDiv2Q?: string; // 2Q配当予想
  FDiv3Q?: string; // 3Q配当予想
  FDivYrEnd?: string; // 期末配当予想
  FDivAnn?: string; // 年間配当予想（今期）
  FDistREIT?: string; // REIT分配金予想
  FTotalDivPaidAnn?: string; // 配当支払総額予想
  FPayoutRatioAnn?: string; // 配当性向予想

  // 配当予想（来期）
  NxFDiv1Q?: string; // 1Q配当来期予想
  NxFDiv2Q?: string; // 2Q配当来期予想
  NxFDiv3Q?: string; // 3Q配当来期予想
  NxFDivYrEnd?: string; // 期末配当来期予想
  NxFDivAnn?: string; // 年間配当来期予想
  NxFDistREIT?: string; // REIT分配金来期予想
  NxFPayoutRatioAnn?: string; // 配当性向来期予想

  // 業績予想（2Q）
  FSales2Q?: string;
  FOpProfit2Q?: string;
  FOrdProfit2Q?: string;
  FProfit2Q?: string;
  FEPS2Q?: string;

  // 業績来期予想（2Q）
  NxFSales2Q?: string;
  NxFOpProfit2Q?: string;
  NxFOrdProfit2Q?: string;
  NxFProfit2Q?: string;
  NxFEPS2Q?: string;

  // 業績予想（通期）
  FSales?: string;
  FOpProfit?: string;
  FOrdProfit?: string;
  FProfit?: string;
  FEPS?: string;

  // 業績来期予想（通期）
  NxFSales?: string;
  NxFOpProfit?: string;
  NxFOrdProfit?: string;
  NxFProfit?: string;
  NxFEPS?: string;

  // その他
  MatChgSub?: string; // 重要な子会社の異動
  SigChgScope?: string; // 連結範囲の重要な変更
  ChgAccStd?: string; // 会計基準の改正に伴う変更
  ChgOther?: string; // 会計基準の改正以外の変更
  ChgEstimate?: string; // 会計上の見積りの変更
  Restatement?: string; // 修正再表示
  SharesOutstanding?: string; // 発行済株式数
  TreasuryStock?: string; // 自己株式数
  AvgShares?: string; // 期中平均株式数

  // 個別業績
  NonConSales?: string;
  NonConOpProfit?: string;
  NonConOrdProfit?: string;
  NonConProfit?: string;
  NonConEPS?: string;
  NonConTotalAssets?: string;
  NonConEquity?: string;
  NonConEquityRatio?: string;
  NonConBPS?: string;

  // 個別業績予想
  FNonConSales2Q?: string;
  FNonConOpProfit2Q?: string;
  FNonConOrdProfit2Q?: string;
  FNonConProfit2Q?: string;
  FNonConEPS2Q?: string;
  NxFNonConSales2Q?: string;
  NxFNonConOpProfit2Q?: string;
  NxFNonConOrdProfit2Q?: string;
  NxFNonConProfit2Q?: string;
  NxFNonConEPS2Q?: string;
  FNonConSales?: string;
  FNonConOpProfit?: string;
  FNonConOrdProfit?: string;
  FNonConProfit?: string;
  FNonConEPS?: string;
  NxFNonConSales?: string;
  NxFNonConOpProfit?: string;
  NxFNonConOrdProfit?: string;
  NxFNonConProfit?: string;
  NxFNonConEPS?: string;
}

/**
 * J-Quants API V2 決算サマリーレスポンス
 * V2では fins/summary を使用し、ルートフィールドは "data"
 */
export interface JQuantsStatementsResponse {
  data: JQuantsStatementData[];
  pagination_key?: string;
}

export interface JQuantsDividendResponse {
  dividend_per_share: number;
}

export interface JQuantsError {
  message: string;
  status: number;
}
