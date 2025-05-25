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

    const baseClasses = 'form-control';
    const errorClass = error ? 'is-invalid' : '';
    const widthClass = fullWidth ? 'w-100' : '';
    const variantClass = variant !== 'outlined' ? `form-control-${variant}` : '';

    const combinedClasses = [
      baseClasses,
      variantClass,
      errorClass,
      widthClass,
      className
    ].filter(Boolean).join(' ');

    const labelElement = label && (
      <label htmlFor={inputId} className="form-label">
        {label}
        {required && <span className="text-danger ms-1">*</span>}
      </label>
    );

    return (
      <div className={widthClass}>
        {labelElement}
        <input
          ref={ref}
          id={inputId}
          className={combinedClasses}
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
          <div id={`${inputId}-error`} className="invalid-feedback" role="alert">
            {error}
          </div>
        )}
        {helpText && (
          <div id={`${inputId}-help`} className="form-text text-muted">
            {helpText}
          </div>
        )}
      </div>
    );
  }
);

InputField.displayName = 'InputField';

export { InputField };
