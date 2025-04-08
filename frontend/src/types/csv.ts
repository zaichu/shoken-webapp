// src/types/csv.ts

export interface CSVParseCallbacks {
  onStart?: () => void;
  onSuccess?: (data: any[]) => void;
  onError?: (error: string) => void;
  onComplete?: () => void;
}

export interface CSVParseOptions {
  header?: boolean;
  skipEmptyLines?: boolean;
  transformHeader?: (header: string) => string;
  dynamicTyping?: boolean;
  encoding?: string;
}
