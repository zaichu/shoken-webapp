import { useState, useEffect } from 'react';
import { ReceiptTemplate } from '../../components/templates/ReceiptTemplate';

interface StockTransaction {
  id: string;
  date: string;
  stockCode: string;
  stockName: string;
  transactionType: '買付' | '売却';
  price: number;
  quantity: number;
  amount: number;
}

interface DomesticStockProps {
  csvData: any[];
}

export const DomesticStock = ({ csvData }: DomesticStockProps) => {

  const [transactions] = useState<StockTransaction[]>([]);

  // CSVデータが提供された場合は処理（実装例）
  useEffect(() => {
    if (csvData.length > 0) {
      // CSVからデータを読み込む処理を実装
      // ...
    }
  }, [csvData]);

  return (
    <ReceiptTemplate title="国内株式取引一覧">
      <div className="table-responsive">
        <table className="table table-striped">
          <thead>
            <tr>
              <th>取引日</th>
              <th>銘柄コード</th>
              <th>銘柄名</th>
              <th>取引種別</th>
              <th className="text-end">価格</th>
              <th className="text-end">数量</th>
              <th className="text-end">金額</th>
            </tr>
          </thead>
          <tbody>
            {transactions.map(item => (
              <tr key={item.id}>
                <td>{item.date}</td>
                <td>{item.stockCode}</td>
                <td>{item.stockName}</td>
                <td>{item.transactionType}</td>
                <td className="text-end">{item.price.toLocaleString()}円</td>
                <td className="text-end">{item.quantity}株</td>
                <td className="text-end">{item.amount.toLocaleString()}円</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </ReceiptTemplate>
  );
}
