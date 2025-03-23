use gloo::console;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::layout::Layout;
use crate::{data::stock::StockData, services::shoken_web_api, setting::*};

#[function_component]
pub fn Search() -> Html {
    let stock = use_state(StockData::default);
    let code_or_name = use_state(String::new);

    let on_input = {
        let stock = stock.clone();
        let code_or_name = code_or_name.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            code_or_name.set(value.clone());
            stock.set(StockData::default());

            console::log!(format!("Input value: {value}"));

            let stock = stock.clone();
            spawn_local(async move {
                match shoken_web_api::fetch_stock_data(&value).await {
                    Ok(new_stock) => stock.set(new_stock),
                    Err(err) => console::log!(&err.to_string()),
                }
            });
        })
    };

    html! {
        <Layout>
            <div class="mb-3">
                <input
                    type="text"
                    class="form-control form-control-lg shadow-sm"
                    id="stockCode"
                    placeholder="銘柄名・銘柄コードを入力"
                    value={(*code_or_name).clone()}
                    oninput={on_input}
                />
            </div>
            <div class="card shadow-sm">
                <div class="card-header bg-info text-white">
                    <h5 class="mb-0">{ "検索結果" }</h5>
                </div>
                <div class="card-body">
                    <table class="table">
                        <tbody>
                            { render_table_row("銘柄名", &stock.name) }
                            { render_table_row("銘柄コード", &stock.code) }
                            { render_table_row("マーケットカテゴリ", &stock.market_category) }
                            { render_table_row("33業種区分", &stock.industry_category_33.clone().unwrap_or_default()) }
                            { render_table_row("17業種区分", &stock.industry_category_17.clone().unwrap_or_default()) }
                            { render_table_row("規模区分", &stock.size_category.clone().unwrap_or_default()) }
                        </tbody>
                    </table>
                    <div class="d-flex flex-wrap">
                        { for STOCK_INFO_LINKS.iter().enumerate().map(|(i, (text, href))| {
                            html! {
                                <>
                                    <a href={href.replace("{}", &stock.code)} target="_blank" class="fw-bold">
                                        { text }
                                    </a>
                                    { if i < STOCK_INFO_LINKS.len() - 1 { html! { <span class="mx-2">{"|"} </span> } } else { html! {} } }
                                </>
                            }
                        })}
                    </div>
                </div>
            </div>
        </Layout>
    }
}

fn render_table_row(label: &str, value: &str) -> Html {
    html! {
        <tr>
            <th scope="row" width="125px">{ label }</th>
            <td>{ value }</td>
        </tr>
    }
}
