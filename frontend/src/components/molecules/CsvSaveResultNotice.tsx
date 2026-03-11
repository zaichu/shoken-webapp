import { cn } from '@/lib/utils/classNames';
import type { CsvUploadResult } from '@/lib/csvImport';

interface CsvSaveResultNoticeProps {
  result: CsvUploadResult;
  modeLabel: string;
  className?: string;
}

export function CsvSaveResultNotice({ result, modeLabel, className }: CsvSaveResultNoticeProps) {
  const hasErrors = result.errors.length > 0;

  return (
    <section
      className={cn(
        'rounded-[1.35rem] border px-4 py-3.5',
        hasErrors
          ? 'border-amber-200/80 bg-amber-50/80'
          : 'border-emerald-200/80 bg-emerald-50/80',
        className,
      )}
      data-testid="csv-save-result-notice"
      role="status"
      aria-live="polite"
    >
      <div className="flex items-start gap-3">
        <span
          className={cn(
            'mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-white',
            hasErrors ? 'border-amber-200 text-amber-600' : 'border-emerald-200 text-emerald-600',
          )}
          aria-hidden="true"
        >
          <svg className="h-4.5 w-4.5" viewBox="0 0 20 20" fill="none" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.8" d="M5.5 10.5l3 3 6-7" />
          </svg>
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold text-slate-800">保存しました</p>
          <div className="mt-2 flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs font-semibold text-slate-700">
              {result.inserted}件反映
            </span>
            <span className="inline-flex items-center rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs font-medium text-slate-600">
              {modeLabel}
            </span>
            {result.skipped > 0 && (
              <span className="inline-flex items-center rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs font-medium text-slate-600">
                {result.skipped}件スキップ
              </span>
            )}
            {hasErrors && (
              <span className="inline-flex items-center rounded-full border border-amber-200 bg-white px-2.5 py-1 text-xs font-medium text-amber-700">
                {result.errors.length}件エラー
              </span>
            )}
          </div>

          {hasErrors && (
            <details className="mt-3 rounded-[1rem] border border-amber-200/80 bg-white/80 px-3 py-2">
              <summary className="cursor-pointer text-sm font-medium text-slate-700">
                エラー詳細を表示
              </summary>
              <ul className="mt-2 space-y-1 text-sm text-slate-600">
                {result.errors.map((error) => (
                  <li key={`${error.row}-${error.message}`}>
                    {error.row}行目: {error.message}
                  </li>
                ))}
              </ul>
            </details>
          )}
        </div>
      </div>
    </section>
  );
}
