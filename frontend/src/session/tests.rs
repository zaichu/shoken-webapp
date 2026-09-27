use super::*;

#[test]
fn oauth_url_appends_authorize_path() {
    assert_eq!(
        oauth_authorize_url(),
        format!("{}/api/v1/oauth/google/authorize", ApiClient::base_url())
    );
}

#[test]
fn delete_account_retry_budget_is_three() {
    assert_eq!(DELETE_ACCOUNT_MAX_RETRIES, 3);
    assert_eq!(DELETE_ACCOUNT_RETRY_DELAY_MS, 1_000);
}

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
    assert!(SessionStore::same_identity(None, None));
    assert!(SessionStore::same_identity(
        alice().as_ref(),
        alice().as_ref()
    ));
    assert!(!SessionStore::same_identity(None, alice().as_ref()));
    assert!(!SessionStore::same_identity(alice().as_ref(), None));
}

#[test]
fn logout_epoch_discards_result_started_before_logout() {
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let epoch = session.logout_epoch.get_untracked();
        assert!(session.check_result_applies(epoch, false));
        assert!(!session.check_result_applies(epoch, true));
        session.mark_unauthenticated();
        assert!(!session.check_result_applies(epoch, false));
        assert!(session.check_result_applies(session.logout_epoch.get_untracked(), false));
    });
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
