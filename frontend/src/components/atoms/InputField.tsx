import { InputHTMLAttributes, ReactNode, forwardRef, useId } from 'react';

export interface InputFieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: ReactNode;
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

    const baseInputClasses = 'block px-3 py-2 text-sm font-medium text-slate-950 bg-white border rounded-md transition-[border-color,box-shadow,background-color] placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-amber-500/25';

    const variantClasses = {
      outlined: 'border-slate-300 focus:border-amber-600',
      filled: 'border-slate-200 bg-slate-50 focus:border-amber-600 focus:bg-white',
      standard: 'border-0 border-b border-slate-300 rounded-none focus:border-amber-600',
    };

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
      <label htmlFor={inputId} className="mb-1 block text-sm font-bold text-slate-800">
        {label}
        {required ? <span className="text-danger ml-1">*</span> : null}
      </label>
    );

    const describedByIds: string[] = [];
    if (error) {
      describedByIds.push(`${inputId}-error`);
    }
    if (helpText) {
      describedByIds.push(`${inputId}-help`);
    }
    const ariaDescribedBy = describedByIds.length > 0 ? describedByIds.join(' ') : undefined;

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
          aria-describedby={ariaDescribedBy}
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
