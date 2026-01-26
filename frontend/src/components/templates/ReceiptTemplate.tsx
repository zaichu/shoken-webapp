import React, { ReactNode } from 'react';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';
import { ResizeProvider } from '@/contexts/ResizeContext';
import { useTriggerResize } from '@/hooks/common/useResize';
import { SearchCategories } from '@/types/common';

interface ReceiptTemplateProps {
  title: string;
  header?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  onSearch?: (query: string) => void;
  searchCategories?: SearchCategories;
  onSearchExpandToggle?: (isExpanded: boolean) => void;
}

// 内部コンポーネント（Context内で動作）
const ReceiptTemplateContent: React.FC<ReceiptTemplateProps> = ({
  title,
  header,
  children,
  footer,
  onSearch,
  searchCategories,
  onSearchExpandToggle,
}) => {
  const triggerResize = useTriggerResize();

  // 検索カードの展開状態変更時の処理
  const handleSearchExpandToggle = (isExpanded: boolean) => {
    // テーブルの強制リサイズをトリガー
    triggerResize?.();
    // 親コンポーネントにも通知
    onSearchExpandToggle?.(isExpanded);
  };

  return (
    <div className="receipt-container">
      {/* 検索カード */}
      {onSearch && (
        <SearchCard
          onSearch={onSearch}
          categories={searchCategories}
          onExpandToggle={handleSearchExpandToggle}
        />
      )}

      {/* ヘッダー情報 */}
      {header && <div>{header}</div>}

      {/* メインコンテンツ */}
      <div className="card shadow-sm mt-1">
        <div className="card-header bg-primary text-white py-2">
          <h5 className="mb-0 lh-base">{title}</h5>
        </div>

        <div className="card-body p-0">{children}</div>
      </div>

      {/* フッター */}
      {footer && <div>{footer}</div>}
    </div>
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