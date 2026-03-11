import { CSVFileInput } from '@/components/molecules/CSVFileInput';
import { CsvSaveResultNotice } from '@/components/molecules/CsvSaveResultNotice';
import { Button } from '@/components/atoms/Button';
import type { CsvUploadResult } from '@/lib/csvImport';

interface ReceiptsCsvToolbarProps {
  isAuthenticated: boolean;
  hasCsvFile: boolean;
  hasDbData: boolean;
  dbDataCount: number;
  saving: boolean;
  deleting: boolean;
  previewing: boolean;
  dbLoading: boolean;
  authLoading: boolean;
  saveLabel: string;
  saveResult?: CsvUploadResult | null;
  selectedFileName?: string;
  /** パネルモード: 縦並びボタンのサイドパネル表示 */
  panelMode?: boolean;
  onFileSelect: (file: File) => void;
  onSave: () => void;
  onDeleteRequest: () => void;
}

export function ReceiptsCsvToolbar({
  isAuthenticated,
  hasCsvFile,
  hasDbData,
  dbDataCount,
  saving,
  deleting,
  previewing,
  dbLoading,
  authLoading,
  saveLabel,
  saveResult,
  selectedFileName,
  panelMode = false,
  onFileSelect,
  onSave,
  onDeleteRequest,
}: ReceiptsCsvToolbarProps) {
  if (panelMode) {
    return (
      <section className="space-y-3 px-5 py-5" role="group" aria-label="データ操作">
        <div className="space-y-3">
          <CSVFileInput
            onFileSelect={onFileSelect}
            selectedFileName={selectedFileName}
            disabled={dbLoading || saving || deleting || previewing || authLoading}
          />
          {isAuthenticated && (
            <>
              {hasCsvFile && (
                <Button
                  variant="primary"
                  size="sm"
                  className="h-11 w-full rounded-xl text-sm font-semibold"
                  onClick={onSave}
                  disabled={saving || deleting || previewing}
                  aria-disabled={saving || deleting || previewing}
                >
                  {saving ? '保存中...' : previewing ? '解析中...' : saveLabel}
                </Button>
              )}
              {hasDbData && (
                <Button
                  variant="outline-danger"
                  size="sm"
                  className="h-11 w-full rounded-xl text-sm font-semibold"
                  onClick={onDeleteRequest}
                  disabled={saving || deleting || dbLoading}
                  aria-disabled={saving || deleting || dbLoading}
                >
                  {deleting ? '削除中...' : `全件削除 (${dbDataCount}件)`}
                </Button>
              )}
            </>
          )}
          {saveResult && (
            <CsvSaveResultNotice
              result={saveResult}
              modeLabel="追加保存"
            />
          )}
        </div>
      </section>
    );
  }

  return (
    <div className="action-toolbar">
      <div className="form-input-container">
        <CSVFileInput
          onFileSelect={onFileSelect}
          selectedFileName={selectedFileName}
          disabled={dbLoading || saving || deleting || previewing || authLoading}
        />
      </div>
      {isAuthenticated && (
        <>
          <div className="action-button-group" role="group" aria-label="データ操作">
            {hasCsvFile && (
              <Button
                variant="primary"
                size="sm"
                onClick={onSave}
                disabled={saving || deleting || previewing}
                aria-disabled={saving || deleting || previewing}
              >
                {saving ? '保存中...' : previewing ? '解析中...' : saveLabel}
              </Button>
            )}
          </div>
          {hasDbData && (
            <div className="ml-auto border-l border-slate-300 pl-3">
              <Button
                variant="outline-danger"
                size="sm"
                onClick={onDeleteRequest}
                disabled={saving || deleting || dbLoading}
                aria-disabled={saving || deleting || dbLoading}
              >
                {deleting ? '削除中...' : `全件削除 (${dbDataCount}件)`}
              </Button>
            </div>
          )}
          {saveResult && (
            <div className="w-full pt-2">
              <CsvSaveResultNotice
                result={saveResult}
                modeLabel="追加保存"
              />
            </div>
          )}
        </>
      )}
    </div>
  );
}
