import React, { useRef } from 'react';
import { cn } from '@/lib/utils/classNames';
import { DateInputs, DateSegment } from './types';
import { CalendarDateButton } from './CalendarDateButton';

const DATE_SEGMENTS: DateSegment[] = ['年', '月', '日', '範囲'];

interface DatePeriodBlockProps {
    dateSegment: DateSegment;
    years: { value: string; label: string }[];
    dateInputs: DateInputs;
    isYearPickerOpen: boolean;
    onSegmentChange: (segment: DateSegment) => void;
    onToggleYearPicker: () => void;
    onYearOptionSelect: (value: string) => void;
    onMonthChange: (value: string) => void;
    onDateValueChange: (value: string) => void;
    onRangeStartChange: (value: string) => void;
    onRangeEndChange: (value: string) => void;
    onClose?: () => void;
}

export const DatePeriodBlock: React.FC<DatePeriodBlockProps> = ({
    dateSegment, years, dateInputs, isYearPickerOpen,
    onSegmentChange, onToggleYearPicker, onYearOptionSelect, onMonthChange, onDateValueChange, onRangeStartChange, onRangeEndChange, onClose,
}) => {
    const { yearValue, monthValue, dateValue, rangeStart, rangeEnd } = dateInputs;
    const availableSegments = years.length > 0 ? DATE_SEGMENTS : DATE_SEGMENTS.filter(seg => seg !== '年');
    const visibleDateSegment = years.length === 0 && dateSegment === '年' ? '月' : dateSegment;

    const triggerRef = useRef<HTMLButtonElement>(null);
    const listboxRef = useRef<HTMLDivElement>(null);

    const getListboxOptions = () => {
        if (!listboxRef.current) return [];
        return Array.from(listboxRef.current.querySelectorAll<HTMLButtonElement>('[role="option"]'));
    };

    const handleListboxKeyDown = (e: React.KeyboardEvent) => {
        const options = getListboxOptions();
        if (options.length === 0) return;
        const currentIndex = options.indexOf(document.activeElement as HTMLButtonElement);
        switch (e.key) {
            case 'ArrowDown':
            case 'ArrowRight': {
                e.preventDefault();
                const next = currentIndex >= 0 && currentIndex < options.length - 1 ? options[currentIndex + 1] : options[0];
                next.focus();
                break;
            }
            case 'ArrowUp':
            case 'ArrowLeft': {
                e.preventDefault();
                const prev = currentIndex > 0 ? options[currentIndex - 1] : options[options.length - 1];
                prev.focus();
                break;
            }
            case 'Home':
                e.preventDefault();
                options[0].focus();
                break;
            case 'End':
                e.preventDefault();
                options[options.length - 1].focus();
                break;
            case 'Enter':
            case ' ': {
                e.preventDefault();
                if (currentIndex >= 0) options[currentIndex].click();
                break;
            }
            case 'Escape':
                e.preventDefault();
                onClose?.();
                triggerRef.current?.focus();
                break;
        }
    };

    const handleTriggerKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
        if (!isYearPickerOpen) return;
        const options = getListboxOptions();
        if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
            e.preventDefault();
            options[0]?.focus();
        } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
            e.preventDefault();
            options[options.length - 1]?.focus();
        } else if (e.key === 'Escape') {
            e.preventDefault();
            onClose?.();
        }
    };

    return (
    <div className="space-y-2">
        <div className="text-sm font-bold text-slate-800">期間</div>
        <div className="flex gap-1">
            {availableSegments.map(seg => (
                <button
                    key={seg}
                    type="button"
                    aria-pressed={visibleDateSegment === seg}
                    onClick={() => onSegmentChange(seg)}
                    className={cn(
                        'flex-1 rounded px-2 py-1 text-xs font-semibold transition-colors',
                        visibleDateSegment === seg
                            ? 'bg-slate-950 text-white'
                            : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                    )}
                >
                    {seg}
                </button>
            ))}
        </div>
        {visibleDateSegment === '年' && (
            <div className="relative">
                <button
                    ref={triggerRef}
                    type="button"
                    aria-label="年を選択"
                    aria-haspopup="listbox"
                    aria-expanded={isYearPickerOpen}
                    onClick={onToggleYearPicker}
                    onKeyDown={handleTriggerKeyDown}
                    className={cn(
                        'w-full flex items-center gap-2 border px-3 py-2 text-sm transition-colors focus:outline-none focus:ring-2 focus:border-amber-600 focus:ring-amber-500/25',
                        isYearPickerOpen ? 'rounded-t-md rounded-b-none' : 'rounded-md',
                        yearValue
                            ? 'border-amber-500 bg-amber-50 text-amber-900 font-semibold'
                            : 'border-slate-300 bg-white text-slate-500 hover:border-slate-400'
                    )}
                >
                    <svg className="w-3.5 h-3.5 shrink-0 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                    </svg>
                    <span className="flex-1 text-left">
                        {yearValue ? (years.find(y => y.value === yearValue)?.label ?? yearValue) : '年を選択'}
                    </span>
                    <svg
                        className={cn('w-4 h-4 shrink-0 text-slate-400 transition-transform duration-150', isYearPickerOpen && 'rotate-180')}
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                    >
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                    </svg>
                </button>
                {isYearPickerOpen && (
                    <div
                        ref={listboxRef}
                        role="listbox"
                        aria-label="年候補"
                        onKeyDown={handleListboxKeyDown}
                        className="absolute z-10 w-full grid grid-cols-3 gap-1 rounded-b-md border border-t-0 border-slate-300 bg-white px-2 pb-2 pt-1.5"
                    >
                        {years.map(year => (
                            <button
                                key={year.value}
                                type="button"
                                role="option"
                                aria-selected={yearValue === year.value}
                                onClick={() => onYearOptionSelect(year.value)}
                                className={
                                    yearValue === year.value
                                        ? 'rounded px-1 py-1.5 text-sm font-semibold text-center whitespace-nowrap bg-amber-50 text-amber-900 ring-1 ring-inset ring-amber-400 transition-colors'
                                        : 'rounded px-1 py-1.5 text-sm text-center whitespace-nowrap text-slate-600 hover:bg-slate-100 hover:text-slate-900 transition-colors'
                                }
                            >
                                {year.label}
                            </button>
                        ))}
                    </div>
                )}
            </div>
        )}
        {visibleDateSegment === '月' && (
            <CalendarDateButton
                label="月を選択"
                value={monthValue}
                inputType="month"
                onChange={onMonthChange}
            />
        )}
        {visibleDateSegment === '日' && (
            <CalendarDateButton
                label="日を選択"
                value={dateValue}
                onChange={onDateValueChange}
            />
        )}
        {visibleDateSegment === '範囲' && (
            <div className="flex flex-col gap-2">
                <CalendarDateButton
                    label="開始日"
                    value={rangeStart}
                    onChange={onRangeStartChange}
                />
                <CalendarDateButton
                    label="終了日"
                    value={rangeEnd}
                    onChange={onRangeEndChange}
                />
            </div>
        )}
    </div>
    );
};
