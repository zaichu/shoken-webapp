import React, { useState } from 'react';
import { NumberInputField } from '@/components/atoms/NumberInputField';
import { StatItem, StatItemWithRate } from '@/components/atoms/StatItem';
import { formatCurrency, parseNumber, normalizeSecurityCode, SECURITY_CODE_REGEX } from '@/lib/utils/formatters';
import { useDividendBatch } from '@/features/jquants/hooks/useDividendBatch';
import { useAssetBalance } from '@/features/assetBalance/hooks/useAssetBalance';
import { SummaryResult } from '@/lib/utils/dataTransformer';
import { DividendData } from '@/lib/interfaces/dividend';

const AssetBadge = () => (
  <span className="ml-1 inline-flex items-center rounded bg-emerald-100 px-1.5 py-0.5 text-xs font-medium text-emerald-700">
    保有銘柄
  </span>
);

const ASSET_BALANCE_HINT = '資産管理にCSVを取り込むと表示されます';
const JQUANTS_HINT = '自動で取得されます';

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

  const isValidSecurityCode = React.useMemo(() => {
    return !!effectiveSecurityCode && SECURITY_CODE_REGEX.test(effectiveSecurityCode);
  }, [effectiveSecurityCode]);

  const { assetBalanceData: assetBalances, getAssetBalanceByCode } = useAssetBalance({ enabled: isValidSecurityCode });

  const dividendBatchCodes = React.useMemo(
    () => (isValidSecurityCode ? [effectiveSecurityCode] : []),
    [effectiveSecurityCode, isValidSecurityCode]
  );
  const { dividendPerShareMap, loading: apiLoading } = useDividendBatch(dividendBatchCodes, isValidSecurityCode);
  const apiDividendPerShare = dividendPerShareMap.get(effectiveSecurityCode);

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

  React.useEffect(() => {
    setDividendPerShare(undefined);
    if (isValidSecurityCode && apiDividendPerShare !== undefined && apiDividendPerShare > 0) {
      setDividendPerShare(apiDividendPerShare);
    }
  }, [apiDividendPerShare, isValidSecurityCode]);

  const dividendYield = (() => {
    if (averageUnitPrice && dividendPerShare) {
      return (dividendPerShare / averageUnitPrice) * 100;
    }
    return 0;
  })();

  const annualDividendAmount = parseNumber(holdingQuantity) * parseNumber(dividendPerShare);
  const totalInvestment = parseNumber(averageUnitPrice) * parseNumber(holdingQuantity);

  const totalNetAmountReceived = React.useMemo(() => {
    return summary.reduce((sum, item) => sum + (item.net_amount_received || 0), 0);
  }, [summary]);

  const totalDividendsBeforeTax = React.useMemo(() => {
    return summary.reduce((sum, item) => sum + (item.dividends_before_tax || 0), 0);
  }, [summary]);

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

  // embedded モード: 資産管理・J-Quants データを read-only KPI カードで表示
  if (embedded) {
    return (
      <div>
        {/* パラメータ行（資産管理・J-Quants データ） */}
        <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
          <div className="rounded-lg bg-slate-50 px-4 py-3">
            <p className="text-xs font-medium text-slate-600 mb-1">
              平均取得価格{assetBalanceData && <AssetBadge />}
            </p>
            <p
              className="text-2xl font-bold tabular-nums text-primary"
              title={averageUnitPrice === undefined ? ASSET_BALANCE_HINT : undefined}
            >
              {averageUnitPrice !== undefined ? formatCurrency(averageUnitPrice) : '---'}
            </p>
            {averageUnitPrice === undefined && (
              <p className="mt-0.5 text-xs text-slate-400">{ASSET_BALANCE_HINT}</p>
            )}
          </div>
          <div className="rounded-lg bg-slate-50 px-4 py-3">
            <p className="text-xs font-medium text-slate-600 mb-1">
              保有数量(株){assetBalanceData && <AssetBadge />}
            </p>
            <p
              className="text-2xl font-bold tabular-nums text-primary"
              title={holdingQuantity === undefined ? ASSET_BALANCE_HINT : undefined}
            >
              {holdingQuantity !== undefined ? holdingQuantity.toLocaleString('ja-JP') : '---'}
            </p>
            {holdingQuantity === undefined && (
              <p className="mt-0.5 text-xs text-slate-400">{ASSET_BALANCE_HINT}</p>
            )}
          </div>
          <div className="rounded-lg bg-slate-50 px-4 py-3">
            <p className="text-xs font-medium text-slate-600 mb-1">
              一株配当
            </p>
            <p className="text-2xl font-bold tabular-nums text-primary">
              {apiLoading ? '取得中...' : dividendPerShare !== undefined ? formatCurrency(dividendPerShare) : '---'}
            </p>
            {!apiLoading && dividendPerShare === undefined && (
              <p className="mt-0.5 text-xs text-slate-400">{JQUANTS_HINT}</p>
            )}
          </div>
        </div>

        {/* 結果行（受取実績） */}
        <div className="grid grid-cols-1 gap-4 md:grid-cols-3 mt-4 pt-4 border-t border-slate-200">
          <div className="rounded-lg bg-emerald-50 px-4 py-3">
            <p className="text-xs font-medium text-slate-600 mb-1">配当金額 (配当利回り)</p>
            <p className="text-2xl font-bold tabular-nums text-emerald-600">
              {formatCurrency(totalDividendsBeforeTax)}
              {grossDividendReturnRate > 0 && (
                <span className="text-sm font-normal text-slate-500 ml-1">
                  ({grossDividendReturnRate.toFixed(2)}%)
                </span>
              )}
            </p>
          </div>
          <div className="rounded-lg bg-red-50 px-4 py-3">
            <p className="text-xs font-medium text-slate-600 mb-1">税額</p>
            <p className="text-2xl font-bold tabular-nums text-red-500">
              {formatCurrency(totalTaxes)}
            </p>
          </div>
          <div className="rounded-lg bg-emerald-50 px-4 py-3">
            <p className="text-xs font-medium text-slate-600 mb-1">受取金額 (累積利回り)</p>
            <p className="text-2xl font-bold tabular-nums text-emerald-600">
              {formatCurrency(totalNetAmountReceived)}
              {dividendReturnRate > 0 && (
                <span className="text-sm font-normal text-slate-500 ml-1">
                  ({dividendReturnRate.toFixed(2)}%)
                </span>
              )}
            </p>
          </div>
        </div>
      </div>
    );
  }

  // standalone モード: シミュレーション（入力フォーム）
  return (
    <div className="bg-white rounded-lg shadow-sm border border-border mt-1">
      <div className="bg-slate-600 text-white px-3 py-1.5 rounded-t-lg">
        <h5 className="text-sm font-medium">配当シミュレーション</h5>
      </div>
      <div className="p-3">
        <div className="stat-grid">
          <div>
            <NumberInputField
              label={<>平均取得価格{assetBalanceData && <AssetBadge />}</>}
              value={averageUnitPrice}
              onChange={setAverageUnitPrice}
            />
          </div>
          <div>
            <NumberInputField
              label={<>保有数量(株){assetBalanceData && <AssetBadge />}</>}
              value={holdingQuantity}
              onChange={setHoldingQuantity}
            />
          </div>
          <div>
            <NumberInputField
              label={<>一株配当</>}
              value={dividendPerShare}
              onChange={setDividendPerShare}
              disabled={apiLoading}
              placeholder={apiLoading ? "データ取得中..." : ""}
            />
          </div>
        </div>
        <div className="stat-grid mt-4">
          <StatItem title="取得総額" value={formatCurrency(totalInvestment)} />
          <StatItemWithRate title="合計受取金額 (累積利回り)" value={totalNetAmountReceived} rate={dividendReturnRate} format={formatCurrency} />
          <StatItemWithRate title="年間配当金額 (配当利回り)" value={annualDividendAmount} rate={dividendYield} format={formatCurrency} />
        </div>
      </div>
    </div>
  );
};
