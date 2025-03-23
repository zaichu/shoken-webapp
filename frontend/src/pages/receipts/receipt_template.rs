use chrono::NaiveDate;
use csv::StringRecord;
use gloo::console;
use itertools::Itertools;
use std::ops::Not;
use wasm_bindgen_futures::spawn_local;
use web_sys::{File, HtmlInputElement};
use yew::{prelude::*, virtual_dom::VNode};

use crate::{services::*, setting::*};

#[derive(Properties, PartialEq, Debug, Clone)]
pub struct ReceiptTemplateProps {
    pub name: String,
}

#[function_component]
pub fn ReceiptTemplate<T: ReceiptProps>(props: &ReceiptTemplateProps) -> Html {
    let receipts = use_state(Vec::<T>::new);
    let csv_file = use_state(|| None::<File>);
    let file_name = use_state(String::new);
    let query = use_state(|| None::<String>);

    {
        let file_name = file_name.clone();
        let receipts = receipts.clone();

        use_effect_with((*csv_file).clone(), move |csv_file| {
            handle_csv_file_change((*csv_file).clone(), file_name, receipts);
        });
    }

    html! {
        <>
            { render_csvfile_input(csv_file.clone(), file_name.clone()) }
            <div class="mt-2">
                <table class="table table-bordered">{ T::view_summary(&(*receipts)) }</table>
            </div>
            <div class="card shadow-sm">
                <div class="card-header bg-info text-white">
                    <div class="row align-items-center">
                        <div class="col col-lg-1"><h5 class="mb-0">{ props.name.clone() }</h5></div>
                        if T::is_view_search() {
                            <div class="col col-md-auto"><h6 class="mb-0">{ "銘柄コード:" }</h6></div>
                            <div class="col col-lg-2">{ render_search::<T>(&(*receipts), &query) }</div>
                        }
                    </div>
                </div>
                <div class="table-responsive" style="max-height: 500px;">
                    <table class="table table-bordered">
                        { render_thead::<T>() }
                        if csv_file.is_some() {
                            { render_tbody::<T>(&(*receipts), &(*query)) }
                        }
                    </table>
                </div>
            </div>
        </>
    }
}

fn render_search<T: ReceiptProps>(
    receipts: &Vec<T>,
    query: &UseStateHandle<Option<String>>,
) -> Html {
    html! {
            <select class="form-select form-select-sm" oninput={on_input_security_code_callback(query)}>
        <option selected=true />
        {
            receipts
                .into_iter()
                .map(|receipt| { (receipt.get_security_code().to_string(), receipt) })
                .sorted_by(|(a, _), (b, _)| a.cmp(b))
                .chunk_by(|(key, _)| key.clone())
                .into_iter()
                .map(|(security_code, group)| {
                    let security_name = group.max_by_key(|(_, r)| r.get_date().unwrap_or_default())
                                             .map(|(_, r)| r.get_security_name().to_string())
                                             .unwrap_or_default();

                    if security_code.is_empty() {
                        html! { <option value={security_name.clone()}>{security_name}</option> }
                    } else {
                        html! { <option value={security_code.clone()}>{format!("{}: {}", security_code, security_name)}</option> }
                    }
                })
                .collect::<Vec<VNode>>()
        }
    </select>

        }
}

fn render_csvfile_input(
    csv_file: UseStateHandle<Option<File>>,
    file_name: UseStateHandle<String>,
) -> Html {
    let on_input = on_input_csvfile_callback(csv_file.clone());
    html! {
    <div class="input-group">
        <label class="input-group-btn" for="csv-file-input">
            <span class="btn bg-info text-white">{ "CSVファイル選択" }</span>
        </label>
        <input id="csv-file-input" type="file" accept=".csv" style="display:none" oninput={on_input} />
        <input type="text" class="form-control form-control-sm" readonly=true placeholder="CSVファイルを選択してください。" value={(*file_name).clone()} />
    </div>
    }
}

fn render_thead<T: ReceiptProps>() -> Html {
    html! {
    <thead class="thead-light">
        <tr> {
            for T::new().get_all_fields().iter().map(|(header, _)| {
                let header_text = HEADERS.get(header).unwrap_or(header);
                html! {
                    <th scope="col" style="position: sticky; top: 0; background-color: white; white-space: nowrap; text-align: center;">
                        { header_text }
                    </th>
                }
            })
        }
        </tr>
    </thead>
    }
}

