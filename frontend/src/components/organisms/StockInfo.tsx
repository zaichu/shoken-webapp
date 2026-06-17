import { StockData } from '../../features/stock/types';
import { StockInfoLinks } from '../molecules/StockInfoLinks';
import { Card, CardBody } from '../atoms/Card';

interface StockInfoProps {
  stockData: StockData;
}

export function StockInfo({ stockData }: StockInfoProps) {
  if (!stockData) {
    return null;
  }

  const {
    code,
    name,
    market_category,
    industry_category_33,
    industry_category_17,
    size_category
  } = stockData;

  const metaItems = [
    { label: '市場', value: market_category },
    { label: '33業種', value: industry_category_33 },
    { label: '17業種', value: industry_category_17 },
    { label: '規模', value: size_category },
  ];

  return (
    <div>
      <Card className="mb-4 overflow-hidden">
        <div className="border-b border-slate-950/10 bg-slate-950 px-5 py-4 text-white">
          <div className="flex flex-wrap items-end justify-between gap-3">
            <div>
              <p className="text-[11px] font-black uppercase tracking-[0.22em] text-amber-300">Security</p>
              <h2 className="mt-1 text-xl font-black leading-tight">{name}</h2>
            </div>
            <span className="inline-flex rounded-md border border-white/20 bg-white px-3 py-1 font-mono text-sm font-black tracking-wider text-slate-950">
              {code}
            </span>
          </div>
        </div>
        <CardBody>
          <dl className="grid grid-cols-1 gap-px bg-slate-200 text-sm sm:grid-cols-2 lg:grid-cols-4">
            {metaItems.map(item => (
              <div key={item.label} className="bg-white px-4 py-3">
                <dt className="text-[11px] font-black uppercase tracking-[0.16em] text-slate-500">{item.label}</dt>
                <dd className="mt-1 font-bold text-dark">{item.value || '-'}</dd>
              </div>
            ))}
          </dl>
          <div className="px-4 py-4">
            <StockInfoLinks code={code} />
          </div>
        </CardBody>
      </Card>
    </div>
  );
}
