export interface CsvImportError {
  row: number;
  message: string;
}

export interface CsvUploadResult {
  inserted: number;
  skipped: number;
  errors: CsvImportError[];
}
