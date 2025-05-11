import { InputField } from './InputField';

interface NumberInputFieldProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
  placeholder?: string;
  id?: string;
  error?: string;
  className?: string;
  disabled?: boolean;
  helpText?: string | React.ReactNode;
}

/**
 * 数値入力専用のフィールドコンポーネント
 * InputFieldをラップして、数値入力用のプロパティを簡略化
 */
export function NumberInputField({
  label,
  value,
  onChange,
  placeholder,
  id,
  error,
  className = 'form-control-plaintext border',
  disabled,
  helpText,
}: NumberInputFieldProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const numberValue = Number(e.target.value);
    onChange(numberValue);
  };

  return (
    <InputField
      label={label}
      type="number"
      className={className}
      value={value}
      onChange={handleChange}
      placeholder={placeholder}
      id={id}
      error={error}
      disabled={disabled}
      helpText={helpText}
    />
  );
}
