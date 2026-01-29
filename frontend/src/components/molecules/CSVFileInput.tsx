import { useRef } from "react";

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
    <div className="flex">
      <label
        className={`cursor-pointer ${disabled ? 'pointer-events-none opacity-65' : ''}`}
        htmlFor="csv-file-input"
      >
        <span className="inline-flex items-center px-3 py-1.5 bg-primary text-sm text-white rounded-l font-medium hover:bg-primary-hover transition-colors">
          CSVファイル選択
        </span>
      </label>
      <input
        id="csv-file-input"
        ref={ref}
        type="file"
        accept=".csv"
        className="hidden"
        onChange={handleFileChange}
        disabled={disabled}
      />
      <input
        type="text"
        className="flex-1 px-3 py-1.5 text-sm border border-l-0 border-gray-300 rounded-r bg-white disabled:bg-gray-100 disabled:cursor-not-allowed"
        readOnly
        placeholder="ファイル未選択"
        value={selectedFileName}
        disabled={disabled}
      />
    </div>
  );
}
