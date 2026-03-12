import { CSVFileInput } from '@/components/molecules/CSVFileInput';
import { CsvSaveResultNotice } from '@/components/molecules/CsvSaveResultNotice';
import { Button } from '@/components/atoms/Button';
import type { CsvUploadResult } from '@/lib/csvImport';

interface DataActionRailProps {
  // ファイル入力
  onFileSelect: (file: File) => void;
  selectedFileName?: string;
  fileInputDisabled?: boolean;

  // 保存ボタン（hasCsvFile=true のとき表示）
  hasCsvFile: boolean;
  saveLabel: string;
  onSave: () => void;
  saveDisabled?: boolean;

  // 削除ボタン（hasDbData=true のとき表示）
  hasDbData: boolean;
  deleteLabel: string;
  onDeleteRequest: () => void;
  deleteDisabled?: boolean;

  // 保存結果通知
  saveResult?: CsvUploadResult | null;
  saveModeLabel: string;
}

/**
 * utility rail 内の CSV 操作セクション（ファイル入力・保存・削除・結果通知）。
 * Receipts と AssetBalance の両ページで共用する。
 */
export function DataActionRail({
  onFileSelect,
  selectedFileName,
  fileInputDisabled,
  hasCsvFile,
  saveLabel,
  onSave,
  saveDisabled,
  hasDbData,
  deleteLabel,
  onDeleteRequest,
  deleteDisabled,
  saveResult,
  saveModeLabel,
}: DataActionRailProps) {
  return (
    <section className="space-y-3 px-5 py-5" role="group" aria-label="データ操作">
      <div className="space-y-3">
        <CSVFileInput
          onFileSelect={onFileSelect}
          selectedFileName={selectedFileName ?? ''}
          disabled={fileInputDisabled}
        />
        {hasCsvFile && (
          <Button
            variant="primary"
            size="sm"
            className="h-11 w-full rounded-xl text-sm font-semibold"
            onClick={onSave}
            disabled={saveDisabled}
            aria-disabled={saveDisabled}
          >
            {saveLabel}
          </Button>
        )}
        {hasDbData && (
          <Button
            variant="outline-danger"
            size="sm"
            className="h-11 w-full rounded-xl text-sm font-semibold"
            onClick={onDeleteRequest}
            disabled={deleteDisabled}
            aria-disabled={deleteDisabled}
          >
            {deleteLabel}
          </Button>
        )}
        {saveResult && (
          <CsvSaveResultNotice result={saveResult} modeLabel={saveModeLabel} />
        )}
      </div>
    </section>
  );
}
