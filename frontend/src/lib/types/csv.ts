// CSV解析オプション
export interface CSVParseOptions {
  skipHeaderRows?: number;
  encoding?: string;
  delimiter?: string;
  maxRows?: number;
}

// CSV解析コールバック
export interface CSVParseCallbacks {
  onStart?: () => void;
  onSuccess?: (result: CSVParseResult) => void;
  onError?: (error: string) => void;
  onComplete?: () => void;
  onProgress?: (progress: number) => void;
}

// CSV解析エラー
export interface CSVParseError {
  type: string;
  code?: string;
  message: string;
  row?: number;
  column?: number;
}

// CSV解析メタデータ
export interface CSVParseMeta {
  encoding: string;
  encodingConfidence: number;
  delimiter: string;
  linebreak: string;
  aborted: boolean;
  truncated: boolean;
  fields?: string[];
}

// CSV解析結果
export interface CSVParseResult {
  data: Record<string, unknown>[];
  errors: CSVParseError[];
  meta: CSVParseMeta;
}

// CSVエクスポートオプション
export interface CSVExportOptions {
  delimiter?: string;
  headers?: boolean;
  encoding?: 'utf-8' | 'shift-jis';
  linebreak?: '\n' | '\r\n';
  quotes?: boolean;
}

// CSV列定義
export interface CSVColumnDefinition {
  field: string;
  header: string;
  formatter?: (value: unknown) => string;
  required?: boolean;
  validator?: (value: unknown) => boolean;
}

// CSV検証ルール
export interface CSVValidationRule {
  field: string;
  validator: (value: unknown, row: Record<string, unknown>) => boolean | string;
  message?: string;
}

// CSV変換マッピング
export interface CSVTransformMapping {
  source: string;
  target: string;
  transform?: (value: unknown) => unknown;
}

// CSVインポート設定
export interface CSVImportConfig {
  columns: CSVColumnDefinition[];
  validationRules?: CSVValidationRule[];
  transformMappings?: CSVTransformMapping[];
  skipEmptyRows?: boolean;
  trimValues?: boolean;
  maxRows?: number;
}

// CSVエクスポート設定
export interface CSVExportConfig {
  columns: CSVColumnDefinition[];
  options?: CSVExportOptions;
  filename?: string;
}
