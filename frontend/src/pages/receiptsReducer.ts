import { DividendData } from '@/lib/interfaces/dividend';
import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { MutualfundData } from '@/lib/interfaces/mutualfund';

export type ReceiptsType = 'dividend' | 'domesticstock' | 'mutualfund';

export interface ReceiptsState {
  receiptsType: ReceiptsType;
  csvData: {
    dividend: Record<string, unknown>[];
    domesticstock: Record<string, unknown>[];
    mutualfund: Record<string, unknown>[];
  };
  dbData: {
    dividend: DividendData[];
    domesticstock: DomesticStockData[];
    mutualfund: MutualfundData[];
  };
  dbLoading: boolean;
  dbError: string | null;
  saving: boolean;
  deleting: boolean;
  showDeleteConfirm: boolean;
}

export type ReceiptsAction =
  | { type: 'SET_RECEIPTS_TYPE'; payload: ReceiptsType }
  | { type: 'SET_CSV_DATA'; receiptsType: ReceiptsType; payload: Record<string, unknown>[] }
  | { type: 'SET_DB_ALL'; dividend: DividendData[]; domesticstock: DomesticStockData[]; mutualfund: MutualfundData[] }
  | { type: 'SET_DB_DATA'; receiptsType: ReceiptsType; payload: DividendData[] | DomesticStockData[] | MutualfundData[] }
  | { type: 'SET_DB_LOADING'; payload: boolean }
  | { type: 'SET_DB_ERROR'; payload: string | null }
  | { type: 'SET_SAVING'; payload: boolean }
  | { type: 'SET_DELETING'; payload: boolean }
  | { type: 'SET_SHOW_DELETE_CONFIRM'; payload: boolean }
  | { type: 'LOGOUT' };

export const initialState: ReceiptsState = {
  receiptsType: 'dividend',
  csvData: { dividend: [], domesticstock: [], mutualfund: [] },
  dbData: { dividend: [], domesticstock: [], mutualfund: [] },
  dbLoading: false,
  dbError: null,
  saving: false,
  deleting: false,
  showDeleteConfirm: false,
};

export function receiptsReducer(state: ReceiptsState, action: ReceiptsAction): ReceiptsState {
  switch (action.type) {
    case 'SET_RECEIPTS_TYPE':
      return { ...state, receiptsType: action.payload };
    case 'SET_CSV_DATA':
      return { ...state, csvData: { ...state.csvData, [action.receiptsType]: action.payload } };
    case 'SET_DB_ALL':
      return { ...state, dbData: { dividend: action.dividend, domesticstock: action.domesticstock, mutualfund: action.mutualfund } };
    case 'SET_DB_DATA':
      return { ...state, dbData: { ...state.dbData, [action.receiptsType]: action.payload } };
    case 'SET_DB_LOADING':
      return { ...state, dbLoading: action.payload };
    case 'SET_DB_ERROR':
      return { ...state, dbError: action.payload };
    case 'SET_SAVING':
      return { ...state, saving: action.payload };
    case 'SET_DELETING':
      return { ...state, deleting: action.payload };
    case 'SET_SHOW_DELETE_CONFIRM':
      return { ...state, showDeleteConfirm: action.payload };
    case 'LOGOUT':
      return {
        ...state,
        csvData: { dividend: [], domesticstock: [], mutualfund: [] },
        dbData: { dividend: [], domesticstock: [], mutualfund: [] },
        dbError: null,
      };
  }
}
