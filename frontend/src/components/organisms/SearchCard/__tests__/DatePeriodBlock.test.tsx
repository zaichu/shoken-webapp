import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import type { ComponentProps } from 'react';
import { beforeEach, describe, expect, test, vi } from 'vitest';
import { DatePeriodBlock } from '../DatePeriodBlock';
import type { DateInputs } from '../types';

const years = [
  { value: '2024', label: '2024年' },
  { value: '2025', label: '2025年' },
  { value: '2026', label: '2026年' },
];

const emptyDateInputs: DateInputs = {
  yearValue: '',
  monthValue: '',
  dateValue: '',
  rangeStart: '',
  rangeEnd: '',
};

function renderDatePeriodBlock(overrides: Partial<ComponentProps<typeof DatePeriodBlock>> = {}) {
  const props: ComponentProps<typeof DatePeriodBlock> = {
    dateSegment: '年',
    years,
    dateInputs: emptyDateInputs,
    isYearPickerOpen: true,
    onSegmentChange: vi.fn(),
    onToggleYearPicker: vi.fn(),
    onYearOptionSelect: vi.fn(),
    onMonthChange: vi.fn(),
    onDateValueChange: vi.fn(),
    onRangeStartChange: vi.fn(),
    onRangeEndChange: vi.fn(),
    onClose: vi.fn(),
    ...overrides,
  };

  render(<DatePeriodBlock {...props} />);
  return props;
}

describe('DatePeriodBlock', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  test('トリガー上の矢印キーで年候補へフォーカスを移動する', () => {
    renderDatePeriodBlock();

    const trigger = screen.getByRole('button', { name: '年を選択' });
    const options = screen.getAllByRole('option');

    trigger.focus();
    fireEvent.keyDown(trigger, { key: 'ArrowDown' });
    expect(options[0]).toHaveFocus();

    trigger.focus();
    fireEvent.keyDown(trigger, { key: 'ArrowUp' });
    expect(options[options.length - 1]).toHaveFocus();
  });

  test('年候補リストの矢印キーとHome/Endでフォーカスを移動する', () => {
    renderDatePeriodBlock();

    const listbox = screen.getByRole('listbox', { name: '年候補' });
    const options = screen.getAllByRole('option');

    options[0].focus();
    fireEvent.keyDown(listbox, { key: 'ArrowDown' });
    expect(options[1]).toHaveFocus();

    fireEvent.keyDown(listbox, { key: 'End' });
    expect(options[2]).toHaveFocus();

    fireEvent.keyDown(listbox, { key: 'ArrowDown' });
    expect(options[0]).toHaveFocus();

    fireEvent.keyDown(listbox, { key: 'Home' });
    expect(options[0]).toHaveFocus();

    fireEvent.keyDown(listbox, { key: 'ArrowUp' });
    expect(options[2]).toHaveFocus();
  });

  test('EnterとSpaceでフォーカス中の年候補を選択する', () => {
    const onYearOptionSelect = vi.fn();
    renderDatePeriodBlock({ onYearOptionSelect });

    const listbox = screen.getByRole('listbox', { name: '年候補' });
    const options = screen.getAllByRole('option');

    options[1].focus();
    fireEvent.keyDown(listbox, { key: 'Enter' });
    expect(onYearOptionSelect).toHaveBeenLastCalledWith('2025');

    options[2].focus();
    fireEvent.keyDown(listbox, { key: ' ' });
    expect(onYearOptionSelect).toHaveBeenLastCalledWith('2026');
  });

  test('Escapeで年候補を閉じてトリガーへフォーカスを戻す', () => {
    const onClose = vi.fn();
    renderDatePeriodBlock({ onClose });

    const trigger = screen.getByRole('button', { name: '年を選択' });
    const listbox = screen.getByRole('listbox', { name: '年候補' });
    const options = screen.getAllByRole('option');

    options[1].focus();
    fireEvent.keyDown(listbox, { key: 'Escape' });

    expect(onClose).toHaveBeenCalledTimes(1);
    expect(trigger).toHaveFocus();
  });
});
