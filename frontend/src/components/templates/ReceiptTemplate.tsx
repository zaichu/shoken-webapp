import { ReactNode, useState, useCallback, cloneElement, Children, isValidElement } from 'react';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';

interface SearchCategories {
  securities?: { value: string, label: string }[];
  products?: string[];
  accounts?: string[];
  years?: string[];
  yearMonths?: { value: string, label: string }[];
}

interface ReceiptTemplateProps {
  title: string;
  header?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  onSearch?: (query: string) => void;
  searchCategories?: SearchCategories;
  onSearchExpandToggle?: (isExpanded: boolean) => void; // 検索の展開状態変更通知
}

export function ReceiptTemplate({
  title,
  header,
  children,
  footer,
  onSearch,
  searchCategories,
  onSearchExpandToggle,
}: ReceiptTemplateProps) {
  const [forceResizeCounter, setForceResizeCounter] = useState(0);

  // 検索カードの展開状態変更時の処理
  const handleSearchExpandToggle = useCallback((isExpanded: boolean) => {
    // テーブルの強制リサイズをトリガー
    setForceResizeCounter(prev => prev + 1);
    // 親コンポーネントにも通知
    onSearchExpandToggle?.(isExpanded);
  }, [onSearchExpandToggle]);

  // 子コンポーネント（ReceiptTable）にforceResizeプロパティを追加
  const enhancedChildren = Children.map(children, (child) => {
    if (isValidElement(child)) {
      // ReceiptTableコンポーネントの場合、forceResizeプロパティを追加
      if ('type' in child && typeof child.type === 'function' && 
          (child.type.name === 'ReceiptTable' || child.type.displayName === 'ReceiptTable')) {
        return cloneElement(child, { 
          ...child.props,
          forceResize: forceResizeCounter 
        });
      }
    }
    return child;
  });

  return (
    <div className="receipt-container">
      {/* 検索カード */}
      {onSearch && (
        <SearchCard
          onSearch={onSearch}
          categories={{
            securities: searchCategories?.securities,
            products: searchCategories?.products,
            accounts: searchCategories?.accounts,
            years: searchCategories?.years,
            yearMonths: searchCategories?.yearMonths,
          }}
          onExpandToggle={handleSearchExpandToggle}
        />
      )}

      {/* ヘッダー情報 */}
      {header && <div>{header}</div>}

      {/* メインコンテンツ */}
      <div className="card shadow-sm mt-1">
        <div className="card-header bg-primary text-white">
          <div className="row align-items-center">
            <div className="col-12">
              <h5 className='mb-0'>{title}</h5>
            </div>
          </div>
        </div>

        <div className="card-body p-0">{enhancedChildren}</div>
      </div>

      {/* フッター */}
      {footer && <div>{footer}</div>}
    </div>
  );
}