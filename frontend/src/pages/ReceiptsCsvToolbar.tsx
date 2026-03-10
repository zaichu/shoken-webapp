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
  onFileSelect,
  onSave,
  onDeleteRequest,
}: ReceiptsCsvToolbarProps) {
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
