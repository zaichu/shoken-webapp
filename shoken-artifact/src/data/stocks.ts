import type { Stock } from '@/types/stock'

export const STOCKS: Stock[] = [
  { code: '7974', name: '任天堂', shares: 100, avgPrice: 7200, currentPrice: 8150 },
  { code: '9984', name: 'ソフトバンクグループ', shares: 200, avgPrice: 6800, currentPrice: 6200 },
  { code: '6758', name: 'ソニーグループ', shares: 50, avgPrice: 12500, currentPrice: 14200 },
  { code: '4755', name: '楽天グループ', shares: 500, avgPrice: 820, currentPrice: 760 },
  { code: '7203', name: 'トヨタ自動車', shares: 300, avgPrice: 2800, currentPrice: 3100 },
]
