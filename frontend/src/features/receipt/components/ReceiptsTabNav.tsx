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
    <nav className="mb-5 no-print" aria-label="取引明細タブ">
      <div
        className="flex flex-wrap gap-1.5 border-b border-slate-200/90"
        role="tablist"
        ref={tablistRef}
      >
        {TABS.map((tab) => {
          const isActive = receiptsType === tab;
          return (
            <button
              key={tab}
              id={`tab-${tab}`}
              className={`-mb-px inline-flex items-center gap-2 rounded-t-2xl border border-transparent border-b-0 px-4 py-3 text-sm font-medium transition-[color,background-color,border-color,box-shadow] ${
                isActive
                  ? 'border-slate-200 bg-white text-slate-900 shadow-[0_-1px_0_0_rgba(255,255,255,1),0_18px_32px_-30px_rgba(15,23,42,0.7)]'
                  : 'text-slate-500 hover:bg-white/80 hover:text-slate-800'
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
                    ? 'border border-slate-200 bg-slate-100 text-slate-700'
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
