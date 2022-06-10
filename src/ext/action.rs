use serde::{Deserialize, Serialize};
use walle_core::{action::ActionType, ExtendedValue, StandardAction};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetNewFriend {
    pub user_id: String,
    pub request_id: i64,
    pub accept: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteFriend {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", content = "params", rename_all = "snake_case")]
pub enum WalleExtraAction {
    SetNewFriend(SetNewFriend),
    DeleteFriend(DeleteFriend),
    GetNewFriendRequest(ExtendedValue),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum WalleAction {
    Standard(StandardAction),
    Extra(WalleExtraAction),
}

impl From<StandardAction> for WalleAction {
    fn from(s: StandardAction) -> Self {
        Self::Standard(s)
    }
}

impl From<WalleExtraAction> for WalleAction {
    fn from(e: WalleExtraAction) -> Self {
        Self::Extra(e)
    }
}

impl ActionType for WalleExtraAction {
    fn content_type(&self) -> walle_core::ContentType {
        walle_core::ContentType::Json
    }
}

impl ActionType for WalleAction {
    fn content_type(&self) -> walle_core::ContentType {
        match self {
            WalleAction::Standard(s) => s.content_type(),
            WalleAction::Extra(e) => e.content_type(),
        }
    }
}
