import { memo } from 'react';
import { StockData } from '../../data/stock';
import { StockInfoLinks } from '../molecules/StockInfoLinks';

interface StockInfoProps {
  data?: StockData;
}

export const StockInfo = memo(({ data }: StockInfoProps) => {
  if (!data) return null;

  return (
    <div className="card">
      <div className="card-body">
        <table className='table'>
          <tbody>
            <tr>
              <th scope='row' style={{ width: '125px' }}>銘柄名</th>
              <td>{data.name || ""}</td>
            </tr>
            <tr>
              <th scope='row'>銘柄コード</th>
              <td>{data.code || ""}</td>
            </tr>
            <tr>
              <th scope='row'>マーケットカテゴリ</th>
              <td>{data.market_category || ""}</td>
            </tr>
            <tr>
              <th scope='row'>33業種区分</th>
              <td>{data.industry_category_33 || ""}</td>
            </tr>
            <tr>
              <th scope='row'>17業種区分</th>
              <td>{data.industry_category_17 || ""}</td>
            </tr>
            <tr>
              <th scope='row'>規模区分</th>
              <td>{data.size_category || ""}</td>
            </tr>
          </tbody>
        </table>
        <StockInfoLinks code={data.code} />
      </div>
    </div>
  );
});
