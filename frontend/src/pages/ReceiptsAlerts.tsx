import { Alert } from '@/components/atoms/Alert';
import { type CsvPreview, type ImportResult } from './receiptsReducer';

interface ReceiptsAlertsProps {
  dbError: string | null | undefined;
  hasCsvFile: boolean;
  previewing: boolean;
  csvPreview: CsvPreview | null | undefined;
  importResult: ImportResult | null | undefined;
}

export function ReceiptsAlerts({
  dbError,
  hasCsvFile,
  previewing,
  csvPreview,
  importResult,
}: ReceiptsAlertsProps) {
  return (
    <>
      {dbError && (
        <Alert variant="danger" className="my-3" role="alert" aria-live="assertive">
          <strong>エラー:</strong> {dbError}
        </Alert>
      )}

      {hasCsvFile && !previewing && csvPreview && !importResult && (
        <div className="my-3" role="status" aria-live="polite">
          <Alert variant={csvPreview.errors.length > 0 ? 'warning' : 'info'}>
            <p>
              <strong>{csvPreview.validRows}件 追加で保存されます</strong>
              {csvPreview.errors.length > 0 && ` / ${csvPreview.errors.length}件エラー`}
              <span className="ml-2 text-xs text-secondary">（保存モード: 追加）</span>
            </p>
            {csvPreview.errors.length > 0 && (
              <ul className="mt-2 list-disc list-inside text-sm space-y-1">
                {csvPreview.errors.map((e) => (
                  <li key={e.row}>{e.row}行目: {e.message}</li>
                ))}
              </ul>
            )}
          </Alert>
        </div>
      )}

      {importResult && (() => {
        const hasErrors = importResult.errors.length > 0;
        return (
          <div className="my-3" role="status" aria-live="polite">
            <Alert variant={hasErrors ? 'warning' : 'success'}>
              <p className="flex flex-wrap items-center gap-x-2">
                <strong>{importResult.inserted}件登録</strong>
                {importResult.skipped > 0 && (
                  <span className="inline-flex items-center rounded-full border border-amber-200 bg-amber-100 px-2.5 py-0.5 text-xs font-medium text-amber-700">
                    {importResult.skipped}件スキップ（重複）
                  </span>
                )}
                {hasErrors && (
                  <span className="text-sm text-secondary">
                    {importResult.errors.length}件エラー
                  </span>
                )}
              </p>
              {hasErrors && (
                <ul className="mt-2 list-disc list-inside text-sm space-y-1">
                  {importResult.errors.map((e) => (
                    <li key={e.row}>{e.row}行目: {e.message}</li>
                  ))}
                </ul>
              )}
            </Alert>
          </div>
        );
      })()}
    </>
  );
}
