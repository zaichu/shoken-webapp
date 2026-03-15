import { PnlBadge } from '@/components/PnlBadge'
import { fmt } from '@/lib/format'
import type { Stock } from '@/types/stock'

type Props = {
  stocks: Stock[]
}

export function StockTable({ stocks }: Props) {
  if (stocks.length === 0) {
    return (
      <div className="text-center text-gray-400 py-10 border rounded-md bg-white">
        該当する銘柄がありません
      </div>
    )
  }

  return (
    <div className="border rounded-md bg-white overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b bg-gray-50 text-gray-500 text-left">
            <th className="px-4 py-2.5 font-medium">銘柄</th>
            <th className="px-4 py-2.5 font-medium text-right">保有数</th>
            <th className="px-4 py-2.5 font-medium text-right">平均取得単価</th>
            <th className="px-4 py-2.5 font-medium text-right">現在値</th>
            <th className="px-4 py-2.5 font-medium text-right">評価額</th>
            <th className="px-4 py-2.5 font-medium text-right">評価損益</th>
          </tr>
        </thead>
        <tbody>
          {stocks.map(s => {
            const value = s.currentPrice * s.shares
            const cost = s.avgPrice * s.shares
            const pnl = value - cost
            const rate = cost > 0 ? (pnl / cost) * 100 : 0
            return (
              <tr key={s.code} className="border-b last:border-0 hover:bg-gray-50">
                <td className="px-4 py-3">
                  <span className="font-medium text-gray-900">{s.name}</span>
                  <span className="ml-2 text-gray-400 text-xs">{s.code}</span>
                </td>
                <td className="px-4 py-3 text-right tabular-nums">{fmt(s.shares)}株</td>
                <td className="px-4 py-3 text-right tabular-nums">{fmt(s.avgPrice)}円</td>
                <td className="px-4 py-3 text-right tabular-nums">{fmt(s.currentPrice)}円</td>
                <td className="px-4 py-3 text-right tabular-nums font-medium">{fmt(value)}円</td>
                <td className="px-4 py-3 text-right">
                  <PnlBadge value={pnl} rate={rate} />
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}
