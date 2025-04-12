import { useState, useEffect } from 'react';
import { ReceiptTemplate } from '../../components/templates/ReceiptTemplate';

interface FundTransaction {
  id: string;
  date: string;
  fundCode: string;
  fundName: string;
  transactionType: '買付' | '売却' | '分配金';
  amount: number;
}

interface MutualFundProps {
  csvData: any[];
}

export const MutualFund = ({ csvData }: MutualFundProps) => {
  // サンプルデータ
  const sampleTransactions: FundTransaction[] = [
    {
      id: '1',
      date: '2024-03-20',
      fundCode: '123456',
      fundName: 'グローバル株式インデックス',
      transactionType: '買付',
      amount: 50000
    },
    {
      id: '2',
      date: '2024-02-10',
      fundCode: '789012',
      fundName: '新興国債券ファンド',
      transactionType: '分配金',
      amount: 1500
    },
    {
      id: '3',
      date: '2024-01-05',
      fundCode: '345678',
      fundName: '国内リートファンド',
      transactionType: '売却',
      amount: 75000
    },
  ];

  const [transactions] = useState<FundTransaction[]>(sampleTransactions);

  // CSVデータが提供された場合は処理（実装例）
  useEffect(() => {
    if (csvData.length > 0) {
      // CSVからデータを読み込む処理を実装
      // ...
    }
  }, [csvData]);

  return (
    <ReceiptTemplate title="投資信託取引一覧">
      <div className="table-responsive">
        <table className="table table-striped">
          <thead>
            <tr>
              <th>取引日</th>
              <th>ファンドコード</th>
              <th>ファンド名</th>
              <th>取引種別</th>
              <th className="text-end">金額</th>
            </tr>
          </thead>
          <tbody>
            {transactions.map(item => (
              <tr key={item.id}>
                <td>{item.date}</td>
                <td>{item.fundCode}</td>
                <td>{item.fundName}</td>
                <td>{item.transactionType}</td>
                <td className="text-end">{item.amount.toLocaleString()}円</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </ReceiptTemplate>
  );
}
