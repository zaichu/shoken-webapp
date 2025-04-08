import { useState, useEffect, useMemo } from 'react';
import { ReceiptTemplate } from '../../components/templates/ReceiptTemplate';
import { DividendItem, DividendSummary, parseDividendItemFromCSV, calculateDividendSummary, formatDate, formatCurrency, sortByDate, searchBySecurityCode } from '../../data/receipt';

interface DividendListProps {
  csvData: any[];
}

export const DividendList = ({ csvData }: DividendListProps) => {
  const [dividends, setDividends] = useState<DividendItem[]>([]);
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    if (csvData.length > 0) {
      const newDividends = csvData.map((row, index) => parseDividendItemFromCSV(row, index));
      const sortedDividends = sortByDate(newDividends);
      setDividends(sortedDividends);
    }
  }, [csvData]);

  // 検索フィルタリングとサマリー計算を最適化
  const { filteredDividends, summary } = useMemo(() => {
    let filtered = dividends;

    if (searchQuery) {
      filtered = searchBySecurityCode(dividends, searchQuery);
    }

    return {
      filteredDividends: filtered,
      summary: calculateDividendSummary(filtered)
    };
  }, [dividends, searchQuery]);

  // 銘柄コードの一意なリストを取得（メモ化）
  const securityOptions = useMemo(() => {
    const uniqueCodes = [...new Set(dividends.map(item => item.security_code).filter(Boolean))].sort();

    return uniqueCodes.map(code => {
      const item = dividends.find(d => d.security_code === code);
      return {
        value: code || '',
        label: item ? `${code}: ${item.security_name}` : code
      };
    });
  }, [dividends]);

  // 検索ハンドラー
  const handleSearch = (query: string) => {
    setSearchQuery(query);
  };

  return (
    <ReceiptTemplate
      title="配当金一覧"
      searchQuery={searchQuery}
      onSearch={handleSearch}
      searchOptions={securityOptions}
    >
      <div className="table-responsive">
        <table className="table table-striped">
          <thead className="table-light">
            <tr>
              <th>入金日(受渡日)</th>
              <th>銘柄コード</th>
              <th>銘柄名</th>
              <th className="text-end">配当・分配金（税引前）</th>
              <th className="text-end">税額</th>
              <th className="text-end">受取金額</th>
            </tr>
          </thead>
          <tbody>
            {filteredDividends.map(item => (
              <tr key={item.id}>
                <td>{formatDate(item.settlement_date)}</td>
                <td>{item.security_code}</td>
                <td>{item.security_name}</td>
                <td className="text-end">{formatCurrency(item.dividends_before_tax)}</td>
                <td className="text-end">{formatCurrency(item.taxes)}</td>
                <td className="text-end">{formatCurrency(item.net_amount_received)}</td>
              </tr>
            ))}
          </tbody>
          <tfoot className="table-success">
            <tr>
              <th colSpan={3}>合計</th>
              <th className="text-end">{formatCurrency(summary.total_dividends_before_tax)}</th>
              <th className="text-end">{formatCurrency(summary.total_taxes)}</th>
              <th className="text-end">{formatCurrency(summary.total_net_amount_received)}</th>
            </tr>
          </tfoot>
        </table>
      </div>
    </ReceiptTemplate>
  );
}
