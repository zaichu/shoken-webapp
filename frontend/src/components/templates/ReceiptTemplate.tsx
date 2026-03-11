import React, { ReactNode } from 'react';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';
import { Card, CardBody } from '@/components/atoms/Card';
import { ResizeProvider } from '@/contexts/ResizeContext';
import { useTriggerResize } from '@/hooks/common/useResize';
import { SearchCategories } from '@/types/common';

interface ReceiptTemplateProps {
  title?: string;
  header?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  onSearch?: (query: string) => void;
  searchCategories?: SearchCategories;
  onSearchExpandToggle?: (isExpanded: boolean) => void;
  layout?: 'stack' | 'workspace';
  utilityRail?: ReactNode;
}

// 内部コンポーネント（Context内で動作）
const ReceiptTemplateContent: React.FC<ReceiptTemplateProps> = ({
  header,
  children,
  footer,
  onSearch,
  searchCategories,
  onSearchExpandToggle,
  layout: layoutProp,
  utilityRail,
}) => {
  const layout = layoutProp ?? (utilityRail ? 'workspace' : 'stack');
  const triggerResize = useTriggerResize();

  // 検索カードの展開状態変更時の処理
  const handleSearchExpandToggle = (isExpanded: boolean) => {
    // テーブルの強制リサイズをトリガー
    triggerResize?.();
    // 親コンポーネントにも通知
    onSearchExpandToggle?.(isExpanded);
  };

  const searchCard = onSearch ? (
    <SearchCard
      onSearch={onSearch}
      categories={searchCategories}
      onExpandToggle={handleSearchExpandToggle}
      defaultExpanded={layout === 'workspace'}
      compact={layout === 'workspace'}
    />
  ) : null;

  const mainCard = (
    <Card className="mt-1 overflow-hidden rounded-[2rem] border-slate-200/90 bg-white/95 shadow-[0_22px_48px_-36px_rgba(15,23,42,0.45)]" data-testid="receipt-card">
      <CardBody className="p-0" data-testid="receipt-card-body">
        {children}
      </CardBody>
    </Card>
  );

  if (layout === 'workspace') {
    return (
      <div className="space-y-2" data-testid="receipt-container">
        <div
          className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_22rem] lg:items-start xl:gap-6 xl:grid-cols-[minmax(0,1fr)_24rem]"
          data-testid="receipt-workspace"
        >
          <div className="min-w-0 space-y-4" data-testid="receipt-main-stage">
            {header}
            {mainCard}
          </div>
          <aside data-testid="receipt-utility-rail">
            <div className="overflow-hidden rounded-[2rem] border border-slate-200/90 bg-white/80 shadow-[0_20px_48px_-34px_rgba(15,23,42,0.45)] backdrop-blur-sm divide-y divide-slate-200/80">
              {utilityRail}
              {searchCard}
            </div>
          </aside>
        </div>
        {footer && <div>{footer}</div>}
      </div>
    );
  }

  return (
    <div className="space-y-2" data-testid="receipt-container">
      {searchCard}
      {header && <div className="mt-1">{header}</div>}
      {mainCard}
      {footer && <div>{footer}</div>}
    </div >
  );
};

// メインコンポーネント（ResizeProviderでラップ）
export const ReceiptTemplate: React.FC<ReceiptTemplateProps> = (props) => {
  return (
    <ResizeProvider>
      <ReceiptTemplateContent {...props} />
    </ResizeProvider>
  );
};