fn render_tbody<T: ReceiptProps>(receipts: &Vec<T>, query: &Option<String>) -> Html {
    html! {
        <tbody> {
            receipts
                .into_iter()
                .filter_map(|receipt| {
                    match &query {
                        Some(q) => receipt.search(q).then(|| (q.to_string(), receipt)),
                        None => receipt.get_date().map(|date| (date.to_string(), receipt)),
                    }
                })
                .chunk_by(|(key, _)| key.clone())
                .into_iter()
                .flat_map(|(_, group)| {
                    let receipts: Vec<&T> = group.map(|(_, receipt)| receipt).collect();
                    let mut views: Vec<Html> = receipts.iter().map(|r| r.view(None)).collect();
                    if let Some(summary) = T::new_summary(&receipts) {
                        views.push(summary.view(Some(format!("table-success"))));
                    }
                    views
                })
                .collect::<Vec<VNode>>()
        }
        </tbody>
    }
}

fn handle_csv_file_change<T: ReceiptProps>(
    csv_file: Option<File>,
    file_name: UseStateHandle<String>,
    receipts: UseStateHandle<Vec<T>>,
) {
    file_name.set("".to_string());

    if let Some(csv_file) = csv_file {
        spawn_local(async move {
            file_name.set(csv_file.name());
            if let Err(err) = csv_reader::read_file(&csv_file)
                .await
                .and_then(|content| process_csv_content(receipts, content))
            {
                console::log!(err.to_string());
            };
        });
    }
}

fn on_input_csvfile_callback(csv_file: UseStateHandle<Option<File>>) -> Callback<InputEvent> {
    Callback::from(move |e: InputEvent| {
        let input: HtmlInputElement = e.target_unchecked_into();
        let value = input.files().and_then(|files| files.get(0));
        csv_file.set(value.clone());
    })
}

fn on_input_security_code_callback(
    security_code: &UseStateHandle<Option<String>>,
) -> Callback<InputEvent> {
    let security_code = security_code.clone();
    Callback::from(move |e: InputEvent| {
        let input: HtmlInputElement = e.target_unchecked_into();
        let value = input.value();
        security_code.set(value.is_empty().not().then_some(value));
    })
}

fn process_csv_content<T: ReceiptProps>(
    receipts: UseStateHandle<Vec<T>>,
    content: Vec<u8>,
) -> Result<(), csv_reader::CSVError> {
    let records = csv_reader::read_csv(content)?;
    let new_receipts: Vec<_> = records
        .into_iter()
        .map(|record| T::new_from_string_record(record))
        .sorted_by(|a, b| {
            a.get_date()
                .unwrap_or_default()
                .cmp(&b.get_date().unwrap_or_default())
        })
        .collect();
    receipts.set(new_receipts);

    Ok(())
}

pub trait ReceiptProps: Clone + Sized + PartialEq + Default + 'static {
    fn new() -> Self;
    fn new_summary(_receipts: &[&Self]) -> Option<Self> {
        None
    }
    fn new_from_string_record(record: StringRecord) -> Self;

    fn get_all_fields(&self) -> Vec<(&'static str, Option<String>)>;
    fn get_date(&self) -> Option<NaiveDate>;

    fn get_security_code(&self) -> &str {
        ""
    }

    fn get_security_name(&self) -> &str {
        ""
    }

    fn search(&self, _query: &str) -> bool {
        true
    }

    fn is_view_search() -> bool {
        true
    }

    fn view_summary(receipts: &[Self]) -> Html;

    fn view(&self, tr_class: Option<String>) -> Html {
        html! {
            <tr class={tr_class}>
                { for self.get_all_fields().iter().map(|(key, value)| {
                    let value = value.as_deref().unwrap_or("");
                    let value = formater::format_value(key, value);
                    let style = "overflow-wrap: break-word; white-space: normal;";
                    let mut class = "text-nowrap".to_string();
                    if value.starts_with("¥ -") {
                        class = format!("{} text-danger", class);
                    }
                    html! {
                        <td class={class} style={style}>
                            {value}
                        </td>
                    }
                })}
            </tr>
        }
    }

    fn render_summary_th_td(key: &str, value: i32) -> Html {
        let style = "max-width: 30px;";
        let mut class = "text-nowrap".to_string();
        let value = &format!("{value}");
        let value = formater::format_value(key, value);
        if value.starts_with("¥ -") {
            class = format!("{} text-danger", class);
        }
        html! {
        <>
            <th class="bg-info text-white text-nowrap" style="max-width: 20px;">{HEADERS.get(key).unwrap_or(&key)}</th>
            <td class={class} style={style}>{value}</td>
        </>
        }
    }
}
