use super::TempMatcher;
use crate::{
    ActionCaller, ActionCallerExt, MatcherHandler, MatchersConfig, PreHandler, Rule, TempMatchers,
};
use std::{sync::Arc, time::Duration};
use walle_core::{
    event::{
        BaseEvent, DetailTypeLevel, ImplLevel, ParseEvent, PlatformLevel, SubTypeLevel,
        TryFromEvent, TypeLevel,
    },
    prelude::{async_trait, Event},
    segment::{IntoMessage, MsgSegment, Segments},
    structs::{Selft, SendMessageResp},
    util::{Value, ValueMap, ValueMapExt},
    WalleError, WalleResult,
};

/// Matcher 使用的 Session
#[derive(Clone)]
pub struct Session {
    pub event: Event,
    pub config: Arc<MatchersConfig>,
    pub caller: Arc<dyn ActionCaller + Send + 'static>,
    reply_sign: ReplySign,
    temps: TempMatchers,
    pub(crate) selft: Option<Selft>,
}

impl Session {
    pub fn new(
        event: Event,
        caller: Arc<dyn ActionCaller + Send + 'static>,
        config: Arc<MatchersConfig>,
        temps: TempMatchers,
    ) -> Self {
        let reply_sign = ReplySign::new(&event);
        Self {
            selft: event.selft(),
            event,
            config,
            caller,
            reply_sign,
            temps,
        }
    }
}

#[derive(Clone)]
enum ReplySign {
    Private(String),
    Group(String, String),
    Channel(String, String, String),
    UnReplyAble,
}

impl ReplySign {
    fn new(event: &Event) -> Self {
        if let (Ok(guild_id), Ok(channel_id), Ok(user_id)) = (
            event.extra.get_downcast("guild_id"),
            event.extra.get_downcast("channel_id"),
            event.extra.get_downcast("user_id"),
        ) {
            ReplySign::Channel(guild_id, channel_id, user_id)
        } else if let (Ok(group_id), Ok(user_id)) = (
            event.extra.get_downcast("group_id"),
            event.extra.get_downcast("user_id"),
        ) {
            ReplySign::Group(group_id, user_id)
        } else if let Ok(user_id) = event.extra.get_downcast("user_id") {
            ReplySign::Private(user_id)
        } else {
            ReplySign::UnReplyAble
        }
    }

    fn ruled<H, PH>(
        &self,
        handler: H,
        extra_pre_handler: PH,
        this_user_only: bool,
    ) -> WalleResult<Box<dyn MatcherHandler + Send + Sync + 'static>>
    where
        H: MatcherHandler + Send + Sync + 'static,
        PH: PreHandler + Send + Sync + 'static,
    {
        use crate::builtin::*;
        Ok(match self {
            ReplySign::Private(user_id) => extra_pre_handler
                .with_rule(user_id_check(user_id))
                .layer(handler)
                .boxed(),
            ReplySign::Group(group_id, user_id) => {
                if this_user_only {
                    extra_pre_handler
                        .with_rule(user_id_check(user_id))
                        .with_rule(group_id_check(group_id))
                        .layer(handler)
                        .boxed()
                } else {
                    extra_pre_handler
                        .with_rule(group_id_check(group_id))
                        .layer(handler)
                        .boxed()
                }
            }
            ReplySign::Channel(guild_id, channel_id, user_id) => {
                if this_user_only {
                    extra_pre_handler
                        .with_rule(user_id_check(user_id))
                        .with_rule(channel_id_check(guild_id, channel_id))
                        .layer(handler)
                        .boxed()
                } else {
                    extra_pre_handler
                        .with_rule(channel_id_check(guild_id, channel_id))
                        .layer(handler)
                        .boxed()
                }
            }
            ReplySign::UnReplyAble => {
                return Err(WalleError::Other("unreplyable session".to_string()))
            }
        })
    }
}

