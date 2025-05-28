import { FormEvent, memo, useCallback } from 'react';
import { Button } from '../atoms/Button';
import { InputField } from '../atoms/InputField';

interface SearchFormProps {
  stockCode: string;
  onStockCodeChange: (value: string) => void;
  onSubmit: (e: FormEvent) => void;
  isLoading?: boolean;
}

export const SearchForm = memo<SearchFormProps>(({
  stockCode,
  onStockCodeChange,
  onSubmit,
  isLoading = false
}) => {
  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onStockCodeChange(e.target.value);
    },
    [onStockCodeChange]
  );

  const handleSubmit = useCallback(
    (e: FormEvent) => {
      e.preventDefault();
      if (!isLoading && stockCode) {
        onSubmit(e);
      }
    },
    [onSubmit, isLoading, stockCode]
  );

  return (
    <form onSubmit={handleSubmit} className="mb-4">
      <div className="input-group">
        <InputField
          type="text"
          className="form-control"
          placeholder="銘柄コードを入力"
          value={stockCode}
          onChange={handleInputChange}
          style={{ maxWidth: '200px' }}
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
});

SearchForm.displayName = 'SearchForm';
