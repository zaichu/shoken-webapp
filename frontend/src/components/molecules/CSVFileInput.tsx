import { useId, useRef } from 'react';

interface CSVFileInputProps {
  onFileSelect: (file: File) => void;
  selectedFileName?: string;
  disabled?: boolean;
}

/**
 * CSVファイル選択コンポーネント
 * ファイル選択UIとファイル名表示を提供
 */
export function CSVFileInput({ onFileSelect, selectedFileName = '', disabled = false }: CSVFileInputProps) {
  const inputId = useId();
  const ref = useRef<HTMLInputElement>(null);
  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (disabled) return;
    const files = e.target.files;
    if (files && files.length > 0) {
      const file = files[0];
      onFileSelect(file);
      if (ref.current) {
        ref.current.value = ''; // ファイル選択後にinputをリセット
      }
    }
  };

  return (
    <div className="space-y-2.5">
      <label
        data-testid="csv-file-trigger"
        className={`flex min-h-20 cursor-pointer items-center justify-between gap-3 rounded-[1.35rem] border border-dashed border-slate-300 bg-slate-50 px-4 py-3 transition-colors ${
          disabled ? 'pointer-events-none opacity-65' : 'hover:border-slate-400 hover:bg-slate-100'
        }`}
        htmlFor={inputId}
      >
        <div className="flex min-w-0 items-center gap-3">
          <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-white text-slate-600 shadow-sm">
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.8} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M12 12v6m0-6l-2.5 2.5M12 12l2.5 2.5" />
            </svg>
          </span>
          <div className="min-w-0">
            <p className="text-sm font-semibold text-slate-800">CSVファイルを選択</p>
          </div>
        </div>
        <span className="inline-flex shrink-0 items-center rounded-full border border-slate-200 bg-white px-3 py-1.5 text-sm font-semibold text-slate-700 shadow-sm">
          参照
        </span>
      </label>
      <input
        id={inputId}
        data-testid="csv-file-input"
        ref={ref}
        type="file"
        accept=".csv"
        className="hidden"
        onChange={handleFileChange}
        disabled={disabled}
        aria-label="CSVファイルを選択"
      />
      <input
        type="text"
        className="w-full rounded-[1.15rem] border border-slate-200 bg-white px-3.5 py-2.5 text-sm text-slate-600 shadow-sm disabled:cursor-not-allowed disabled:bg-slate-100"
        readOnly
        placeholder="ファイル未選択"
        value={selectedFileName}
        disabled={disabled}
        aria-label="選択されたファイル名"
      />
    </div>
  );
}
