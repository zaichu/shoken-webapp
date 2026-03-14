import { Card, CardContent } from '@/components/ui/card'

type Props = {
  totalValue: number
  totalCost: number
  totalPnl: number
  totalPnlRate: number
  stockCount: number
}

function fmt(n: number) {
  return n.toLocaleString('ja-JP')
}

function PnlText({ value, rate }: { value: number; rate: number }) {
  const positive = value >= 0
  const color = positive ? 'text-emerald-600' : 'text-red-600'
  const sign = positive ? '+' : ''
  return (
    <span className={color}>
      {sign}{fmt(value)}円 ({sign}{rate.toFixed(2)}%)
    </span>
  )
}

export function PortfolioSummary({ totalValue, totalCost, totalPnl, totalPnlRate, stockCount }: Props) {
  return (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
      <Card>
        <CardContent className="pt-5">
          <p className="text-xs text-gray-500 mb-1">銘柄数</p>
          <p className="text-2xl font-semibold tabular-nums">{stockCount}</p>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-5">
          <p className="text-xs text-gray-500 mb-1">評価額合計</p>
          <p className="text-2xl font-semibold tabular-nums">{fmt(totalValue)}円</p>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-5">
          <p className="text-xs text-gray-500 mb-1">取得金額合計</p>
          <p className="text-2xl font-semibold tabular-nums">{fmt(totalCost)}円</p>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-5">
          <p className="text-xs text-gray-500 mb-1">評価損益</p>
          <p className="text-xl font-semibold tabular-nums">
            <PnlText value={totalPnl} rate={totalPnlRate} />
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
