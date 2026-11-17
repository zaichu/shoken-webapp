use leptos::prelude::*;

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub fn MoreLink(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    #[prop(into, optional)] testid: String,
    /// 中央寄せの行で使うときは "more-link-centered" を渡す(右寄せの既定を止める)
    #[prop(into, optional)]
    extra_class: String,
) -> impl IntoView {
    let testid = if testid.is_empty() {
        None
    } else {
        Some(testid)
    };
    let class = if extra_class.is_empty() {
        "more-link".to_string()
    } else {
        format!("more-link {extra_class}")
    };
    view! {
        <a class=class href=href data-testid=testid>
            {label}
            <span aria-hidden="true">" ›"</span>
        </a>
    }
}
