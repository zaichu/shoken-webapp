use leptos::prelude::*;

pub fn allowed_avatar_url(url: Option<&str>) -> Option<String> {
    let raw = url.filter(|s| !s.is_empty())?;
    let rest = raw.strip_prefix("https://")?;
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.split('@').next_back()?;
    let host = host.split(':').next()?;
    if host == "googleusercontent.com" || host.ends_with(".googleusercontent.com") {
        Some(raw.to_string())
    } else {
        None
    }
}

pub fn avatar_initials(name: Option<&str>, email: Option<&str>) -> String {
    if let Some(name) = name.filter(|name| !name.is_empty()) {
        let parts: Vec<&str> = name.split_whitespace().collect();
        if parts.len() >= 2 {
            let first = parts[0].chars().next().unwrap_or_default();
            let last = parts[parts.len() - 1].chars().next().unwrap_or_default();
            return format!("{first}{last}").to_uppercase();
        }
        return name.chars().take(2).collect::<String>().to_uppercase();
    }
    if let Some(email) = email.filter(|email| !email.is_empty()) {
        return email.chars().take(2).collect::<String>().to_uppercase();
    }
    "U".to_string()
}

#[component]
pub fn UserAvatar(
    picture_url: Option<String>,
    name: Option<String>,
    email: String,
    #[prop(optional)] large: bool,
    #[prop(into, optional)] alt: Signal<String>,
) -> impl IntoView {
    let allowed = allowed_avatar_url(picture_url.as_deref());
    let failed = RwSignal::new(false);
    let initials = avatar_initials(name.as_deref(), Some(email.as_str()));
    let (badge_class, img_class, size) = if large {
        (
            "flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-gold text-brand font-bold text-2xl",
            "h-14 w-14 shrink-0 rounded-full object-cover",
            (56, 56),
        )
    } else {
        (
            "flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gold text-brand font-bold text-lg",
            "h-10 w-10 shrink-0 rounded-full object-cover",
            (40, 40),
        )
    };
    let (width, height) = size;
    view! {
        {move || {
            match allowed.clone() {
                Some(src) if !failed.get() => {
                    view! {
                        <img
                            src=src
                            alt=move || alt.get()
                            referrerpolicy="no-referrer"
                            loading="lazy"
                            width=width
                            height=height
                            class=img_class
                            on:error=move |_| failed.set(true)
                        />
                    }
                        .into_any()
                }
                _ => {
                    view! { <span class=badge_class>{initials.clone()}</span> }.into_any()
                }
            }
        }}
    }
}

#[cfg(test)]
mod tests;
