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
  DiscNo?: string; // 開示番号
  DocType: string; // 書類種別
  CurPerType?: string; // 当期種別
  CurPerSt?: string; // 当期開始日
  CurPerEn?: string; // 当期終了日
  CurFYSt?: string; // 当会計年度開始日
  CurFYEn?: string; // 当会計年度終了日
  NxtFYSt?: string; // 翌会計年度開始日
  NxtFYEn?: string; // 翌会計年度終了日

  // 業績（連結）
  Sales?: string; // 売上高
  OP?: string; // 営業利益
  OdP?: string; // 経常利益
  NP?: string; // 当期純利益
  EPS?: string; // 1株当たり当期純利益
  DEPS?: string; // 希薄化後EPS
  TA?: string; // 総資産
  Eq?: string; // 純資産
  EqAR?: string; // 自己資本比率
  BPS?: string; // 1株当たり純資産

  // キャッシュフロー
  CFO?: string; // 営業CF
  CFI?: string; // 投資CF
  CFF?: string; // 財務CF
  CashEq?: string; // 現金及び現金同等物

  // 配当実績
  Div1Q?: string; // 1Q配当実績
  Div2Q?: string; // 2Q配当実績
  Div3Q?: string; // 3Q配当実績
  DivFY?: string; // 期末配当実績
  DivAnn?: string; // 年間配当実績
  DivUnit?: string; // REIT分配金
  DivTotalAnn?: string; // 配当支払総額実績
  PayoutRatioAnn?: string; // 配当性向実績

  // 配当予想（今期）
  FDiv1Q?: string; // 1Q配当予想
  FDiv2Q?: string; // 2Q配当予想
  FDiv3Q?: string; // 3Q配当予想
  FDivFY?: string; // 期末配当予想
  FDivAnn?: string; // 年間配当予想（今期）
  FDivUnit?: string; // REIT分配金予想
  FDivTotalAnn?: string; // 配当支払総額予想
  FPayoutRatioAnn?: string; // 配当性向予想

  // 配当予想（来期）
  NxFDiv1Q?: string; // 1Q配当来期予想
  NxFDiv2Q?: string; // 2Q配当来期予想
  NxFDiv3Q?: string; // 3Q配当来期予想
  NxFDivFY?: string; // 期末配当来期予想
  NxFDivAnn?: string; // 年間配当来期予想
  NxFDivUnit?: string; // REIT分配金来期予想
  NxFPayoutRatioAnn?: string; // 配当性向来期予想

  // 業績予想（2Q）
  FSales2Q?: string;
  FOP2Q?: string;
  FOdP2Q?: string;
  FNP2Q?: string;
  FEPS2Q?: string;

  // 業績来期予想（2Q）
  NxFSales2Q?: string;
  NxFOP2Q?: string;
  NxFOdP2Q?: string;
  NxFNp2Q?: string;
  NxFEPS2Q?: string;

  // 業績予想（通期）
  FSales?: string;
  FOP?: string;
  FOdP?: string;
  FNP?: string;
  FEPS?: string;

  // 業績来期予想（通期）
  NxFSales?: string;
  NxFOP?: string;
  NxFOdP?: string;
  NxFNp?: string;
  NxFEPS?: string;

  // その他
  MatChgSub?: string; // 重要な子会社の異動
  SigChgInC?: string; // 連結範囲の重要な変更
  ChgByASRev?: string; // 会計基準の改正に伴う変更
  ChgNoASRev?: string; // 会計基準の改正以外の変更
  ChgAcEst?: string; // 会計上の見積りの変更
  RetroRst?: string; // 修正再表示
  ShOutFY?: string; // 発行済株式数
  TrShFY?: string; // 自己株式数
  AvgSh?: string; // 期中平均株式数

  // 個別業績
  NCSales?: string;
  NCOP?: string;
  NCOdP?: string;
  NCNP?: string;
  NCEPS?: string;
  NCTA?: string;
  NCEq?: string;
  NCEqAR?: string;
  NCBPS?: string;

  // 個別業績予想
  FNCSales2Q?: string;
  FNCOP2Q?: string;
  FNCOdP2Q?: string;
  FNCNP2Q?: string;
  FNCEPS2Q?: string;
  NxFNCSales2Q?: string;
  NxFNCOP2Q?: string;
  NxFNCOdP2Q?: string;
  NxFNCNP2Q?: string;
  NxFNCEPS2Q?: string;
  FNCSales?: string;
  FNCOP?: string;
  FNCOdP?: string;
  FNCNP?: string;
  FNCEPS?: string;
  NxFNCSales?: string;
  NxFNCOP?: string;
  NxFNCOdP?: string;
  NxFNCNP?: string;
  NxFNCEPS?: string;
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
