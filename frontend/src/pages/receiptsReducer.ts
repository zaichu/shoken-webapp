export type ReceiptsType = 'dividend' | 'domesticstock' | 'mutualfund';

export interface ImportResult {
  inserted: number;
  skipped: number;
  errors: { row: number; message: string }[];
}

export interface CsvPreview {
  totalRows: number;
  validRows: number;
  errors: { row: number; message: string }[];
  rows: Record<string, unknown>[];
}

export interface ReceiptsState {
  receiptsType: ReceiptsType;
  rawFiles: {
    dividend: File | null;
    domesticstock: File | null;
    mutualfund: File | null;
  };
  csvPreviews: {
    dividend: CsvPreview | null;
    domesticstock: CsvPreview | null;
    mutualfund: CsvPreview | null;
  };
  lastImportResults: {
    dividend: ImportResult | null;
    domesticstock: ImportResult | null;
    mutualfund: ImportResult | null;
  };
  showDeleteConfirm: boolean;
}

export type ReceiptsAction =
  | { type: 'SET_RECEIPTS_TYPE'; payload: ReceiptsType }
  | { type: 'SET_RAW_FILE'; receiptsType: ReceiptsType; payload: File | null }
  | { type: 'SET_CSV_PREVIEW'; receiptsType: ReceiptsType; payload: CsvPreview | null }
  | { type: 'SET_IMPORT_RESULT'; receiptsType: ReceiptsType; payload: ImportResult }
  | { type: 'CLEAR_IMPORT_RESULT'; receiptsType: ReceiptsType }
  | { type: 'SET_SHOW_DELETE_CONFIRM'; payload: boolean }
  | { type: 'LOGOUT' };

export const initialState: ReceiptsState = {
  receiptsType: 'dividend',
  rawFiles: { dividend: null, domesticstock: null, mutualfund: null },
  csvPreviews: { dividend: null, domesticstock: null, mutualfund: null },
  lastImportResults: { dividend: null, domesticstock: null, mutualfund: null },
  showDeleteConfirm: false,
};

export function receiptsReducer(state: ReceiptsState, action: ReceiptsAction): ReceiptsState {
  switch (action.type) {
    case 'SET_RECEIPTS_TYPE':
      return { ...state, receiptsType: action.payload };
    case 'SET_RAW_FILE':
      return {
        ...state,
        rawFiles: { ...state.rawFiles, [action.receiptsType]: action.payload },
        // 新しいファイルを選択したら前回の結果とプレビューをクリア
        csvPreviews: { ...state.csvPreviews, [action.receiptsType]: null },
        lastImportResults: { ...state.lastImportResults, [action.receiptsType]: null },
      };
    case 'SET_CSV_PREVIEW':
      return {
        ...state,
        csvPreviews: { ...state.csvPreviews, [action.receiptsType]: action.payload },
      };
    case 'SET_IMPORT_RESULT':
      return {
        ...state,
        lastImportResults: { ...state.lastImportResults, [action.receiptsType]: action.payload },
      };
    case 'CLEAR_IMPORT_RESULT':
      return {
        ...state,
        lastImportResults: { ...state.lastImportResults, [action.receiptsType]: null },
      };
    case 'SET_SHOW_DELETE_CONFIRM':
      return { ...state, showDeleteConfirm: action.payload };
    case 'LOGOUT':
      return {
        ...state,
        rawFiles: { dividend: null, domesticstock: null, mutualfund: null },
        csvPreviews: { dividend: null, domesticstock: null, mutualfund: null },
        lastImportResults: { dividend: null, domesticstock: null, mutualfund: null },
      };
  }
}
