import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Layout } from './Layout';
import { fetchStockData } from '../services/api';

export const Search = () => {
  const [stockCode, setStockCode] = useState('');
  const [searchTerm, setSearchTerm] = useState('');

  const {
    data: stockData,
    error
  } = useQuery({
    queryKey: ['stock', searchTerm],
    queryFn: () => fetchStockData(searchTerm),
    enabled: !!searchTerm,
  });

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setSearchTerm(stockCode);
  };

  return (
    <Layout>
      <form onSubmit={handleSearch} className="mb-4">
        <div className="input-group">
          <input
            type="text"
            className="form-control"
            placeholder="銘柄コードを入力"
            value={stockCode}
            onChange={(e) => setStockCode(e.target.value)}
            style={{ maxWidth: '200px' }}
          />
          <button className="btn btn-primary" type="submit">検索</button>
        </div>
      </form>

      {error && (
        <div className="alert alert-danger">
          エラーが発生しました: {error instanceof Error ? error.message : 'Unknown error'}
        </div>
      )}

      <div className="card">
        <div className="card-body">
          <table className='table'>
            <tbody>
              {RenderTableRow("銘柄名", stockData?.name)}
              {RenderTableRow("銘柄コード", stockData?.code)}
              {RenderTableRow("マーケットカテゴリ", stockData?.market_category)}
              {RenderTableRow("33業種区分", stockData?.industry_category_33)}
              {RenderTableRow("17業種区分", stockData?.industry_category_17)}
              {RenderTableRow("規模区分", stockData?.size_category)}
            </tbody>
          </table>
          {RenderStockInfoLink(stockData?.code)}
        </div>
      </div>
    </Layout>
  );
}

const RenderTableRow = (label: string, value?: string) => {
  return (
    <tr>
      <th scope='row' style={{ width: '125px' }}>{label}</th>
      <td>{value || ""}</td>
    </tr>
  )
}

interface StockInfoLinkObject {
  name: string;
  url: string;
}

const STOCK_INFO_LINKS_OBJECTS: StockInfoLinkObject[] = [
  { name: "楽天証券", url: "https://www.rakuten-sec.co.jp/web/market/search/quote.html?ric={}.T" },
  { name: "SBI証券", url: "https://site3.sbisec.co.jp/ETGate/?_ControlID=WPLETsiR001Control&_DataStoreID=DSWPLETsiR001Control&_PageID=WPLETsiR001Ilst10&_ActionID=getDetailOfStockPriceJP&s_rkbn=1&i_stock_sec=%94%43%93%56%93%B0&i_dom_flg=1&i_exchange_code=JPN&i_output_type=0&stock_sec_code_mul={}" },
  { name: "株探", url: "https://kabutan.jp/stock/?code={}" },
  { name: "Yahoo! Finance", url: "https://finance.yahoo.co.jp/quote/{}" },
  { name: "日経", url: "https://www.nikkei.com/nkd/company/?scode={}" },
  { name: "バフェットコード", url: "https://www.buffett-code.com/company/{}" },
  { name: "みんかぶ", url: "https://minkabu.jp/stock/{}/" },
  { name: "IR BANK", url: "https://irbank.net/{}" },
  { name: "銘柄スカウター", url: "https://monex.ifis.co.jp/index.php?sa=report_index&bcode={}" },
  { name: "ザイマニ", url: "https://zaimani.com/search/?_sf_s={}" },
];

const RenderStockInfoLink = (code?: string) => {
  return (
    <div className='d-flex flex-wrap'>
      {STOCK_INFO_LINKS_OBJECTS.map((item, index) => (
        <React.Fragment key={index}>
          <a className='fw-bold' href={item.url.replace('{}', code || "")} target='_blank'>{item.name}</a>
          {index < STOCK_INFO_LINKS_OBJECTS.length - 1 && (<span className="mx-2">|</span>)}
        </React.Fragment>
      ))}
    </div>
  )
}