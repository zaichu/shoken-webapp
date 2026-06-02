import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StockInfoLinks } from '../StockInfoLinks';

describe('StockInfoLinks', () => {
  it('すべてのリンクを表示する（11件）', () => {
    render(<StockInfoLinks code="7203" />);
    const links = screen.getAllByRole('link');
    expect(links).toHaveLength(11);
  });

  it('楽天証券リンクとSBI証券リンクを表示する', () => {
    render(<StockInfoLinks code="7203" />);

    const rakutenLink = screen.getByRole('link', { name: /楽天証券/ });
    const sbiLink = screen.getByRole('link', { name: /SBI証券/ });

    expect(rakutenLink).toHaveAttribute(
      'href',
      'https://www.rakuten-sec.co.jp/web/market/search/quote.html?ric=7203.T'
    );
    expect(sbiLink).toHaveAttribute(
      'href',
      'https://site3.sbisec.co.jp/ETGate/?_ControlID=WPLETsiR001Control&_DataStoreID=DSWPLETsiR001Control&_PageID=WPLETsiR001Ilst10&_ActionID=getDetailOfStockPriceJP&s_rkbn=1&i_stock_sec=%94%43%93%56%93%B0&i_dom_flg=1&i_exchange_code=JPN&i_output_type=0&stock_sec_code_mul=7203'
    );
  });

  it('JPX Explorerリンクを表示し、コードに-TSEサフィックスを付けたhrefになる', () => {
    render(<StockInfoLinks code="7203" />);

    const jpxLink = screen.getByRole('link', { name: /JPX Explorer/ });
    expect(jpxLink).toHaveAttribute(
      'href',
      'https://jpx-explorer.com/ja-JP/7203-TSE'
    );
  });

  it('銘柄コードがない場合は何も表示しない', () => {
    const { container } = render(<StockInfoLinks />);
    expect(container).toBeEmptyDOMElement();
  });
});
