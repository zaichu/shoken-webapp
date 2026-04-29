import { StockData } from '../../features/stock/types';
import { StockInfoLinks } from '../molecules/StockInfoLinks';
import { Card, CardHeader, CardBody } from '../atoms/Card';

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
      <Card className="mb-4">
        <CardHeader variant="primary" className="rounded-t-lg">
          <div className="flex items-center gap-3 flex-wrap">
            <span className="text-base font-semibold leading-tight">{name}</span>
            <span className="inline-block rounded bg-slate-500 px-2 py-0.5 font-mono text-sm tracking-wider">
              {code}
            </span>
          </div>
        </CardHeader>
        <CardBody>
          <dl className="grid grid-cols-2 gap-px bg-gray-200 border-b border-gray-200 text-sm">
            {metaItems.map(item => (
              <div key={item.label} className="bg-white px-3 py-2">
                <dt className="text-xs font-medium text-slate-500">{item.label}</dt>
                <dd className="mt-0.5 text-dark">{item.value || '-'}</dd>
              </div>
            ))}
          </dl>
          <div className="px-3 py-3">
            <StockInfoLinks code={code} />
          </div>
        </CardBody>
      </Card>
    </div>
  );
}
