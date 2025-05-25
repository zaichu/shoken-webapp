export interface CSVParseCallbacks {
  onStart?: () => void;
  onSuccess?: (data: Record<string, unknown>[]) => void;
  onError?: (error: string) => void;
  onComplete?: () => void;
}

export interface CSVParseOptions {
  header?: boolean;
  skipEmptyLines?: boolean;
  transformHeader?: (header: string) => string;
  dynamicTyping?: boolean;
  encoding?: string;
  skipHeaderRows?: number;  // ヘッダー行の前にスキップする行数
  skipFooterRows?: number;  // フッター行（合計行など）をスキップする行数
  detectHeader?: boolean;   // ヘッダー行を自動検出するかどうか
  headerPatterns?: string[][]; // ヘッダー検出パターンをカスタマイズ
  skipPatterns?: string[];  // スキップする行のパターン（例：['合計', 'TOTAL']）
}
