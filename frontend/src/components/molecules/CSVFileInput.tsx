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
    <div className="input-group">
      <label
        className={`input-group-btn ${disabled ? 'disabled' : ''}`}
        htmlFor="csv-file-input"
        style={disabled ? { pointerEvents: 'none', opacity: 0.65 } : undefined}
      >
        <span className="btn bg-primary text-white">CSVファイル選択</span>
      </label>
      <input
        id="csv-file-input"
        ref={ref}
        type="file"
        accept=".csv"
        className="csv-file-input-hidden"
        onChange={handleFileChange}
        disabled={disabled}
      />
      <input
        type="text"
        className="form-control form-control-sm"
        readOnly
        placeholder="ファイル未選択"
        value={selectedFileName}
        disabled={disabled}
      />
    </div>
  );
}