impl Session {
    pub async fn reply<M: IntoMessage + Send>(&self, message: M) -> WalleResult<SendMessageResp> {
        match &self.reply_sign {
            ReplySign::Private(user_id) => {
                self.send_private_message(user_id.clone(), message).await
            }
            ReplySign::Group(group_id, ..) => {
                self.send_group_message(group_id.clone(), message).await
            }
            ReplySign::Channel(guild_id, channel_id, ..) => {
                self.send_channel_message(guild_id.clone(), channel_id.clone(), message)
                    .await
            }
            ReplySign::UnReplyAble => Err(WalleError::Other("unreplyable session".to_string())),
        }
    }
    pub async fn get<M>(&mut self, message: M, this_user_only: bool) -> WalleResult<SendMessageResp>
    where
        M: IntoMessage + Send,
    {
        self.get_with_pre_handler(message, (), this_user_only).await
    }
    pub async fn get_with_rule<M, R>(
        &mut self,
        message: M,
        rule: R,
        this_user_only: bool,
    ) -> WalleResult<SendMessageResp>
    where
        M: IntoMessage + Send,
        R: Rule + Send + Sync + 'static,
    {
        self.get_with_pre_handler(message, ().with_rule(rule), this_user_only)
            .await
    }
    pub async fn get_with_pre_handler<M, PH>(
        &mut self,
        message: M,
        pre_handler: PH,
        this_user_only: bool,
    ) -> WalleResult<SendMessageResp>
    where
        M: IntoMessage + Send,
        PH: PreHandler + Send + Sync + 'static,
    {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let temp = self
            .reply_sign
            .ruled(TempMatcher { tx }, pre_handler, this_user_only)?;
        self.temps.lock().await.insert(self.event.id.clone(), temp);
        let resp = self.reply(message).await?;
        if let Ok(Some(event)) = tokio::time::timeout(Duration::from_secs(60), rx.recv()).await {
            self.event = event;
        } else {
            self.temps.lock().await.remove(&self.event.id);
        }
        Ok(resp)
    }
}

#[async_trait]
pub trait FromSessionPart: Sized {
    async fn from_session_part(session: &mut Session) -> WalleResult<Self>;
}

#[async_trait]
pub trait FromSession: Sized {
    async fn from_session(session: Session) -> WalleResult<Self>;
}

impl FromSessionPart for Segments {
    fn from_session_part<'life0, 'async_trait>(
        session: &'life0 mut Session,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = WalleResult<Self>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let segments = session
                .event
                .extra
                .try_get_as_mut::<&mut Vec<Value>>("message")?;
            let segments = std::mem::replace(segments, vec![]);
            segments
                .into_iter()
                .map(|v| MsgSegment::try_from(v))
                .collect()
        })
    }
}

impl<T, D, S, P, I> FromSessionPart for BaseEvent<T, D, S, P, I>
where
    T: TryFromEvent<TypeLevel>,
    D: TryFromEvent<DetailTypeLevel>,
    S: TryFromEvent<SubTypeLevel>,
    P: TryFromEvent<PlatformLevel>,
    I: TryFromEvent<ImplLevel>,
{
    fn from_session_part<'life0, 'async_trait>(
        session: &'life0 mut Session,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = WalleResult<Self>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let event = std::mem::replace(
                &mut session.event,
                Event {
                    id: String::default(),
                    time: 0.0,
                    ty: String::default(),
                    detail_type: String::default(),
                    sub_type: String::default(),
                    extra: ValueMap::default(),
                },
            );
            let implt = "todo"; //todo
            Self::parse(event, implt)
        })
    }
}

impl FromSession for Session {
    fn from_session<'async_trait>(
        session: Session,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = WalleResult<Self>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        Self: 'async_trait,
    {
        Box::pin(async move { Ok(session) })
    }
}

#[async_trait]
impl<T: FromSessionPart> FromSession for T {
    async fn from_session(mut session: Session) -> WalleResult<Self> {
        Self::from_session_part(&mut session).await
    }
}
