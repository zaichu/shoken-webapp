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

