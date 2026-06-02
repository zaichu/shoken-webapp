
interface StockInfoLinkConfig {
  name: string;
  href: (code: string) => string;
}

interface StockInfoLinksProps {
  code?: string;
}

const STOCK_INFO_LINKS: StockInfoLinkConfig[] = [
  { name: "楽天証券", href: (code) => `https://www.rakuten-sec.co.jp/web/market/search/quote.html?ric=${code}.T` },
  { name: "SBI証券", href: (code) => `https://site3.sbisec.co.jp/ETGate/?_ControlID=WPLETsiR001Control&_DataStoreID=DSWPLETsiR001Control&_PageID=WPLETsiR001Ilst10&_ActionID=getDetailOfStockPriceJP&s_rkbn=1&i_stock_sec=%94%43%93%56%93%B0&i_dom_flg=1&i_exchange_code=JPN&i_output_type=0&stock_sec_code_mul=${code}` },
  { name: "株探", href: (code) => `https://kabutan.jp/stock/?code=${code}` },
  { name: "Yahoo! Finance", href: (code) => `https://finance.yahoo.co.jp/quote/${code}` },
  { name: "日経", href: (code) => `https://www.nikkei.com/nkd/company/?scode=${code}` },
  { name: "バフェットコード", href: (code) => `https://www.buffett-code.com/company/${code}` },
  { name: "みんかぶ", href: (code) => `https://minkabu.jp/stock/${code}/` },
  { name: "IR BANK", href: (code) => `https://irbank.net/${code}` },
  { name: "銘柄スカウター", href: (code) => `https://monex.ifis.co.jp/index.php?sa=report_index&bcode=${code}` },
  { name: "ザイマニ", href: (code) => `https://zaimani.com/search/?_sf_s=${code}` },
  { name: "JPX Explorer", href: (code) => `https://jpx-explorer.com/ja-JP/${code}-TSE` },
];

export const StockInfoLinks = ({ code }: StockInfoLinksProps) => {
  if (!code) return null;

  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2">
      {STOCK_INFO_LINKS.map((item) => (
        <a
          key={item.name}
          className="inline-flex items-center justify-between gap-1 rounded-md border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-primary hover:bg-slate-50 hover:border-primary transition-colors"
          href={item.href(code)}
          target='_blank'
          rel="noopener noreferrer"
          aria-label={`${item.name}（新しいタブで開く）`}
        >
          {item.name}
          <svg className="h-3.5 w-3.5 shrink-0 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
        </a>
      ))}
    </div>
  );
};
