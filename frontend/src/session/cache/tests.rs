use super::*;

fn user(id: &str, name: Option<&str>) -> SessionUser {
    SessionUser {
        id: id.to_string(),
        email: format!("{id}@example.com"),
        name: name.map(str::to_string),
        picture_url: None,
    }
}

#[test]
fn snapshot_roundtrips_display_fields() {
    let snapshot = to_snapshot(&user("alice", Some("Alice Liddell"))).unwrap();
    let restored = hydrate(snapshot).unwrap();
    assert_eq!(restored.id, "alice");
    assert_eq!(restored.name.as_deref(), Some("Alice Liddell"));
    // メールアドレスは保存・復元しない
    assert!(restored.email.is_empty());
}

#[test]
fn nameless_user_is_not_snapshotted() {
    assert!(to_snapshot(&user("alice", None)).is_none());
    assert!(to_snapshot(&user("alice", Some(""))).is_none());
    assert!(to_snapshot(&user("", Some("Alice"))).is_none());
}

#[test]
fn hydrate_rejects_incomplete_snapshot() {
    let empty_id = Snapshot {
        id: String::new(),
        name: "Alice".to_string(),
        picture_url: None,
    };
    let empty_name = Snapshot {
        id: "alice".to_string(),
        name: String::new(),
        picture_url: None,
    };
    assert!(hydrate(empty_id).is_none());
    assert!(hydrate(empty_name).is_none());
}

#[test]
fn snapshot_json_contains_no_email() {
    let snapshot = to_snapshot(&user("alice", Some("Alice"))).unwrap();
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(!json.contains("alice@example.com"));
    assert!(!json.contains("email"));
}
