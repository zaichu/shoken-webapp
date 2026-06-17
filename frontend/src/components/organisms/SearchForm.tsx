import React, { FormEvent } from 'react';
import { Button } from '../atoms/Button';
import { InputField } from '../atoms/InputField';
import { Card, CardBody } from '../atoms/Card';

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
    <Card className="mb-5 overflow-hidden border-slate-950/10">
      <CardBody className="p-4 sm:p-5">
        <form onSubmit={handleSubmit}>
          <div className="flex w-full items-stretch rounded-lg border border-slate-300 bg-white p-1 shadow-inner shadow-slate-200/80 focus-within:border-amber-600 focus-within:ring-2 focus-within:ring-amber-500/20">
            <div className="flex-1 min-w-0">
              <InputField
                type="text"
                className="h-11 rounded-r-none border-0 bg-transparent text-base shadow-none focus:ring-0"
                placeholder="銘柄コードまたは銘柄名"
                value={stockCode}
                onChange={handleInputChange}
                aria-label="銘柄コードまたは銘柄名"
                autoComplete="off"
                disabled={isLoading}
                fullWidth
              />
            </div>
            <Button
              type="submit"
              variant="primary"
              disabled={isLoading || !stockCode}
              loading={isLoading}
              aria-label={isLoading ? '検索中' : '銘柄を検索'}
              className="h-11 shrink-0 whitespace-nowrap rounded-md px-5"
            >
              {isLoading ? '検索中...' : '検索'}
            </Button>
          </div>
        </form>
      </CardBody>
    </Card>
  );
};
