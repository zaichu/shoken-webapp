import React from 'react';
import { type ReceiptsType } from './receiptsReducer';

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
  onKeyDown: (e: React.KeyboardEvent<HTMLButtonElement>) => void;
}

export function ReceiptsTabNav({
  receiptsType,
  tablistRef,
  onTabChange,
  onKeyDown,
}: ReceiptsTabNavProps) {
  return (
    <nav className="border-b border-slate-200 no-print" aria-label="取引明細タブ">
      <div className="flex flex-wrap gap-1" role="tablist" ref={tablistRef}>
        {TABS.map((tab) => {
          const isActive = receiptsType === tab;
          return (
            <button
              key={tab}
              id={`tab-${tab}`}
              className={`px-4 py-3 text-sm font-medium transition-colors border-b-2 -mb-px ${
                isActive
                  ? 'border-primary text-primary bg-white'
                  : 'border-transparent text-slate-600 hover:text-slate-900 hover:border-slate-300'
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
            </button>
          );
        })}
      </div>
    </nav>
  );
}

export { TABS, TAB_LABEL };
