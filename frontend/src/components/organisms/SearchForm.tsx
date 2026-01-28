import React, { FormEvent } from 'react';
import { Button } from '../atoms/Button';
import { InputField } from '../atoms/InputField';

interface SearchFormProps {
  stockCode: string;
  onStockCodeChange: (value: string) => void;
  onSubmit: (e: FormEvent) => void;
  isLoading?: boolean;
}

export const SearchForm: React.FC<SearchFormProps> = ({
  stockCode,
  onStockCodeChange,
  onSubmit,
  isLoading = false
}) => {
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onStockCodeChange(e.target.value);
  };

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!isLoading && stockCode) {
      onSubmit(e);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="search-form mb-4">
      <div className="input-group">
        <InputField
          type="text"
          className="search-form-input"
          placeholder="銘柄コードを入力"
          value={stockCode}
          onChange={handleInputChange}
          aria-label="銘柄コード"
          autoComplete="off"
          disabled={isLoading}
        />
        <Button
          type="submit"
          variant="primary"
          disabled={isLoading || !stockCode}
          loading={isLoading}
          aria-label={isLoading ? '検索中' : '銘柄を検索'}
        >
          {isLoading ? '検索中...' : '検索'}
        </Button>
      </div>
    </form>
  );
};
