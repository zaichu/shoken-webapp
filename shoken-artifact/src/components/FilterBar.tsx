import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

type Props = {
  value: string
  onChange: (v: string) => void
}

export function FilterBar({ value, onChange }: Props) {
  return (
    <div className="flex items-center gap-2">
      <Input
        placeholder="銘柄コード・銘柄名で絞り込み"
        value={value}
        onChange={e => onChange(e.target.value)}
        className="max-w-xs"
      />
      {value && (
        <Button
          variant="ghost"
          size="sm"
          type="button"
          onClick={() => onChange('')}
          className="text-gray-400 hover:text-gray-600"
        >
          クリア
        </Button>
      )}
    </div>
  )
}
