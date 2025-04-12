import { useState, useEffect, useMemo, CSSProperties } from 'react';
import { ReceiptTemplate } from '../../components/templates/ReceiptTemplate';
import { DividendItem, parseDividendItemFromCSV, calculateDividendSummary, formatDate, formatCurrency, sortByDate, searchBySecurityCode, HEADERS } from '../../data/receipt';

interface DividendListProps {
  csvData: any[];
}

const DIVIDEND_HEADER_KEYS = [
  'settlement_date',
  'product',
  'account',
  'security_code',
  'security_name',
  'currency',
  'unit_price',
  'shares',
  'dividends_before_tax',
  'taxes',
  'net_amount_received',
  'total_dividends_before_tax',
  'total_taxes',
  'total_net_amount_received'
] as const;

const DIVIDEND_HEADERS: Record<string, string> = Object.fromEntries(
  DIVIDEND_HEADER_KEYS.map(key => [key, HEADERS[key]])
);

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

  const { filteredDividends, summary, groupedByYearMonth } = useMemo(() => {
    let filtered = dividends;

    if (searchQuery) {
      if (searchQuery.startsWith('name:')) {
        const securityName = searchQuery.substring(5);
        filtered = dividends.filter(item => item.security_name === securityName);
      } else {
        filtered = searchBySecurityCode(dividends, searchQuery);
      }
    }

    const grouped = filtered.reduce((acc, item) => {
      if (!item.settlement_date) return acc;

      const date = new Date(item.settlement_date);
      const yearMonth = `${date.getFullYear()}/${String(date.getMonth() + 1).padStart(2, '0')}`;

      if (!acc[yearMonth]) {
        acc[yearMonth] = {
          items: [],
          total_dividends_before_tax: 0,
          total_taxes: 0,
          total_net_amount_received: 0
        };
      }

      acc[yearMonth].items.push(item);
      acc[yearMonth].total_dividends_before_tax += (item.dividends_before_tax || 0);
      acc[yearMonth].total_taxes += (item.taxes || 0);
      acc[yearMonth].total_net_amount_received += (item.net_amount_received || 0);

      return acc;
    }, {} as Record<string, {
      items: DividendItem[],
      total_dividends_before_tax: number,
      total_taxes: number,
      total_net_amount_received: number
    }>);

    return {
      filteredDividends: filtered,
      summary: calculateDividendSummary(filtered),
      groupedByYearMonth: grouped
    };
  }, [dividends, searchQuery]);

  const securityOptions = useMemo(() => {
    const uniqueIdentifiers = dividends
      .map(item => {
        const identifier = item.security_code || (item.security_name ? `name:${item.security_name}` : null);
        return {
          identifier,
          name: item.security_name
        };
      })
      .filter((item): item is { identifier: string; name: string | undefined } => item.identifier !== null)
      .reduce((unique: { identifier: string; name: string | undefined }[], item) => {
        if (!unique.some(u => u.identifier === item.identifier)) {
          unique.push(item);
        }
        return unique;
      }, [])
      .sort((a, b) => {
        if (a.identifier.startsWith('name:') && !b.identifier.startsWith('name:')) return 1;
        if (!a.identifier.startsWith('name:') && b.identifier.startsWith('name:')) return -1;
        return a.identifier.localeCompare(b.identifier);
      });

    return uniqueIdentifiers.map(item => {
      const isNameOnly = item.identifier.startsWith('name:');
      return {
        value: item.identifier,
        label: isNameOnly
          ? item.name || ''
          : (item.name ? `${item.identifier}: ${item.name}` : item.identifier)
      };
    });
  }, [dividends]);

  const handleSearch = (query: string) => setSearchQuery(query);
  const stickyHeaderStyle: CSSProperties = {
    position: 'sticky',
    top: 0,
    backgroundColor: 'white',
    whiteSpace: 'nowrap',
    textAlign: 'center',
    zIndex: 1,
  };

  const sortedYearMonthKeys = useMemo(() => {
    return Object.keys(groupedByYearMonth).sort((a, b) => a.localeCompare(b));
  }, [groupedByYearMonth]);

  return (
    <ReceiptTemplate
      title="配当金一覧"
      searchQuery={searchQuery}
      onSearch={handleSearch}
      searchOptions={securityOptions}
    >
      <div className="table-responsive" style={{ maxHeight: '70vh', overflowY: 'auto' }}>
        <table className="table table-bordered">
          <thead className="table-light">
            <tr>
              {Object.entries(DIVIDEND_HEADERS).map((header, index) => (
                <th key={index} scope="col" style={stickyHeaderStyle}>
                  {header[1]}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {searchQuery ? (
              <>
                {filteredDividends.map(item => (
                  <tr key={item.id}>
                    <td>{formatDate(item.settlement_date)}</td>
                    <td>{item.product}</td>
                    <td>{item.account}</td>
                    <td>{item.security_code}</td>
                    <td>{item.security_name}</td>
                    <td>{item.currency}</td>
                    <td>{item.unit_price}</td>
                    <td className="text-end">{item.shares}</td>
                    <td className="text-end">{formatCurrency(item.dividends_before_tax)}</td>
                    <td className="text-end">{formatCurrency(item.taxes)}</td>
                    <td className="text-end">{formatCurrency(item.net_amount_received)}</td>
                  </tr>
                ))}
                <tr className="table-success">
                  <td colSpan={11}></td>
                  <td className="text-end">{formatCurrency(summary.total_dividends_before_tax)}</td>
                  <td className="text-end">{formatCurrency(summary.total_taxes)}</td>
                  <td className="text-end">{formatCurrency(summary.total_net_amount_received)}</td>
                </tr>
              </>
            ) : (
              sortedYearMonthKeys.flatMap(yearMonth => [
                ...groupedByYearMonth[yearMonth].items.map(item => (
                  <tr key={item.id}>
                    <td>{formatDate(item.settlement_date)}</td>
                    <td>{item.product}</td>
                    <td>{item.account}</td>
                    <td>{item.security_code}</td>
                    <td>{item.security_name}</td>
                    <td>{item.currency}</td>
                    <td>{item.unit_price}</td>
                    <td className="text-end">{item.shares}</td>
                    <td className="text-end">{formatCurrency(item.dividends_before_tax)}</td>
                    <td className="text-end">{formatCurrency(item.taxes)}</td>
                    <td className="text-end">{formatCurrency(item.net_amount_received)}</td>
                  </tr>
                )),
                <tr key={`total-${yearMonth}`} className="table-success">
                  <td colSpan={11}></td>
                  <td className="text-end">{formatCurrency(groupedByYearMonth[yearMonth].total_dividends_before_tax)}</td>
                  <td className="text-end">{formatCurrency(groupedByYearMonth[yearMonth].total_taxes)}</td>
                  <td className="text-end">{formatCurrency(groupedByYearMonth[yearMonth].total_net_amount_received)}</td>
                </tr>
              ])
            )}
          </tbody>
        </table>
      </div>
    </ReceiptTemplate>
  );
}