import { CSVFileInput } from '@/components/molecules/CSVFileInput';
import { CsvSaveResultNotice } from '@/components/molecules/CsvSaveResultNotice';
import { Button } from '@/components/atoms/Button';
import { cn } from '@/lib/utils/classNames';
import type { CsvUploadResult } from '@/lib/csvImport';

export interface DataActionRailProps {
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

  /**
   * section 要素への追加クラス（任意）。
   * 未指定時は従来どおりで、既存利用箇所（取引明細）の出力は変わらない。
   */
  sectionClassName?: string;
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
  sectionClassName,
}: DataActionRailProps) {
  return (
    <section className={cn('space-y-3 bg-slate-50/60 px-5 py-5', sectionClassName)} role="group" aria-label="データ操作">
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
            className="h-11 w-full rounded-md text-sm font-bold"
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
            className="h-11 w-full rounded-md text-sm font-bold"
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
