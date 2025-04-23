import { useState } from 'react';

interface CSVFileInputProps {
  onFileSelect: (file: File) => void;
}

export function CSVFileInput({ onFileSelect }: CSVFileInputProps) {
  const [selectedFileName, setSelectedFileName] = useState('');

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && files.length > 0) {
      const file = files[0];
      setSelectedFileName(file.name);
      onFileSelect(file);
    }
  };

  return (
    <div className="input-group">
      <label className="input-group-btn" htmlFor="csv-file-input">
        <span className="btn bg-primary text-white">CSVファイル選択</span>
      </label>
      <input
        id="csv-file-input"
        type="file"
        accept=".csv"
        style={{ display: 'none' }}
        onChange={handleFileChange}
      />
      <input
        type="text"
        className="form-control form-control-sm"
        readOnly
        placeholder="CSVファイルを選択してください。"
        value={selectedFileName}
      />
    </div>
  );
}
