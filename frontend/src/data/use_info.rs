use serde::{Deserialize, Serialize};
use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Debug)]
pub struct UserInfo {
    pub auth_code: Option<String>,
    pub name: String,
    pub email: String,
}

impl Reducible for UserInfo {
    type Action = UserInfo;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        action.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_info_serialization() {
        let user = UserInfo {
            auth_code: Some("test_code".into()),
            name: "Taro".into(),
            email: "taro@example.com".into(),
        };

        let serialized = serde_json::to_string(&user).unwrap();
        let deserialized: UserInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(user, deserialized);
    }

    #[test]
    fn test_reduce_implementation() {
        let original = Rc::new(UserInfo::default());
        let new_user = UserInfo {
            auth_code: Some("new_code".into()),
            name: "Jiro".into(),
            email: "jiro@example.com".into(),
        };

        let reduced = original.reduce(new_user.clone());
        assert_eq!(*reduced, new_user);
    }
}
