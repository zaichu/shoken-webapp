import React, { ReactNode, useId, useState } from 'react';
import { cn } from '@/lib/utils/classNames';
import { HeaderItem, KpiTone } from '@/types/common';

interface ReceiptHeaderProps {
    items: HeaderItem[];
    title?: string;
    children?: ReactNode;
    collapsible?: boolean;
    defaultExpanded?: boolean;
}

// コンポーネント外に定義（毎レンダーで新参照が生成されるのを防ぐ）
// AssetPortfolioSummary の KPI カード（border-{color}-100 bg-{color}-50 / border-slate-200 bg-white）と統一
const KPI_CARD_BG: Record<KpiTone, string> = {
    emerald: 'border-teal-200 bg-teal-50',
    red:     'border-rose-100 bg-rose-50',
    blue:    'border-blue-100 bg-blue-50',
    slate:   'border-slate-200 bg-white',
};

const TONE_VALUE_COLOR: Record<KpiTone, string> = {
    emerald: 'text-teal-700',
    red:     'text-red-500',
    blue:    'text-blue-600',
    slate:   'text-slate-800',
};

const resolveTone = (item: HeaderItem): KpiTone => item.tone ?? 'slate';

// KPI 1枚分の描画
interface KpiCardProps {
    item: HeaderItem;
    cardClassName: string;
    valueSizeClassName: string;
}

function KpiCard({ item, cardClassName, valueSizeClassName }: KpiCardProps) {
    return (
        <div className={cardClassName}>
            <p className="mb-1 text-xs font-medium text-slate-600">{item.title}</p>
            <p
                className={cn(valueSizeClassName, 'font-bold tabular-nums', TONE_VALUE_COLOR[resolveTone(item)])}
                data-negative={item.value < 0 ? 'true' : undefined}
            >
                {item.format(item.value)}
            </p>
        </div>
    );
}

// KPI グリッド描画
interface KpiGridProps {
    items: HeaderItem[];
    gridClassName: string;
    cardBg: Record<KpiTone, string>;
    cardBaseClassName: string;
    valueSizeClassName: string;
}

function KpiGrid({ items, gridClassName, cardBg, cardBaseClassName, valueSizeClassName }: KpiGridProps) {
    if (items.length === 0) return null;
    return (
        <div className={gridClassName} data-testid="kpi-grid">
            {items.map((item) => (
                <KpiCard
                    key={item.title}
                    item={item}
                    cardClassName={cn(cardBaseClassName, cardBg[resolveTone(item)])}
                    valueSizeClassName={valueSizeClassName}
                />
            ))}
        </div>
    );
}

// children セクション描画（KPI グリッドとの境界線を含む）
interface ChildrenSectionProps {
    children: ReactNode;
    hasItems: boolean;
    dividerClassName: string;
}

function ChildrenSection({ children, hasItems, dividerClassName }: ChildrenSectionProps) {
    if (!children) return null;
    return (
        <div className={hasItems ? dividerClassName : ''}>
            {children}
        </div>
    );
}

