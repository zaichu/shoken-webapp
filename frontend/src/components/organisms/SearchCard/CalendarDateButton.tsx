import React, { useRef } from 'react';
import { cn } from '@/lib/utils/classNames';
import { formatDateLabel } from './searchQueryUtils';

interface CalendarDateButtonProps {
    label: string;
    value: string;
    inputType?: 'date' | 'month';
    onChange: (value: string) => void;
}

export const CalendarDateButton: React.FC<CalendarDateButtonProps> = ({ label, value, inputType = 'date', onChange }) => {
    const inputRef = useRef<HTMLInputElement>(null);

    const handleButtonClick = () => {
        const input = inputRef.current;
        if (!input) return;
        try {
            const pickerInput = input as HTMLInputElement & { showPicker?: () => void };
            if (typeof pickerInput.showPicker === 'function') {
                pickerInput.showPicker();
                return;
            }
        } catch {
            // showPicker が失敗した場合はクリックフォールバックへ
        }
        input.focus();
        input.click();
    };

    return (
        <div className="relative">
            <button
                type="button"
                onClick={handleButtonClick}
                className={cn(
                    'w-full flex items-center gap-2 rounded-md border px-3 py-2 text-sm transition-colors focus:outline-none focus:ring-2 focus:border-amber-600 focus:ring-amber-500/25',
                    value
                        ? 'border-amber-500 bg-amber-50 text-amber-900 font-semibold'
                        : 'border-slate-300 bg-white text-slate-500 hover:border-slate-400'
                )}
            >
                <svg className="w-3.5 h-3.5 shrink-0 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                <span className="flex-1 text-left">
                    {value ? formatDateLabel(value) : label}
                </span>
            </button>
            <input
                ref={inputRef}
                type={inputType}
                value={value}
                onChange={e => onChange(e.target.value)}
                aria-hidden="true"
                tabIndex={-1}
                className="absolute opacity-0 pointer-events-none w-px h-px overflow-hidden"
            />
        </div>
    );
};
