import { InputHTMLAttributes } from 'react';

interface InputFieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  fullWidth?: boolean;
  className?: string;
  id?: string;
  placeholder?: string;
}

export function InputField({
  label,
  error,
  fullWidth = false,
  className = '',
  id,
  placeholder = '',
  ...rest
}: InputFieldProps) {
  const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;
  const baseClasses = 'form-control';
  const errorClass = error ? 'is-invalid' : '';
  const widthClass = fullWidth ? 'w-100' : '';

  const combinedClasses = [
    baseClasses,
    errorClass,
    widthClass,
    className
  ].filter(Boolean).join(' ');

  return (
    <div className={`${widthClass}`}>
      {label && (
        <label htmlFor={inputId} className="form-label">
          {label}
        </label>
      )}
      <input id={inputId} className={combinedClasses} placeholder={placeholder} {...rest} />
      {error && <div className="invalid-feedback">{error}</div>}
    </div>
  );
}
