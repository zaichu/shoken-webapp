import { useId, useState } from 'react';
import { Alert } from '@/components/atoms/Alert';
import {
  DataActionRail,
  type DataActionRailProps,
} from '@/components/organisms/DataActionRail/DataActionRail';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';
import { cn } from '@/lib/utils/classNames';
import type { AssetBalanceData } from '@/types/api';
import type { SearchCategories } from '@/types/common';
import { AssetReviewPromptCard } from './AssetReviewPromptCard';

export interface AssetBalanceUtilityRailProps {
  actionRailProps: DataActionRailProps;
  error: string | null;
  warning?: string | null;
  searchCardProps: {
    visible: boolean;
    categories: SearchCategories;
    value: string;
    onSearch: (query: string) => void;
  };
  reviewPromptCardProps: {
    assetBalanceData: AssetBalanceData[];
  };
}

export function AssetBalanceUtilityRail({
  actionRailProps,
  error,
  warning,
  searchCardProps,
  reviewPromptCardProps,
}: AssetBalanceUtilityRailProps) {
  // Issue #859 追加対応: 取引明細 (ReceiptsUtilityRail, Issue #837) と同じ CSV 折り畳みを
  // そのまま踏襲する。スマホ幅 (<sm = 640px) では CSV 操作を折り畳み式にする。
  // CSV取り込みは都度行う操作ではなく、常時展開が一覧を画面外へ押し出すため。
  // 初期は折り畳み、PC幅ではトグルを表示せず常時展開するので表示は変わらない。
  // 単一 DataActionRail インスタンスを CSS で出し分けし、状態の二重化はしない。
  const [csvExpanded, setCsvExpanded] = useState(false);
  const csvBodyId = useId();

  return (
    <>
      <div>
        {/* スマホ幅 (<sm) のみ: 折り畳み入口。PC幅では hidden */}
        <div className="bg-slate-50/60 px-5 sm:hidden">
          <button
            type="button"
            className="flex min-h-[44px] w-full cursor-pointer items-center justify-between gap-2 text-left select-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-950"
            onClick={() => setCsvExpanded((prev) => !prev)}
            aria-expanded={csvExpanded}
            aria-controls={csvBodyId}
            data-testid="assetbalance-csv-toggle"
          >
            <span className="text-sm font-bold text-slate-800">CSV取り込み・削除</span>{' '}
            <span className="flex shrink-0 items-center gap-1 text-slate-700">
              <span className="text-xs font-semibold">{csvExpanded ? '閉じる' : '開く'}</span>
              <svg
                aria-hidden="true"
                className={cn(
                  'h-4 w-4 text-slate-500 transition-transform duration-200',
                  csvExpanded && 'rotate-180'
                )}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
              </svg>
            </span>
          </button>
        </div>
        <div
          id={csvBodyId}
          role="region"
          aria-label="CSV取り込み・削除"
          className={cn(
            !csvExpanded && 'max-sm:hidden',
            csvExpanded && 'max-sm:border-t max-sm:border-slate-950/10'
          )}
        >
          <DataActionRail {...actionRailProps} />
        </div>
      </div>

      {error && (
        <div className="px-5 py-4">
          <Alert variant="danger" role="alert" aria-live="assertive">
            <strong>エラー:</strong> {error}
          </Alert>
        </div>
      )}

      {warning && (
        <div className="px-5 py-4" role="status" aria-live="polite">
          <Alert variant="warning">{warning}</Alert>
        </div>
      )}

      {searchCardProps.visible && (
        <SearchCard
          onSearch={searchCardProps.onSearch}
          categories={searchCardProps.categories}
          value={searchCardProps.value}
          compact
        />
      )}

      <AssetReviewPromptCard {...reviewPromptCardProps} />
    </>
  );
}
