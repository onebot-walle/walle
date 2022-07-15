use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use walle_core::prelude::*;

use crate::MatchersConfig;
use walle_core::{
    action::Action,
    event::{BaseEvent, Event},
    resp::Resp,
};

mod handle;
mod hook;
mod matchers;
mod pre_handle;
mod rule;

pub use handle::*;
pub use hook::*;
pub use matchers::*;
pub use pre_handle::*;
pub use rule::*;

/// Matcher 使用的 Session
#[derive(Clone)]
pub struct Session<T = (), D = (), S = (), P = (), I = ()> {
    pub event: BaseEvent<T, D, S, P, I>,
    pub config: Arc<MatchersConfig>,
    caller: Arc<dyn ActionCaller + Send + 'static>,
    temps: TempMatchers,
}

impl<T, D, S, P, I> Session<T, D, S, P, I> {
    pub fn new(
        event: BaseEvent<T, D, S, P, I>,
        caller: Arc<dyn ActionCaller + Send + 'static>,
        config: Arc<MatchersConfig>,
        temps: TempMatchers,
    ) -> Self {
        Self {
            event,
            config,
            caller,
            temps,
        }
    }

    pub async fn call(&self, mut action: Action) -> WalleResult<Resp> {
        action
            .params
            .insert("self_id".to_string(), self.event.self_id.as_str().into());
        self.caller.clone().call(action).await
    }
}

pub trait ActionCaller: Sync {
    fn call(
        self: Arc<Self>,
        action: Action,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 'static>>;
}

impl<AH, EH> ActionCaller for OneBot<AH, EH, 12>
where
    AH: ActionHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
    EH: EventHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
{
    fn call(
        self: Arc<Self>,
        action: Action,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 'static>> {
        Box::pin(async move { self.action_handler.call(action, &self).await })
    }
}
