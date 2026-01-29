import React from 'react';

// 共通の型定義
export type FormatFunction<T = unknown> = (value: T) => string | number | React.ReactNode;

export interface HeaderItem {
  title: string;
  value: number | string;
  format?: FormatFunction;
}

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
  group?: string;
}

export type TableColumnAlignment = 'left' | 'center' | 'right';

export interface BaseColumnConfig {
  key: string;
  header: string;
  width?: string;
  textAlign?: TableColumnAlignment;
  format?: FormatFunction;
  sortable?: boolean;
  filterable?: boolean;
  className?: string;
  headerClassName?: string;
}

// ジェネリック型定義
export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;
export type AsyncFunction<T = void> = () => Promise<T>;
export type VoidFunction = () => void;
export type ValueOf<T> = T[keyof T];
export type DeepPartial<T> = T extends object ? {
  [P in keyof T]?: DeepPartial<T[P]>;
} : T;
export type DeepReadonly<T> = T extends object ? {
  readonly [P in keyof T]: DeepReadonly<T[P]>;
} : T;

// ユーティリティ型
export type ExtractKeys<T, U> = {
  [K in keyof T]: T[K] extends U ? K : never;
}[keyof T];

export type RequireAtLeastOne<T, Keys extends keyof T = keyof T> =
  Pick<T, Exclude<keyof T, Keys>> &
  {
    [K in Keys]-?: Required<Pick<T, K>> & Partial<Pick<T, Exclude<Keys, K>>>;
  }[Keys];

export type RequireOnlyOne<T, Keys extends keyof T = keyof T> =
  Pick<T, Exclude<keyof T, Keys>> &
  {
    [K in Keys]-?: Required<Pick<T, K>> & Partial<Record<Exclude<Keys, K>, undefined>>;
  }[Keys];

// レスポンスの基本型
export interface BaseResponse {
  success: boolean;
  message?: string;
  timestamp?: string;
}

export interface DataResponse<T> extends BaseResponse {
  data: T;
  metadata?: Record<string, unknown>;
}

export interface ErrorResponse extends BaseResponse {
  success: false;
  error: {
    code: string;
    message: string;
    details?: unknown;
    stack?: string;
  };
}

export interface ListResponse<T> extends BaseResponse {
  items: T[];
  totalCount: number;
}

// ページネーション
export interface PaginationParams {
  page: number;
  pageSize: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResponse<T> extends ListResponse<T> {
  page: number;
  pageSize: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

// フィルタリング
export interface FilterOperator {
  equals?: unknown;
  notEquals?: unknown;
  contains?: string;
  startsWith?: string;
  endsWith?: string;
  gt?: number | Date;
  gte?: number | Date;
  lt?: number | Date;
  lte?: number | Date;
  in?: unknown[];
  notIn?: unknown[];
  isNull?: boolean;
  isNotNull?: boolean;
}

export interface FilterCondition {
  field: string;
  operator: keyof FilterOperator;
  value: unknown;
}

export interface FilterParams {
  conditions: FilterCondition[];
  logic?: 'AND' | 'OR';
}

// フォーム関連
export interface FormField<T = unknown> {
  value: T;
  error?: string;
  touched: boolean;
  pristine: boolean;
  validating: boolean;
}

export interface FormState<T extends Record<string, unknown>> {
  values: T;
  errors: Partial<Record<keyof T, string>>;
  touched: Partial<Record<keyof T, boolean>>;
  isSubmitting: boolean;
  isValidating: boolean;
  isValid: boolean;
  isDirty: boolean;
}

// イベントハンドラー
export type ChangeHandler<T = unknown> = (value: T) => void;
export type SubmitHandler<T = unknown> = (values: T) => void | Promise<void>;
export type ValidationFunction<T = unknown> = (value: T) => string | undefined | Promise<string | undefined>;

// APIリクエスト状態
export interface RequestState<T = unknown> {
  data: T | null;
  loading: boolean;
  error: Error | null;
  fetching: boolean;
  lastFetch: Date | null;
}

// キャッシュ関連
export interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

export interface CacheOptions {
  ttl?: number; // Time to live in milliseconds
  staleWhileRevalidate?: boolean;
  invalidateOnError?: boolean;
}

// 通知関連
export type NotificationType = 'info' | 'success' | 'warning' | 'error';

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message?: string;
  duration?: number;
  actions?: Array<{
    label: string;
    handler: VoidFunction;
  }>;
}

// テーマ関連
export interface Theme {
  colors: {
    primary: string;
    secondary: string;
    success: string;
    info: string;
    warning: string;
    danger: string;
    light: string;
    dark: string;
  };
  typography: {
    fontFamily: string;
    fontSize: {
      xs: string;
      sm: string;
      base: string;
      lg: string;
      xl: string;
    };
  };
  spacing: {
    xs: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
  };
  breakpoints: {
    sm: string;
    md: string;
    lg: string;
    xl: string;
  };
}

// アクセシビリティ関連
export interface A11yProps {
  role?: string;
  'aria-label'?: string;
  'aria-labelledby'?: string;
  'aria-describedby'?: string;
  'aria-hidden'?: boolean;
  'aria-live'?: 'polite' | 'assertive' | 'off';
  'aria-busy'?: boolean;
  'aria-disabled'?: boolean;
  'aria-expanded'?: boolean;
  'aria-controls'?: string;
  tabIndex?: number;
}

// メタデータ
export interface Metadata {
  createdAt: Date;
  updatedAt: Date;
  createdBy?: string;
  updatedBy?: string;
  version?: number;
  tags?: string[];
}

// 検索カテゴリ
export interface SearchCategories {
  securities?: { value: string, label: string }[];
  products?: string[];
  accounts?: string[];
  years?: { value: string, label: string }[];
  yearMonths?: { value: string, label: string }[];
}

// エクスポート
export type { 
  HeaderItem as HeaderItemType,
  SelectOption as SelectOptionType,
  BaseColumnConfig as ColumnConfig,
};
