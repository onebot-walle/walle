use super::TempMatcher;
use crate::{ActionCaller, ActionCallerExt, MatcherHandlerExt, MatchersConfig, TempMatchers};
use std::{sync::Arc, time::Duration};
use walle_core::{
    action::SendMessage,
    event::{
        BaseEvent, Group, ImplLevel, Message, MessageDeatilTypes, PlatformLevel, Private,
        SubTypeLevel, TryFromEvent,
    },
    prelude::async_trait,
    segment::{IntoMessage, Segments},
    structs::SendMessageResp,
    WalleError, WalleResult,
};

/// Matcher 使用的 Session
#[derive(Clone)]
pub struct Session<T = (), D = (), S = (), P = (), I = ()> {
    pub event: BaseEvent<T, D, S, P, I>,
    pub config: Arc<MatchersConfig>,
    pub caller: Arc<dyn ActionCaller + Send + 'static>,
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

impl<S, P, I> Session<Message, Private, S, P, I>
where
    S: Sync,
    P: Sync,
    I: Sync,
{
    pub async fn send(&self, message: Segments) -> WalleResult<SendMessageResp> {
        self.call(SendMessage {
            detail_type: "private".to_string(),
            user_id: Some(self.event.ty.user_id.clone()),
            group_id: None,
            channel_id: None,
            guild_id: None,
            message,
        })
        .await
    }
}

impl<S, P, I> Session<Message, Group, S, P, I>
where
    S: Sync,
    P: Sync,
    I: Sync,
{
    pub async fn send(&self, message: Segments) -> WalleResult<SendMessageResp> {
        self.call(SendMessage {
            detail_type: "group".to_string(),
            user_id: Some(self.event.ty.user_id.clone()),
            group_id: Some(self.event.detail_type.group_id.clone()),
            channel_id: None,
            guild_id: None,
            message,
        })
        .await
    }
}

#[async_trait]
impl<S, P, I> ReplyAbleSession for Session<Message, MessageDeatilTypes, S, P, I>
where
    S: TryFromEvent<SubTypeLevel> + Send + Sync + 'static,
    P: TryFromEvent<PlatformLevel> + Send + Sync + 'static,
    I: TryFromEvent<ImplLevel> + Send + Sync + 'static,
{
    async fn send<M: IntoMessage + Send + 'static>(
        &self,
        message: M,
    ) -> WalleResult<SendMessageResp> {
        let group_id = match &self.event.detail_type {
            MessageDeatilTypes::Group(group) => Some(group.group_id.clone()),
            _ => None,
        };
        self.call(SendMessage {
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
        })
        .await
    }
    async fn get<M>(&mut self, message: M, duration: Option<Duration>) -> WalleResult<()>
    where
        M: IntoMessage + Send + 'static,
    {
        use crate::builtin::{group_id_check, user_id_check};
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let temp = TempMatcher { tx }.with_rule(user_id_check(&self.event.ty.user_id));
        let mut temps = self.temps.lock().await;
        if let MessageDeatilTypes::Group(group) = &self.event.detail_type {
            temps.insert(
                self.event.id.clone(),
                temp.with_rule(group_id_check(&group.group_id)).boxed(),
            );
        } else {
            temps.insert(self.event.id.clone(), temp.boxed());
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
