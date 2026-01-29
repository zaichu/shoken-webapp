import { InputHTMLAttributes, ReactNode, forwardRef, useId } from 'react';

export interface InputFieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  fullWidth?: boolean;
  helpText?: ReactNode;
  required?: boolean;
  variant?: 'outlined' | 'filled' | 'standard';
}

const InputField = forwardRef<HTMLInputElement, InputFieldProps>(
  (
    {
      label,
      error,
      fullWidth = false,
      helpText,
      required = false,
      variant = 'outlined',
      className = '',
      id,
      placeholder = '',
      ...rest
    },
    ref
  ) => {
    const generatedId = useId();
    const inputId = id || generatedId;

    // ベーススタイル
    const baseInputClasses = 'block px-3 py-1.5 text-base text-dark bg-white border rounded transition-colors focus:outline-none focus:ring-2 focus:ring-primary/25';

    // バリアント別スタイル
    const variantClasses = {
      outlined: 'border-gray-300 focus:border-primary',
      filled: 'border-gray-300 bg-gray-100 focus:border-primary focus:bg-white',
      standard: 'border-0 border-b border-gray-300 rounded-none focus:border-primary',
    };

    // エラー時のスタイル
    const errorClasses = error
      ? 'border-danger focus:border-danger focus:ring-danger/25'
      : '';

    const widthClass = fullWidth ? 'w-full' : '';

    const combinedInputClasses = [
      baseInputClasses,
      variantClasses[variant],
      errorClasses,
      widthClass,
      className
    ].filter(Boolean).join(' ');

    const labelElement = label && (
      <label htmlFor={inputId} className="block mb-1 text-sm font-medium text-dark">
        {label}
        {required && <span className="text-danger ml-1">*</span>}
      </label>
    );

    return (
      <div className={widthClass}>
        {labelElement}
        <input
          ref={ref}
          id={inputId}
          className={combinedInputClasses}
          placeholder={placeholder}
          required={required}
          aria-invalid={error ? 'true' : 'false'}
          aria-describedby={
            [
              error ? `${inputId}-error` : '',
              helpText ? `${inputId}-help` : ''
            ].filter(Boolean).join(' ') || undefined
          }
          {...rest}
        />
        {error && (
          <div id={`${inputId}-error`} className="mt-1 text-sm text-danger" role="alert">
            {error}
          </div>
        )}
        {helpText && (
          <div id={`${inputId}-help`} className="mt-1 text-sm text-secondary">
            {helpText}
          </div>
        )}
      </div>
    );
  }
);

InputField.displayName = 'InputField';

export { InputField };
