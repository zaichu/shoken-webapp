import React, { useState } from 'react';
import { NumberInputField } from '@/components/atoms/NumberInputField';
import { StatItem, StatItemWithRate } from '@/components/atoms/StatItem';
import { formatCurrency, parseNumber, normalizeSecurityCode } from '@/lib/utils/formatters';
import { useJQuantsDividend } from '@/features/jquants/hooks/useJQuantsDividend';
import { useAssetBalance } from '@/hooks/common/useAssetBalance';
import { SummaryResult } from '@/lib/utils/dataTransformer';
import { DividendData } from '@/lib/interfaces/dividend';

interface DividendInfoProps {
  searchQuery: string;
  securityCode?: string;
  summary: SummaryResult<keyof Pick<DividendData, 'dividends_before_tax' | 'taxes' | 'net_amount_received'>>[];
}

export const DividendInfo: React.FC<DividendInfoProps> = ({ searchQuery, securityCode, summary }) => {
  const [averageUnitPrice, setAverageUnitPrice] = useState<number | undefined>(undefined);
  const [holdingQuantity, setHoldingQuantity] = useState<number | undefined>(undefined);
  const [dividendPerShare, setDividendPerShare] = useState<number | undefined>(undefined);
  const effectiveSecurityCode = React.useMemo(() => {
    if (securityCode) return normalizeSecurityCode(securityCode);
    const match = searchQuery.match(/^\\s*([0-9A-Za-z]+)\\s*[:：]/);
    return normalizeSecurityCode(match?.[1] || searchQuery);
  }, [securityCode, searchQuery]);

  // 保有銘柄データを取得（常にフェッチ）
  const { assetBalanceData: assetBalances, getAssetBalanceByCode } = useAssetBalance();

  // J-Quants APIから配当情報を取得
  const {
    dividendPerShare: apiDividendPerShare,
    loading: apiLoading,
  } = useJQuantsDividend(effectiveSecurityCode, !!effectiveSecurityCode);

  // searchQueryが変更されたときにstateを初期化し、保有銘柄データがあれば自動入力
  React.useEffect(() => {
    if (effectiveSecurityCode) {
      const assetBalanceData = getAssetBalanceByCode(effectiveSecurityCode);
      if (assetBalanceData) {
        setAverageUnitPrice(assetBalanceData.average_purchase_price);
        setHoldingQuantity(assetBalanceData.shares);
      } else {
        setAverageUnitPrice(undefined);
        setHoldingQuantity(undefined);
      }
    } else {
      setAverageUnitPrice(undefined);
      setHoldingQuantity(undefined);
    }
  }, [effectiveSecurityCode, getAssetBalanceByCode, assetBalances]);

  // APIからデータが取得されたら自動設定
  React.useEffect(() => {
    setDividendPerShare(undefined);
    if (effectiveSecurityCode && apiDividendPerShare !== undefined && apiDividendPerShare > 0) {
      setDividendPerShare(apiDividendPerShare);
    }
  }, [apiDividendPerShare, effectiveSecurityCode]);

  // 各種計算値
  const dividendYield = (() => {
    if (averageUnitPrice && dividendPerShare) {
      return (dividendPerShare / averageUnitPrice) * 100;
    }
    return 0;
  })();

  const annualDividendAmount = parseNumber(holdingQuantity) * parseNumber(dividendPerShare);

  const totalInvestment = parseNumber(averageUnitPrice) * parseNumber(holdingQuantity);

  const dividendReturnRate = (() => {
    if (totalInvestment > 0 && summary[0]) {
      return (summary[0].net_amount_received / totalInvestment) * 100;
    }
    return 0;
  })();

  if (!searchQuery) {
    return null;
  }

  const assetBalanceData = effectiveSecurityCode
    ? getAssetBalanceByCode(effectiveSecurityCode)
    : undefined;

  return (
    <div className="card shadow-sm mt-1">
      <div className="card-header bg-primary text-white d-flex justify-content-between align-items-center">
        <h5 className="mb-0">配当情報</h5>
        {assetBalanceData && (
          <small className="text-light">
            保有銘柄データから自動入力
          </small>
        )}
      </div>
      <div className="card-body">
        <div className="row">
          <div className='col'>
            <NumberInputField label="平均取得価格" value={averageUnitPrice} onChange={setAverageUnitPrice} />
          </div>
          <div className='col'>
            <NumberInputField label="保有数量(株)" value={holdingQuantity} onChange={setHoldingQuantity} />
          </div>
          <div className='col'>
            <NumberInputField
              label="一株配当"
              value={dividendPerShare}
              onChange={setDividendPerShare}
              disabled={apiLoading}
              placeholder={apiLoading ? "データ取得中..." : ""}
            />
          </div>
        </div>

        <div className="row mt-3">
          <StatItem title="取得総額" value={formatCurrency(totalInvestment)} />
          <StatItemWithRate title="合計受取金額 (累積利回り)" value={summary[0]?.net_amount_received || 0} rate={dividendReturnRate} format={formatCurrency} />
          <StatItemWithRate title="年間配当金額 (配当利回り)" value={annualDividendAmount} rate={dividendYield} format={formatCurrency} />
        </div>
      </div>
    </div>
  );
};
