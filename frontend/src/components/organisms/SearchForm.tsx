import { FormEvent } from 'react';
import { Button } from '../atoms/Button';
import { InputField } from '../atoms/InputField';

interface SearchFormProps {
  stockCode: string;
  onStockCodeChange: (value: string) => void;
  onSubmit: (e: FormEvent) => void;
  isLoading?: boolean;
}

export function SearchForm({
  stockCode,
  onStockCodeChange,
  onSubmit,
  isLoading = false
}: SearchFormProps) {
  return (
    <form onSubmit={onSubmit} className="mb-4">
      <div className="input-group">
        <InputField
          type="text"
          className="form-control"
          placeholder="銘柄コードを入力"
          value={stockCode}
          onChange={(e) => onStockCodeChange(e.target.value)}
          style={{ maxWidth: '200px' }}
        />
        <Button
          type="submit"
          variant="primary"
          disabled={isLoading || !stockCode}
        >
          {isLoading ? '検索中...' : '検索'}
        </Button>
      </div>
    </form>
  );
}
