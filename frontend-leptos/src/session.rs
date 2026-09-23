use crate::api::ApiClient;
use crate::dto::SessionUser;
use leptos::prelude::*;

fn oauth_authorize_url() -> String {
    format!("{}/api/v1/oauth/google/authorize", ApiClient::base_url())
}

#[derive(Clone)]
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
        if !Self::same_identity(&self.user.get_untracked(), &user) {
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
        self.mark_unauthenticated();
        self.loaded.set(true);
        let client = ApiClient::default_client();
        let _ = client.delete_empty("/api/v1/session").await;
        Self::redirect_to("/login");
    }

    pub fn login(&self) {
        Self::redirect_to(&oauth_authorize_url());
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
    provide_context(session.clone());
    let startup = session.clone();
    leptos::task::spawn_local(async move {
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
    fn generation_advances_on_identity_change() {
        let owner = leptos::prelude::Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            assert_eq!(session.generation.get_untracked(), 0);
            session.set_user(None);
            assert_eq!(session.generation.get_untracked(), 0);
            session.set_user(alice());
            assert_eq!(session.generation.get_untracked(), 1);
            session.set_user(alice());
            assert_eq!(session.generation.get_untracked(), 1);
            session.set_user(None);
            assert_eq!(session.generation.get_untracked(), 2);
            session.set_user(alice());
            assert_eq!(session.generation.get_untracked(), 3);
            assert!(session.is_current(3));
            assert!(!session.is_current(2));
        });
    }
}
