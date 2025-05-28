import { ReactNode } from 'react';
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
}

export function ReceiptTemplate({
  title,
  header,
  children,
  footer,
  onSearch,
  searchCategories,
}: ReceiptTemplateProps) {
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

        <div className="card-body p-0">{children}</div>
      </div>

      {/* フッター */}
      {footer && <div>{footer}</div>}
    </div>
  );
}