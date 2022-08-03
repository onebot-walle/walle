use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use walle_core::{
    action::SendMessage,
    event::{
        Group, ImplDeclare, Message, MessageDeatilTypes, PlatformDeclare, Private, SubTypeDeclare,
    },
    prelude::*,
    structs::SendMessageResp,
};

use crate::MatchersConfig;
use walle_core::{
    action::Action,
    event::{BaseEvent, Event},
    resp::Resp,
    segment::IntoMessage,
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
        self.caller.call(action).await
    }
}

impl<D, S, P, I> Session<Message, D, S, P, I> {
    pub fn message(&self) -> &Segments {
        &self.event.ty.message
    }
    pub fn message_mut(&mut self) -> &mut Segments {
        &mut self.event.ty.message
    }
    pub fn update_alt(&mut self) {
        self.event.ty.alt_message = self.message().iter().map(|seg| seg.alt()).collect();
    }
}

#[async_trait]
pub trait ReplyAbleSession {
    async fn send<M: IntoMessage + Send + 'static>(
        &self,
        message: M,
    ) -> WalleResult<SendMessageResp>;
    async fn get<M: IntoMessage + Send + 'static>(
        &mut self,
        message: M,
        timeout: Option<Duration>,
    ) -> WalleResult<()>;
}

impl<S, P, I> Session<Message, Private, S, P, I> {
    pub async fn send(&self, message: Segments) -> WalleResult<SendMessageResp> {
        self.call(
            SendMessage {
                detail_type: "private".to_string(),
                user_id: Some(self.event.ty.user_id.clone()),
                group_id: None,
                channel_id: None,
                guild_id: None,
                message,
            }
            .into(),
        )
        .await?
        .as_result()?
        .try_into()
    }
}

impl<S, P, I> Session<Message, Group, S, P, I> {
    pub async fn send(&self, message: Segments) -> WalleResult<SendMessageResp> {
        self.call(
            SendMessage {
                detail_type: "group".to_string(),
                user_id: Some(self.event.ty.user_id.clone()),
                group_id: Some(self.event.detail_type.group_id.clone()),
                channel_id: None,
                guild_id: None,
                message,
            }
            .into(),
        )
        .await?
        .as_result()?
        .try_into()
    }
}

#[async_trait]
impl<S, P, I> ReplyAbleSession for Session<Message, MessageDeatilTypes, S, P, I>
where
    S: for<'a> TryFrom<&'a mut Event, Error = WalleError>
        + std::fmt::Debug
        + SubTypeDeclare
        + Send
        + Sync
        + 'static,
    P: for<'a> TryFrom<&'a mut Event, Error = WalleError>
        + std::fmt::Debug
        + PlatformDeclare
        + Send
        + Sync
        + 'static,
    I: for<'a> TryFrom<&'a mut Event, Error = WalleError>
        + std::fmt::Debug
        + ImplDeclare
        + Send
        + Sync
        + 'static,
{
    async fn send<M: IntoMessage + Send + 'static>(
        &self,
        message: M,
    ) -> WalleResult<SendMessageResp> {
        let group_id = match &self.event.detail_type {
            MessageDeatilTypes::Group(group) => Some(group.group_id.clone()),
            _ => None,
        };
        self.call(
            SendMessage {
                detail_type: if group_id.is_some() {
                    "group".to_string()
                } else {
                    "private".to_string()
                },
                user_id: Some(self.event.ty.user_id.clone()),
                group_id,
                channel_id: None,
                guild_id: None,
                message: message.into_message(),
            }
            .into(),
        )
        .await?
        .as_result()?
        .try_into()
    }
    async fn get<M>(&mut self, message: M, duration: Option<Duration>) -> WalleResult<()>
    where
        M: IntoMessage + Send + 'static,
    {
        use crate::builtin::{group_id_check, user_id_check};
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let temp = TempMatcher { tx }.with_rule(user_id_check(&self.event.ty.user_id));
        if let MessageDeatilTypes::Group(group) = &self.event.detail_type {
            self.temps.insert(
                self.event.id.clone(),
                temp.with_rule(group_id_check(&group.group_id)).boxed(),
            );
        } else {
            self.temps.insert(self.event.id.clone(), temp.boxed());
        }
        self.send(message.into_message()).await?;
        match tokio::time::timeout(duration.unwrap_or(Duration::from_secs(30)), rx.recv()).await {
            Ok(Some(event)) => {
                self.event = event;
                Ok(())
            }
            Ok(None) => Err(WalleError::Other("unexpected tx drop".to_string())),
            Err(e) => Err(WalleError::Other(e.to_string())),
        }
    }
}

#[derive(Clone)]
pub struct Bot {
    pub self_id: String,
    pub caller: Arc<dyn ActionCaller + Send + 'static>,
}

impl Bot {
    pub async fn call(&self, mut action: Action) -> WalleResult<Resp> {
        action
            .params
            .insert("self_id".to_string(), self.self_id.as_str().into());
        self.caller.call(action).await
    }
}

#[async_trait]
pub trait ActionCaller: Sync {
    async fn call(&self, action: Action) -> WalleResult<Resp>;
    async fn get_bots(self: Arc<Self>) -> Vec<Bot>;
}

#[async_trait]
impl<AH, EH> ActionCaller for OneBot<AH, EH, 12>
where
    AH: ActionHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
    EH: EventHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
{
    fn call<'a, 't>(
        &'a self,
        action: Action,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.action_handler.call(action)
    }

    async fn get_bots(self: Arc<Self>) -> Vec<Bot> {
        self.action_handler
            .self_ids()
            .await
            .into_iter()
            .map(|id| Bot {
                self_id: id,
                caller: self.clone(),
            })
            .collect()
    }
}

impl ActionCaller for Bot {
    fn call<'a, 't>(
        &'a self,
        action: Action,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.caller.call(action)
    }
    fn get_bots<'t>(self: Arc<Self>) -> Pin<Box<dyn Future<Output = Vec<Bot>> + Send + 't>>
    where
        Self: 't,
    {
        self.caller.clone().get_bots()
    }
}

struct TempMatcher<T, D, S, P, I> {
    pub tx: tokio::sync::mpsc::UnboundedSender<BaseEvent<T, D, S, P, I>>,
}

#[async_trait]
impl<T, D, S, P, I> MatcherHandler<T, D, S, P, I> for TempMatcher<T, D, S, P, I>
where
    T: Send + 'static,
    D: Send + 'static,
    S: Send + 'static,
    P: Send + 'static,
    I: Send + 'static,
{
    async fn handle(&self, session: Session<T, D, S, P, I>) {
        self.tx.send(session.event).ok();
    }
}
