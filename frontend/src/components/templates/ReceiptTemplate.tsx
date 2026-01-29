import React, { ReactNode } from 'react';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';
import { Card, CardBody, CardHeader } from '@/components/atoms/Card';
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
    <div className="space-y-3" data-testid="receipt-container">
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
      <Card className="mt-1" data-testid="receipt-card">
        <CardBody data-testid="receipt-card-body">
          {children}
        </CardBody>
      </Card>

      {/* フッター */}
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
