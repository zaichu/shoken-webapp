import { useMemo, useState } from 'react'
import './App.css'
import { STOCKS } from '@/data/stocks'
import { PortfolioSummary } from '@/components/PortfolioSummary'
import { StockTable } from '@/components/StockTable'
import { FilterBar } from '@/components/FilterBar'

function App() {
  const [filter, setFilter] = useState('')

  const filtered = useMemo(
    () => STOCKS.filter(s => s.name.includes(filter) || s.code.includes(filter)),
    [filter]
  )

  const { totalValue, totalCost, totalPnl, totalPnlRate } = useMemo(() => {
    const totalValue = filtered.reduce((sum, s) => sum + s.currentPrice * s.shares, 0)
    const totalCost = filtered.reduce((sum, s) => sum + s.avgPrice * s.shares, 0)
    const totalPnl = totalValue - totalCost
    const totalPnlRate = totalCost > 0 ? (totalPnl / totalCost) * 100 : 0
    return { totalValue, totalCost, totalPnl, totalPnlRate }
  }, [filtered])

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b px-6 py-4">
        <h1 className="text-xl font-semibold text-gray-900">保有銘柄サマリー</h1>
        <p className="text-sm text-gray-500 mt-0.5">楽天証券 ポートフォリオ概要</p>
      </header>
      <main className="px-6 py-6 max-w-4xl mx-auto space-y-6">
        <PortfolioSummary
          totalValue={totalValue}
          totalCost={totalCost}
          totalPnl={totalPnl}
          totalPnlRate={totalPnlRate}
          stockCount={filtered.length}
        />
        <div className="space-y-3">
          <FilterBar value={filter} onChange={setFilter} />
          <StockTable stocks={filtered} />
        </div>
      </main>
    </div>
  )
}

export default App
