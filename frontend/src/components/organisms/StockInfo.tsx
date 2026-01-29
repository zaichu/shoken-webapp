import { StockData } from '../../features/stock/types';
import { StockInfoLinks } from '../molecules/StockInfoLinks';
import { Card, CardBody, CardHeader } from '../atoms/Card';

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

  const items = [
    { label: '銘柄コード', value: code },
    { label: '銘柄名', value: name },
    { label: '市場', value: market_category },
    { label: '33業種', value: industry_category_33 },
    { label: '17業種', value: industry_category_17 },
    { label: '規模', value: size_category }
  ];

  return (
    <div>
      <Card className="mb-4">
        <CardHeader variant="primary">
          <h5 className="text-sm font-semibold">{name} ({code})</h5>
        </CardHeader>
        <CardBody>
          <table className="w-full border-collapse text-sm">
            <tbody>
              {items.map(item => (
                <tr key={item.label} className="border-b border-gray-200 last:border-b-0">
                  <th className="w-[10%] bg-gray-50 px-3 py-2 text-left font-medium text-dark">{item.label}</th>
                  <td className="px-3 py-2">{item.value || '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <div className="mt-3">
            <StockInfoLinks code={code} />
          </div>
        </CardBody>
      </Card>
    </div>
  );
}
