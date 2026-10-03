mod cross_tab;
mod idle;
mod pending_logout;
mod probe;

use crate::api::dto::{MessageResponse, SessionUser};
use crate::api::{ApiClient, ApiError};
use leptos::prelude::*;

pub(crate) fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

fn oauth_authorize_url() -> String {
    format!("{}/api/v1/oauth/google/authorize", ApiClient::base_url())
}

async fn session_invalidated() -> bool {
    matches!(
        ApiClient::auth_client()
            .get_json::<SessionUser>("/api/v1/session", &[])
            .await,
        Err(error) if error.is_unauthorized()
    )
}

const DELETE_ACCOUNT_MAX_RETRIES: u32 = 3;
const DELETE_ACCOUNT_RETRY_DELAY_MS: u32 = 1_000;

// プローブ(3秒で打ち切り)に続く撃ち直しの時間予算
const PROBE_FOLLOWUP_TIMEOUT_MS: u64 = 7_000;

// 応答喪失はセッションの生死で再送可否を分ける(HTTP拒否は確定失敗)
async fn delete_with_verification(client: &ApiClient) -> Result<(), ApiError> {
    let mut retries = 0;
    loop {
        let error = match client.delete_empty("/api/v1/account").await {
            Ok(()) => return Ok(()),
            Err(error) => error,
        };
        if !matches!(error, ApiError::Network | ApiError::Timeout) {
            return Err(error);
        }
        if session_invalidated().await {
            // セッション失効は削除済みか通常の失効か区別できないため、成功とはみなさない
            return Err(ApiError::Http {
                status: 401,
                server_message: None,
            });
        }
        if retries >= DELETE_ACCOUNT_MAX_RETRIES {
            return Err(error);
        }
        gloo_timers::future::TimeoutFuture::new(DELETE_ACCOUNT_RETRY_DELAY_MS << retries).await;
        retries += 1;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Generation(u64);

impl Generation {
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    fn advance(&mut self) {
        *self = self.next();
    }
}

#[derive(Clone, Copy)]
pub struct SessionStore {
    pub user: RwSignal<Option<SessionUser>>,
    pub loaded: RwSignal<bool>,
    pub generation: RwSignal<Generation>,
    logout_epoch: RwSignal<u64>,
}

impl SessionStore {
    pub fn new() -> Self {
        SessionStore {
            user: RwSignal::new(None),
            loaded: RwSignal::new(false),
            generation: RwSignal::new(Generation::default()),
            logout_epoch: RwSignal::new(0),
        }
    }

    pub fn is_current(&self, generation: Generation) -> bool {
        self.generation.get_untracked() == generation
    }

    fn same_identity(old: Option<&SessionUser>, new: Option<&SessionUser>) -> bool {
        match (old, new) {
            (Some(a), Some(b)) => a.id == b.id,
            (None, None) => true,
            _ => false,
        }
    }

    fn set_user(&self, user: Option<SessionUser>) {
        let previous = self.user.get_untracked();
        if previous.is_some() && !Self::same_identity(previous.as_ref(), user.as_ref()) {
            self.generation.update(Generation::advance);
        }
        self.user.set(user);
    }

    pub async fn check(&self) {
        // 送信前に採った時点からログアウトが起きていれば、遅れて届いた応答で復活させない
        let epoch = self.logout_epoch.get_untracked();
        let client = ApiClient::auth_client();
        // プローブの結果は起動時の1回だけ使う。4xx の確定した拒否以外は撃ち直す
        let fetched = match probe::take().await {
            probe::Probe::Authenticated(user) => Some(user),
            probe::Probe::Anonymous => None,
            probe::Probe::Retry => client
                .with_timeout_ms(PROBE_FOLLOWUP_TIMEOUT_MS)
                .with_max_retries(0)
                .get_json::<SessionUser>("/api/v1/session", &[])
                .await
                .ok(),
            probe::Probe::Missing => client
                .get_json::<SessionUser>("/api/v1/session", &[])
                .await
                .ok(),
        };
        let user = fetched.filter(|user| !user.id.is_empty());
        batch(|| {
            if self.check_result_applies(epoch, pending_logout::is_pending()) {
                self.set_user(user);
            } else {
                self.set_user(None);
            }
            self.loaded.set(true);
        });
    }

    fn check_result_applies(&self, epoch: u64, pending_logout: bool) -> bool {
        !pending_logout && self.logout_epoch.get_untracked() == epoch
    }

    pub fn mark_unauthenticated(&self) {
        self.logout_epoch.update(|epoch| *epoch += 1);
        self.set_user(None);
    }

    pub async fn logout(&self) {
        // 送信途中で閉じられても起動時に再送できるよう、結果を待たず先に記録する
        pending_logout::mark();
        self.mark_unauthenticated();
        self.loaded.set(true);
        cross_tab::notify_logout();
        let client = ApiClient::default_client();
        if pending_logout::is_finished(&client.delete_empty("/api/v1/session").await) {
            pending_logout::clear();
        } else {
            pending_logout::start_retry_loop();
        }
    }

    pub async fn delete_account(&self) -> Result<(), ApiError> {
        let client = ApiClient::default_client().with_max_retries(0);
        let result = match client
            .post_json::<serde_json::Value, MessageResponse>(
                "/api/v1/account-deletion-confirmations",
                &serde_json::json!({}),
            )
            .await
        {
            Err(error) => Err(error),
            Ok(_) => delete_with_verification(&client).await,
        };
        // 削除失敗時はセッションを残し、呼び出し側がエラーを出して再試行できるようにする
        let session_lost = match &result {
            Ok(()) => true,
            Err(error) => error.is_unauthorized(),
        };
        if session_lost {
            self.mark_unauthenticated();
            self.loaded.set(true);
            cross_tab::notify_logout();
        }
        result
    }

    pub fn login(&self) {
        if pending_logout::is_pending() {
            leptos::task::spawn_local(async move {
                let client = ApiClient::auth_client().with_max_retries(0);
                // 失敗しても消す。記録を残すと戻ってきたとき新しいセッションまで消える
                let _ = client.delete_empty("/api/v1/session").await;
                pending_logout::clear();
                Self::redirect_to(&oauth_authorize_url());
            });
        } else {
            Self::redirect_to(&oauth_authorize_url());
        }
    }

    pub fn redirect_to(path: &str) {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(path);
        }
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn provide_session() -> SessionStore {
    let session = SessionStore::new();
    provide_context(session);
    idle::watch_idle_logout(session);
    cross_tab::watch_logout_notifications(session);
    let startup = session;
    leptos::task::spawn_local(async move {
        // 保留中はセッション確認を行わず、先にログアウトを完了させる
        if pending_logout::is_pending() {
            startup.logout().await;
            return;
        }
        startup.check().await;
    });
    session
}

pub fn use_session() -> SessionStore {
    expect_context::<SessionStore>()
}

#[cfg(test)]
mod tests;
