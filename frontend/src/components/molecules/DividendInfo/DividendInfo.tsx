import React, { useState } from 'react';
import { NumberInputField } from '@/components/atoms/NumberInputField';
import { StatItem, StatItemWithRate } from '@/components/atoms/StatItem';
import { formatCurrency, parseNumber, normalizeSecurityCode, SECURITY_CODE_REGEX } from '@/lib/utils/formatters';
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

  // 銘柄コード形式かどうかを判定（商品/口座/年月検索では不要なAPI呼び出しを防ぐ）
  const isValidSecurityCode = React.useMemo(() => {
    return !!effectiveSecurityCode && SECURITY_CODE_REGEX.test(effectiveSecurityCode);
  }, [effectiveSecurityCode]);

  // 保有銘柄データを取得（常にフェッチ）
  const { assetBalanceData: assetBalances, getAssetBalanceByCode } = useAssetBalance();

  // J-Quants APIから配当情報を取得（銘柄コード形式の場合のみ）
  const {
    dividendPerShare: apiDividendPerShare,
    loading: apiLoading,
  } = useJQuantsDividend(effectiveSecurityCode, isValidSecurityCode);

  // searchQueryが変更されたときにstateを初期化し、保有銘柄データがあれば自動入力
  React.useEffect(() => {
    if (isValidSecurityCode) {
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
  }, [effectiveSecurityCode, isValidSecurityCode, getAssetBalanceByCode, assetBalances]);

  // APIからデータが取得されたら自動設定
  React.useEffect(() => {
    setDividendPerShare(undefined);
    if (isValidSecurityCode && apiDividendPerShare !== undefined && apiDividendPerShare > 0) {
      setDividendPerShare(apiDividendPerShare);
    }
  }, [apiDividendPerShare, isValidSecurityCode]);

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

  const assetBalanceData = isValidSecurityCode
    ? getAssetBalanceByCode(effectiveSecurityCode)
    : undefined;

  return (
    <div className="bg-white rounded-lg shadow-sm border border-border mt-1">
      <div className="bg-primary text-white px-4 py-2 rounded-t-lg flex justify-between items-center">
        <h5 className="font-semibold">配当情報</h5>
        {assetBalanceData && (
          <small className="text-white/80">
            保有銘柄データから自動入力
          </small>
        )}
      </div>
      <div className="p-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <NumberInputField label="平均取得価格" value={averageUnitPrice} onChange={setAverageUnitPrice} />
          </div>
          <div>
            <NumberInputField label="保有数量(株)" value={holdingQuantity} onChange={setHoldingQuantity} />
          </div>
          <div>
            <NumberInputField
              label="一株配当"
              value={dividendPerShare}
              onChange={setDividendPerShare}
              disabled={apiLoading}
              placeholder={apiLoading ? "データ取得中..." : ""}
            />
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
          <StatItem title="取得総額" value={formatCurrency(totalInvestment)} />
          <StatItemWithRate title="合計受取金額 (累積利回り)" value={summary[0]?.net_amount_received || 0} rate={dividendReturnRate} format={formatCurrency} />
          <StatItemWithRate title="年間配当金額 (配当利回り)" value={annualDividendAmount} rate={dividendYield} format={formatCurrency} />
        </div>
      </div>
    </div>
  );
};
