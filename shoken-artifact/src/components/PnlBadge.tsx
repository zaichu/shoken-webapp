import { Badge } from '@/components/ui/badge'
import { fmt } from '@/lib/format'

type Props = {
  value: number
  rate: number
  variant?: 'text' | 'badge'
}

export function PnlBadge({ value, rate, variant = 'badge' }: Props) {
  const positive = value >= 0
  const sign = positive ? '+' : ''
  const label = `${sign}${fmt(value)}円 (${sign}${rate.toFixed(2)}%)`

  if (variant === 'text') {
    const color = positive ? 'text-emerald-600' : 'text-red-600'
    return <span className={color}>{label}</span>
  }

  return (
    <Badge variant={positive ? 'default' : 'destructive'} className="tabular-nums font-normal">
      {label}
    </Badge>
  )
}
