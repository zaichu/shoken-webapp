import React from 'react';
import type { ReceiptsType } from '../reducer';

const TAB_LABEL: Record<ReceiptsType, string> = {
  dividend: '配当金',
  domesticstock: '国内株式',
  mutualfund: '投資信託',
};

const TABS = ['dividend', 'domesticstock', 'mutualfund'] as const;

interface ReceiptsTabNavProps {
  receiptsType: ReceiptsType;
  tablistRef: React.RefObject<HTMLDivElement | null>;
  onTabChange: (tab: ReceiptsType) => void;
  onKeyDown: (event: React.KeyboardEvent<HTMLButtonElement>) => void;
  counts: Record<ReceiptsType, number>;
}

export function ReceiptsTabNav({
  receiptsType,
  tablistRef,
  onTabChange,
  onKeyDown,
  counts,
}: ReceiptsTabNavProps) {
  return (
    <nav className="mb-2 no-print" aria-label="取引明細タブ">
      <div
        className="flex flex-wrap gap-1.5 rounded-xl border border-slate-950/10 bg-white/70 p-1 shadow-sm"
        role="tablist"
        ref={tablistRef}
      >
        {TABS.map((tab) => {
          const isActive = receiptsType === tab;
          return (
            <button
              key={tab}
              id={`tab-${tab}`}
              className={`inline-flex items-center gap-2 rounded-lg border px-4 py-2.5 text-sm font-bold transition-[color,background-color,border-color,box-shadow] ${
                isActive
                  ? 'border-slate-950 bg-slate-950 text-white shadow-[inset_0_-2px_0_#f59e0b]'
                  : 'border-transparent text-slate-500 hover:bg-white hover:text-slate-900'
              }`}
              onClick={() => onTabChange(tab)}
              onKeyDown={onKeyDown}
              type="button"
              role="tab"
              aria-selected={isActive}
              aria-controls={`tabpanel-${tab}`}
              tabIndex={isActive ? 0 : -1}
            >
              {TAB_LABEL[tab]}
              <span
                data-testid={`tab-count-${tab}`}
                className={`inline-flex min-w-6 items-center justify-center rounded-full px-2 py-0.5 text-xs font-semibold ${
                  isActive
                    ? 'border border-white/20 bg-white text-slate-950'
                    : 'bg-slate-100 text-slate-500'
                }`}
              >
                {counts[tab]}
              </span>
            </button>
          );
        })}
      </div>
    </nav>
  );
}

export { TABS, TAB_LABEL };
