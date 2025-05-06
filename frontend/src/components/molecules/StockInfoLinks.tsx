import { memo, Fragment } from 'react';

interface StockInfoLinkObject {
  name: string;
  url: string;
}

interface StockInfoLinksProps {
  code?: string;
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
  { name: "ザイマニ", url: "https://zaimani.com/search/?_sf_s={}" }
];

export const StockInfoLinks = memo(({ code }: StockInfoLinksProps) => {
  if (!code) return null;

  return (
    <div className='d-flex flex-wrap'>
      {STOCK_INFO_LINKS_OBJECTS.map((item, index) => (
        <Fragment key={index}>
          <a
            className='fw-bold'
            href={item.url.replace('{}', code)}
            target='_blank'
            rel="noopener noreferrer"
          >
            {item.name}
          </a>
          {index < STOCK_INFO_LINKS_OBJECTS.length - 1 && (<span className="mx-2">|</span>)}
        </Fragment>
      ))}
    </div>
  );
});
