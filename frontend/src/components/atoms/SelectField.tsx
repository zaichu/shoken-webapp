import { SelectHTMLAttributes } from 'react';

interface SelectOption {
  value: string;
  label: string;
}

interface SelectFieldProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  options: SelectOption[];
  error?: string;
  fullWidth?: boolean;
  emptyOptionLabel?: string;
}

export function SelectField({
  label,
  options,
  error,
  fullWidth = false,
  className = '',
  id,
  emptyOptionLabel,
  ...rest
}: SelectFieldProps) {
  const selectId = id || `select-${Math.random().toString(36).substr(2, 9)}`;
  const baseClasses = 'form-select';
  const errorClass = error ? 'is-invalid' : '';
  const widthClass = fullWidth ? 'w-100' : '';

  const combinedClasses = [
    baseClasses,
    errorClass,
    widthClass,
    className
  ].filter(Boolean).join(' ');

  return (
    <div className={`mb-3 ${widthClass}`}>
      {label && (
        <label htmlFor={selectId} className="form-label">
          {label}
        </label>
      )}
      <select id={selectId} className={combinedClasses} {...rest}>
        {emptyOptionLabel && (
          <option value="">{emptyOptionLabel}</option>
        )}
        {options.map((option, index) => (
          <option key={index} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      {error && <div className="invalid-feedback">{error}</div>}
    </div>
  );
}
