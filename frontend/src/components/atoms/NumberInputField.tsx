import { forwardRef } from 'react';
import { InputField, InputFieldProps } from './InputField';

interface NumberInputFieldProps extends Omit<InputFieldProps, 'type' | 'value' | 'onChange'> {
  value: number | undefined;
  onChange: (value: number | undefined) => void;
  min?: number;
  max?: number;
  step?: number;
  allowDecimal?: boolean;
  allowNegative?: boolean;
  precision?: number;
}

const NumberInputField = forwardRef<HTMLInputElement, NumberInputFieldProps>(
  (
    {
      value,
      onChange,
      min,
      max,
      step = 1,
      allowDecimal = true,
      allowNegative = true,
      precision = 2,
      className = 'bg-transparent border border-gray-300',
      ...rest
    },
    ref
  ) => {
    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const inputValue = e.target.value;

      if (inputValue === '') {
        onChange(undefined);
        return;
      }

      let numberValue = parseFloat(inputValue);

      if (isNaN(numberValue)) {
        return;
      }

      if (!allowNegative && numberValue < 0) {
        numberValue = 0;
      }

      if (!allowDecimal) {
        numberValue = Math.round(numberValue);
      } else if (precision >= 0) {
        numberValue = Math.round(numberValue * Math.pow(10, precision)) / Math.pow(10, precision);
      }

      if (min !== undefined && numberValue < min) {
        numberValue = min;
      }
      if (max !== undefined && numberValue > max) {
        numberValue = max;
      }

      onChange(numberValue);
    };

    const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
      const inputValue = e.target.value;

      if (inputValue === '') {
        return;
      }

      const numberValue = parseFloat(inputValue);
      if (!isNaN(numberValue)) {
        e.target.value = allowDecimal
          ? numberValue.toFixed(precision).replace(/\.?0+$/, '')
          : numberValue.toString();
      }

      rest.onBlur?.(e);
    };

    return (
      <InputField
        ref={ref}
        type="number"
        className={className}
        value={value?.toString() ?? ''}
        onChange={handleChange}
        onBlur={handleBlur}
        min={min}
        max={max}
        step={step}
        {...rest}
      />
    );
  }
);

NumberInputField.displayName = 'NumberInputField';

export { NumberInputField };
