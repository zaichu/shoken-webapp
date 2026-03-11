import { CSVFileInput } from '@/components/molecules/CSVFileInput';
import { Button } from '@/components/atoms/Button';

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
  selectedFileName,
  panelMode = false,
  onFileSelect,
  onSave,
  onDeleteRequest,
}: ReceiptsCsvToolbarProps) {
  if (panelMode) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-3 shadow-sm" role="group" aria-label="データ操作">
        <p className="mb-2.5 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-slate-500">
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          CSV操作
        </p>
        <div className="space-y-2">
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
                  className="w-full"
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
                  className="w-full"
                  onClick={onDeleteRequest}
                  disabled={saving || deleting || dbLoading}
                  aria-disabled={saving || deleting || dbLoading}
                >
                  {deleting ? '削除中...' : `全件削除 (${dbDataCount}件)`}
                </Button>
              )}
            </>
          )}
        </div>
      </div>
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
        </>
      )}
    </div>
  );
}
