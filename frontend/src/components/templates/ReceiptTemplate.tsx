import { ReactNode } from 'react';

interface SearchOption {
  value: string;
  label: string;
}

interface ReceiptTemplateProps {
  title: string;
  children: ReactNode;
  searchQuery?: string;
  onSearch?: (query: string) => void;
  searchOptions?: SearchOption[];
}

export const ReceiptTemplate = ({
  title,
  children,
  searchQuery = '',
  onSearch,
  searchOptions = []
}: ReceiptTemplateProps) => {
  const handleSearchChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    if (onSearch) {
      onSearch(e.target.value);
    }
  };

  return (
    <div className="receipt-container">
      <div className="card shadow-sm">
        <div className="card-header bg-info text-white">
          <div className="row align-items-center">
            <div className="col">
              <h5 className="mb-0">{title}</h5>
            </div>
            {onSearch && searchOptions.length > 0 && (
              <div className="col-md-4">
                <div className="d-flex align-items-center">
                  <label htmlFor="security-search" className="me-2 mb-0">銘柄コード:</label>
                  <select
                    id="security-search"
                    className="form-select form-select-sm"
                    value={searchQuery}
                    onChange={handleSearchChange}
                  >
                    <option value="">全て表示</option>
                    {searchOptions.map((option, index) => (
                      <option key={index} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            )}
          </div>
        </div>
        <div className="card-body p-0">
          {children}
        </div>
      </div>
    </div>
  );
}
