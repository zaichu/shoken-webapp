use crate::api::{ApiClient, ApiError};
use crate::dto::{MessageResponse, SessionUser};
use crate::pending_logout;
use leptos::prelude::*;

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

// 応答喪失はセッション確認で削除済みか未到達かを分け、未到達なら再送する(HTTP拒否は確定失敗)
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
            return Ok(());
        }
        if retries >= DELETE_ACCOUNT_MAX_RETRIES {
            return Err(error);
        }
        gloo_timers::future::TimeoutFuture::new(DELETE_ACCOUNT_RETRY_DELAY_MS << retries).await;
        retries += 1;
    }
}

#[derive(Clone, Copy)]
pub struct SessionStore {
    pub user: RwSignal<Option<SessionUser>>,
    pub loaded: RwSignal<bool>,
    pub generation: RwSignal<u64>,
}

impl SessionStore {
    pub fn new() -> Self {
        SessionStore {
            user: RwSignal::new(None),
            loaded: RwSignal::new(false),
            generation: RwSignal::new(0),
        }
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.generation.get_untracked() == generation
    }

    fn same_identity(old: &Option<SessionUser>, new: &Option<SessionUser>) -> bool {
        match (old, new) {
            (Some(a), Some(b)) => a.id == b.id,
            (None, None) => true,
            _ => false,
        }
    }

    fn set_user(&self, user: Option<SessionUser>) {
        let previous = self.user.get_untracked();
        if previous.is_some() && !Self::same_identity(&previous, &user) {
            self.generation.update(|generation| *generation += 1);
        }
        self.user.set(user);
    }

    pub async fn check(&self) {
        let client = ApiClient::auth_client();
        let user = client
            .get_json::<SessionUser>("/api/v1/session", &[])
            .await
            .ok()
            .filter(|user| !user.id.is_empty());
        batch(|| {
            self.set_user(user);
            self.loaded.set(true);
        });
    }

    pub fn mark_unauthenticated(&self) {
        self.set_user(None);
    }

    pub async fn logout(&self) {
        // 送信途中で閉じられても起動時に再送できるよう、結果を待たず先に記録する
        pending_logout::mark();
        self.mark_unauthenticated();
        self.loaded.set(true);
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
        self.mark_unauthenticated();
        self.loaded.set(true);
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
    crate::idle::watch_idle_logout(session);
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
mod tests {
    use super::*;

    fn alice() -> Option<SessionUser> {
        Some(SessionUser {
            id: "a".to_string(),
            email: "a@example.com".to_string(),
            name: None,
            picture_url: None,
        })
    }

    #[test]
    fn identity_comparison() {
        assert!(SessionStore::same_identity(&None, &None));
        assert!(SessionStore::same_identity(&alice(), &alice()));
        assert!(!SessionStore::same_identity(&None, &alice()));
        assert!(!SessionStore::same_identity(&alice(), &None));
    }

    #[test]
    fn generation_advances_on_identity_loss() {
        let owner = leptos::prelude::Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            assert_eq!(session.generation.get_untracked(), 0);
            session.set_user(None);
            assert_eq!(session.generation.get_untracked(), 0);
            session.set_user(alice());
            assert_eq!(session.generation.get_untracked(), 0);
            session.set_user(alice());
            assert_eq!(session.generation.get_untracked(), 0);
            session.set_user(None);
            assert_eq!(session.generation.get_untracked(), 1);
            // ログアウト時の世代進行で、再ログイン後の同一ユーザー応答と
            // ログアウト前の古い応答は区別できる
            session.set_user(alice());
            assert_eq!(session.generation.get_untracked(), 1);
            assert!(session.is_current(1));
            assert!(!session.is_current(0));
        });
    }
}
