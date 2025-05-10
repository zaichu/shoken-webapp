// 共通の型定義
export type FormatFunction<T = unknown> = (value: T) => string | number;

export interface HeaderItem {
  title: string;
  value: number | string;
  format?: FormatFunction;
}

export interface SelectOption {
  value: string;
  label: string;
}

export type TableColumnAlignment = 'left' | 'center' | 'right';

export interface BaseColumnConfig {
  key: string;
  header: string;
  width?: string;
  textAlign?: TableColumnAlignment;
  format?: FormatFunction;
}

// ジェネリック型定義
export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;
export type AsyncFunction<T = void> = () => Promise<T>;
export type VoidFunction = () => void;

// レスポンスの基本型
export interface BaseResponse {
  success: boolean;
  message?: string;
}

export interface DataResponse<T> extends BaseResponse {
  data: T;
}

export interface ErrorResponse extends BaseResponse {
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
}

// ページネーション
export interface PaginationParams {
  page: number;
  pageSize: number;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}
