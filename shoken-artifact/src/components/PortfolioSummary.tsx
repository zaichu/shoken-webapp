import { Card, CardContent } from '@/components/ui/card'
import { PnlBadge } from '@/components/PnlBadge'
import { fmt } from '@/lib/format'

type Props = {
  totalValue: number
  totalCost: number
  totalPnl: number
  totalPnlRate: number
  stockCount: number
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
            <PnlBadge value={totalPnl} rate={totalPnlRate} variant="text" />
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
