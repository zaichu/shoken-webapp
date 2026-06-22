import React from 'react';

// 共通の型定義
export type FormatFunction<T = unknown> = (value: T) => string | number | React.ReactNode;

export type KpiTone = 'emerald' | 'red' | 'blue' | 'slate';

export interface HeaderItem {
  title: string;
  value: number;
  format: (value: number) => string;
  tone?: KpiTone;
}

export type TableColumnAlignment = 'left' | 'center' | 'right';

// 検索カテゴリ
export interface SearchCategories {
  securities?: { value: string, label: string }[];
  products?: string[];
  accounts?: string[];
  years?: { value: string, label: string }[];
  dates?: boolean; // true の場合、日付検索UIを表示
}
