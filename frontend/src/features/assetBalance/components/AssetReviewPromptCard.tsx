import { useState } from 'react';
import { Button } from '@/components/atoms/Button';
import type { AssetBalanceData } from '@/types/api';
import { generateAssetReviewPrompt } from '../utils/assetReviewPrompt';
export interface AssetReviewPromptCardProps {
  assetBalanceData: AssetBalanceData[];
}
type CopyStatus = 'idle' | 'success' | 'error';
export function AssetReviewPromptCard({ assetBalanceData }: AssetReviewPromptCardProps) {
  const [status, setStatus] = useState<CopyStatus>('idle');
  const handleCopy = async () => {
    const prompt = generateAssetReviewPrompt(assetBalanceData);
    try {
      await navigator.clipboard.writeText(prompt);
      setStatus('success');
      setTimeout(() => setStatus('idle'), 3000);
    } catch {
      setStatus('error');
      setTimeout(() => setStatus('idle'), 3000);
    }
  };
  const disabled = assetBalanceData.length === 0;
  const buttonLabel = status === 'success'
    ? 'コピーしました！'
    : status === 'error'
      ? 'コピーに失敗しました'
      : 'AI総評プロンプトをコピー';
  return (
    <div className="px-5 py-4">
      <p className="text-xs font-medium text-secondary mb-2">AI総評プロンプト</p>
      <Button
        variant="outline-secondary"
        size="sm"
        fullWidth
        onClick={handleCopy}
        disabled={disabled}
        className="truncate"
        aria-label={buttonLabel}
      >
        {buttonLabel}
      </Button>
    </div>
  );
}