export const ReceiptHeader: React.FC<ReceiptHeaderProps> = ({
    items,
    title = '集計情報',
    children,
    collapsible = false,
    defaultExpanded = true,
}: ReceiptHeaderProps) => {
    const [isExpanded, setIsExpanded] = useState(() => (collapsible ? defaultExpanded : true));
    // スマホ幅 (<sm) の1行サマリー用。初期は折り畳み、PC幅の表示には影響しない
    const [mobileExpanded, setMobileExpanded] = useState(false);
    const bodyId = useId();
    const mobileBodyId = useId();
    const effectiveExpanded = collapsible ? isExpanded : true;

    // 折り畳み1行目に出す主要金額: 配当=受取額、国内株式/投信=実現損益税引後。
    // 各ページが主要指標を末尾に置く規約のため、末尾の項目を使う
    const primaryItem = items.length > 0 ? items[items.length - 1] : undefined;

    const handleToggleExpanded = () => {
        if (!collapsible) return;
        setIsExpanded(prev => !prev);
    };

    const chevron = collapsible && (
        <span className="flex items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2.5 py-1 text-slate-700" aria-hidden="true">
            <span className="text-xs font-semibold">{effectiveExpanded ? '閉じる' : '開く'}</span>
            <svg
                className={cn('w-4 h-4 text-slate-500 transition-transform duration-200', effectiveExpanded && 'rotate-180')}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
        </span>
    );

    return (
        <section
            className="rounded-xl border border-slate-950/10 bg-white/95 px-4 py-3 shadow-[0_12px_34px_-30px_rgba(15,23,42,0.85)] max-sm:px-3 max-sm:py-0"
            data-testid="receipt-summary-strip"
        >
            {/* スマホ幅 (<sm) のみ: 主要金額1行＋タップ展開。PC幅では hidden */}
            <div className="sm:hidden" data-testid="receipt-summary-compact">
                <button
                    type="button"
                    className="flex min-h-[40px] w-full items-center justify-between gap-3 text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-950"
                    onClick={() => setMobileExpanded(prev => !prev)}
                    aria-expanded={mobileExpanded}
                    aria-controls={mobileBodyId}
                    aria-label={primaryItem ? `${primaryItem.title} ${primaryItem.format(primaryItem.value)}` : title}
                    data-testid="receipt-summary-compact-toggle"
                >
                    {primaryItem ? (
                        <span className="flex min-w-0 items-baseline gap-2">
                            <span className="shrink-0 text-xs font-medium text-slate-600">{primaryItem.title}</span>
                            <span
                                className={cn('truncate text-base font-bold tabular-nums', TONE_VALUE_COLOR[resolveTone(primaryItem)])}
                                data-negative={primaryItem.value < 0 ? 'true' : undefined}
                            >
                                {primaryItem.format(primaryItem.value)}
                            </span>
                        </span>
                    ) : (
                        <span className="text-sm font-black text-slate-950">{title}</span>
                    )}
                    <span className="flex shrink-0 items-center gap-1 text-slate-700">
                        <span className="text-xs font-semibold">{mobileExpanded ? '閉じる' : '開く'}</span>
                        <svg
                            aria-hidden="true"
                            className={cn('h-4 w-4 text-slate-500 transition-transform duration-200', mobileExpanded && 'rotate-180')}
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                        </svg>
                    </span>
                </button>
                {mobileExpanded && (
                    <div id={mobileBodyId} role="region" className="border-t border-slate-950/10 py-3">
                        <KpiGrid
                            items={items}
                            gridClassName="grid grid-cols-1 gap-2.5"
                            cardBg={KPI_CARD_BG}
                            cardBaseClassName="rounded-lg border px-3.5 py-3"
                            valueSizeClassName="text-2xl"
                        />
                        <ChildrenSection
                            hasItems={items.length > 0}
                            dividerClassName="mt-3 border-t border-slate-950/10 pt-3"
                        >
                            {children}
                        </ChildrenSection>
                    </div>
                )}
            </div>
            {/* PC幅 (sm以上) のみ: 従来表示をそのまま維持 */}
            <div className="hidden sm:block" data-testid="receipt-summary-desktop">
            {collapsible ? (
                <button
                    type="button"
                    className="flex w-full items-start justify-between gap-3 border-b border-slate-950/10 pb-2.5 text-left"
                    onClick={handleToggleExpanded}
                    aria-expanded={effectiveExpanded}
                    aria-controls={bodyId}
                    data-testid="receipt-header"
                >
                    <div><h2 className="text-sm font-black text-slate-950">{title}</h2></div>
                    {chevron}
                </button>
            ) : (
                <div
                    className="flex items-start justify-between gap-3 border-b border-slate-950/10 pb-2.5"
                    data-testid="receipt-header"
                >
                    <div><h2 className="text-sm font-black text-slate-950">{title}</h2></div>
                </div>
            )}
            <div id={bodyId} hidden={collapsible && !effectiveExpanded} className="pt-3">
                <KpiGrid
                    items={items}
                    gridClassName="grid grid-cols-1 gap-2.5 sm:grid-cols-2 xl:grid-cols-3"
                    cardBg={KPI_CARD_BG}
                    cardBaseClassName="rounded-lg border px-3.5 py-3"
                    valueSizeClassName="text-2xl"
                />
                <ChildrenSection
                    hasItems={items.length > 0}
                    dividerClassName="mt-3 border-t border-slate-950/10 pt-3"
                >
                    {children}
                </ChildrenSection>
            </div>
            </div>
        </section>
    );
};
