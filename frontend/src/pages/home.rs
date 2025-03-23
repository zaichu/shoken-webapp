use yew::prelude::*;

use super::layout::Layout;

#[function_component]
pub fn Home() -> Html {
    html! {
        <Layout>
            <div class="jumbotron">
                <h1 class="display-4">{ "証券Webへようこそ" }</h1>
                <p class="lead">{ "このプラットフォームでは、銘柄検索や受取金の管理ができます。" }</p>
                <hr class="my-4" />
                <p>{ "さまざまな機能を使って、投資をより効率的に管理しましょう。" }</p>
            </div>
        </Layout>
    }
}
