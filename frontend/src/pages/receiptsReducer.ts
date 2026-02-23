export type ReceiptsType = 'dividend' | 'domesticstock' | 'mutualfund';

export interface ReceiptsState {
  receiptsType: ReceiptsType;
  csvData: {
    dividend: Record<string, unknown>[];
    domesticstock: Record<string, unknown>[];
    mutualfund: Record<string, unknown>[];
  };
  showDeleteConfirm: boolean;
}

export type ReceiptsAction =
  | { type: 'SET_RECEIPTS_TYPE'; payload: ReceiptsType }
  | { type: 'SET_CSV_DATA'; receiptsType: ReceiptsType; payload: Record<string, unknown>[] }
  | { type: 'SET_SHOW_DELETE_CONFIRM'; payload: boolean }
  | { type: 'LOGOUT' };

export const initialState: ReceiptsState = {
  receiptsType: 'dividend',
  csvData: { dividend: [], domesticstock: [], mutualfund: [] },
  showDeleteConfirm: false,
};

export function receiptsReducer(state: ReceiptsState, action: ReceiptsAction): ReceiptsState {
  switch (action.type) {
    case 'SET_RECEIPTS_TYPE':
      return { ...state, receiptsType: action.payload };
    case 'SET_CSV_DATA':
      return { ...state, csvData: { ...state.csvData, [action.receiptsType]: action.payload } };
    case 'SET_SHOW_DELETE_CONFIRM':
      return { ...state, showDeleteConfirm: action.payload };
    case 'LOGOUT':
      return { ...state, csvData: { dividend: [], domesticstock: [], mutualfund: [] } };
  }
}
