import { Alert } from '@/components/atoms/Alert';
import { type CsvPreview } from './receiptsReducer';

interface ReceiptsAlertsProps {
  dbError: string | null | undefined;
  hasCsvFile: boolean;
  previewing: boolean;
  csvPreview: CsvPreview | null | undefined;
}

export function ReceiptsAlerts({
  dbError,
  hasCsvFile,
  previewing,
  csvPreview,
}: ReceiptsAlertsProps) {
  return (
    <>
      {dbError && (
        <section className="px-5 py-4">
          <Alert variant="danger" role="alert" aria-live="assertive">
            <strong>エラー:</strong> {dbError}
          </Alert>
        </section>
      )}

      {hasCsvFile && !previewing && csvPreview && (
        <section className="px-5 py-4" role="status" aria-live="polite">
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
        </section>
      )}
    </>
  );
}
