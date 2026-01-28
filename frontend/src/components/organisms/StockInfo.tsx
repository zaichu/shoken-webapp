import { StockData } from '../../features/stock/types';
import { StockInfoLinks } from '../molecules/StockInfoLinks';

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
    <div className="stock-info">
      <div className="card mb-4">
        <div className="card-header bg-primary text-white">
          <h5 className="mb-0">{name} ({code})</h5>
        </div>
        <div className="card-body">
          <table className="table table-bordered mb-0">
            <tbody>
              {items.map(item => (
                <tr key={item.label}>
                  <th className="stock-info-label">{item.label}</th>
                  <td>{item.value}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <div className="mt-3">
            <StockInfoLinks code={code} />
          </div>
        </div>
      </div>
    </div>
  );
}
