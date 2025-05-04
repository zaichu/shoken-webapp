import { ReactNode, ChangeEvent } from 'react';

interface SearchOption {
  value: string;
  label: string;
}

interface SearchProps {
  searchQuery?: string;
  onSearch?: (query: string) => void;
  searchOptions?: SearchOption[];
}

interface ReceiptTemplateProps extends SearchProps {
  title: string;
  header?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
}

const SearchControl = ({ searchQuery = '', onSearch, searchOptions = [] }: SearchProps) => {
  if (!onSearch || searchOptions.length === 0) return null;

  const handleSearchChange = (e: ChangeEvent<HTMLSelectElement>) => {
    onSearch(e.target.value);
  };

  return (
    <div className="col-md-4">
      <div className="d-flex align-items-center">
        <select
          id="security-search"
          className="form-select form-select-sm"
          value={searchQuery}
          onChange={handleSearchChange}
          aria-label="検索フィルター"
        >
          <option value="">全て表示</option>
          {searchOptions.map((option, index) => (
            <option key={`search-option-${option.value}-${index}`} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
};

export function ReceiptTemplate({
  title,
  header,
  children,
  footer,
  searchQuery = '',
  onSearch,
  searchOptions = []
}: ReceiptTemplateProps) {
  const hasSearchFeature = onSearch && searchOptions.length > 0;

  return (
    <div className="receipt-container">
      {header && <div className="mb-3">{header}</div>}

      <div className="card shadow-sm mt-2">
        <div className="card-header bg-primary text-white">
          <div className="row align-items-center">
            <div className={`col ${hasSearchFeature ? '' : 'col-12'}`}>
              <h5 className="mb-0">{title}</h5>
            </div>

            {hasSearchFeature && (
              <SearchControl
                searchQuery={searchQuery}
                onSearch={onSearch}
                searchOptions={searchOptions}
              />
            )}
          </div>
        </div>

        <div className="card-body p-0">
          {children}
        </div>

        {footer && <div className="card-footer">{footer}</div>}
      </div>
    </div>
  );
}