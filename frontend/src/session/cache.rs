use crate::api::dto::SessionUser;
use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "session_snapshot";

// 起動時に先に画面を出すための前回ログイン状態。表示に必要なものだけ置き、
// メールアドレスやトークンなどの個人情報・機密情報は保存しない
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Snapshot {
    id: String,
    name: String,
    #[serde(default)]
    picture_url: Option<String>,
}

fn storage() -> Option<web_sys::Storage> {
    // web_sys のアクセスは wasm32 でしか動かない(js-sys の静的 import は native で panic する)
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

// 表示名が無いユーザーはヘッダーがメールアドレスに倒すため、保存対象外にして
// メールアドレス由来の値が localStorage に残らないようにする
fn to_snapshot(user: &SessionUser) -> Option<Snapshot> {
    let name = user.name.as_ref().filter(|name| !name.is_empty())?;
    if user.id.is_empty() {
        return None;
    }
    Some(Snapshot {
        id: user.id.clone(),
        name: name.clone(),
        picture_url: user.picture_url.clone(),
    })
}

fn hydrate(snapshot: Snapshot) -> Option<SessionUser> {
    if snapshot.id.is_empty() || snapshot.name.is_empty() {
        return None;
    }
    Some(SessionUser {
        id: snapshot.id,
        email: String::new(),
        name: Some(snapshot.name),
        picture_url: snapshot.picture_url,
    })
}

pub fn load() -> Option<SessionUser> {
    let text = storage()?.get_item(STORAGE_KEY).ok().flatten()?;
    serde_json::from_str::<Snapshot>(&text)
        .ok()
        .and_then(hydrate)
}

pub fn save(user: &SessionUser) {
    let Some(storage) = storage() else {
        return;
    };
    match to_snapshot(user) {
        Some(snapshot) => {
            if let Ok(json) = serde_json::to_string(&snapshot) {
                let _ = storage.set_item(STORAGE_KEY, &json);
            }
        }
        // 名前のないユーザーに変わったとき、前のユーザーの表示名を残さない
        None => clear(),
    }
}

pub fn clear() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(STORAGE_KEY);
    }
}

#[cfg(test)]
mod tests;
