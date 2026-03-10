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

/**
 * 数値入力専用のフィールドコンポーネント
 * InputFieldをラップして、数値入力用のプロパティを簡略化
 */
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

      // 空文字の場合はundefinedを返す
      if (inputValue === '') {
        onChange(undefined);
        return;
      }

      // 数値変換
      let numberValue = parseFloat(inputValue);

      // NaNの場合は処理しない
      if (isNaN(numberValue)) {
        return;
      }

      // 負の値の制限
      if (!allowNegative && numberValue < 0) {
        numberValue = 0;
      }

      // 小数点の制限
      if (!allowDecimal) {
        numberValue = Math.round(numberValue);
      } else if (precision >= 0) {
        numberValue = Math.round(numberValue * Math.pow(10, precision)) / Math.pow(10, precision);
      }

      // 最小値・最大値の制限
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

      // 空文字の場合は何もしない
      if (inputValue === '') {
        return;
      }

      // 数値変換してフォーマット
      const numberValue = parseFloat(inputValue);
      if (!isNaN(numberValue)) {
        // 入力フィールドの値を適切な形式で更新
        e.target.value = allowDecimal
          ? numberValue.toFixed(precision).replace(/\.?0+$/, '')
          : numberValue.toString();
      }

      // 元のonBlurイベントがあれば実行
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
