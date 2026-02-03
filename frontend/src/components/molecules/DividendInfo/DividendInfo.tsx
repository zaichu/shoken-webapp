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
  embedded?: boolean;
}

export const DividendInfo: React.FC<DividendInfoProps> = ({
  searchQuery,
  securityCode,
  summary,
  embedded = false
}) => {
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

  // 全グループの合計受取金額を計算
  const totalNetAmountReceived = React.useMemo(() => {
    return summary.reduce((sum, item) => sum + (item.net_amount_received || 0), 0);
  }, [summary]);

  // 全グループの合計配当金（税引前）を計算
  const totalDividendsBeforeTax = React.useMemo(() => {
    return summary.reduce((sum, item) => sum + (item.dividends_before_tax || 0), 0);
  }, [summary]);

  // 全グループの合計税額を計算
  const totalTaxes = React.useMemo(() => {
    return summary.reduce((sum, item) => sum + (item.taxes || 0), 0);
  }, [summary]);

  const grossDividendReturnRate = (() => {
    if (totalInvestment > 0 && totalDividendsBeforeTax > 0) {
      return (totalDividendsBeforeTax / totalInvestment) * 100;
    }
    return 0;
  })();

  const dividendReturnRate = (() => {
    if (totalInvestment > 0 && totalNetAmountReceived > 0) {
      return (totalNetAmountReceived / totalInvestment) * 100;
    }
    return 0;
  })();

  if (!searchQuery) {
    return null;
  }

  const assetBalanceData = isValidSecurityCode
    ? getAssetBalanceByCode(effectiveSecurityCode)
    : undefined;

  const content = (
    <>
      <div className="stat-grid">
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

      {embedded ? (
        <div className="stat-grid mt-4">
          <StatItemWithRate title="配当金額 (配当利回り)" value={totalDividendsBeforeTax} rate={grossDividendReturnRate} format={formatCurrency} />
          <StatItem title="税額" value={formatCurrency(totalTaxes)} />
          <StatItemWithRate title="受取金額 (累積利回り)" value={totalNetAmountReceived} rate={dividendReturnRate} format={formatCurrency} />
        </div>
      ) : (
        <div className="stat-grid mt-4">
          <StatItem title="取得総額" value={formatCurrency(totalInvestment)} />
          <StatItemWithRate title="合計受取金額 (累積利回り)" value={totalNetAmountReceived} rate={dividendReturnRate} format={formatCurrency} />
          <StatItemWithRate title="年間配当金額 (配当利回り)" value={annualDividendAmount} rate={dividendYield} format={formatCurrency} />
        </div>
      )}
    </>
  );

  if (embedded) {
    return (
      <div className="mt-4 pt-4">
        {assetBalanceData && (
          <div className="mb-4 flex justify-end">
            <small className="text-gray-500">
              保有銘柄データから自動入力
            </small>
          </div>
        )}
        {content}
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-border mt-1">
      <div className="bg-slate-700 text-white px-4 py-2 rounded-t-lg flex justify-between items-center">
        <h5 className="font-semibold">配当情報</h5>
        {assetBalanceData && (
          <small className="text-white/80">
            保有銘柄データから自動入力
          </small>
        )}
      </div>
      <div className="p-4">
        {content}
      </div>
    </div>
  );
};
