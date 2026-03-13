import { useState } from 'react'
import './App.css'
import { PortfolioSummary } from '@/components/PortfolioSummary'
import { StockTable } from '@/components/StockTable'
import { FilterBar } from '@/components/FilterBar'

export type Stock = {
  code: string
  name: string
  shares: number
  avgPrice: number
  currentPrice: number
}

const STOCKS: Stock[] = [
  { code: '7974', name: '任天堂', shares: 100, avgPrice: 7200, currentPrice: 8150 },
  { code: '9984', name: 'ソフトバンクグループ', shares: 200, avgPrice: 6800, currentPrice: 6200 },
  { code: '6758', name: 'ソニーグループ', shares: 50, avgPrice: 12500, currentPrice: 14200 },
  { code: '4755', name: '楽天グループ', shares: 500, avgPrice: 820, currentPrice: 760 },
  { code: '7203', name: 'トヨタ自動車', shares: 300, avgPrice: 2800, currentPrice: 3100 },
]

function App() {
  const [filter, setFilter] = useState('')

  const filtered = STOCKS.filter(
    s => s.name.includes(filter) || s.code.includes(filter)
  )

  const totalValue = STOCKS.reduce((sum, s) => sum + s.currentPrice * s.shares, 0)
  const totalCost = STOCKS.reduce((sum, s) => sum + s.avgPrice * s.shares, 0)
  const totalPnl = totalValue - totalCost
  const totalPnlRate = (totalPnl / totalCost) * 100

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
          stockCount={STOCKS.length}
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
