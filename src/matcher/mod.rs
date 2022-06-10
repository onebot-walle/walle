use async_trait::async_trait;
use std::sync::Arc;
use walle_core::action::BotActionExt;
use walle_core::resp::SendMessageRespContent;
use walle_core::{
    BaseEvent, EventContent, IntoMessage, Message, MetaContent, NoticeContent, RequestContent,
    Resp, WalleResult,
};

use crate::{MessageContent, WalleBot};

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

use crate::MatchersConfig;

/// 单个匹配执行的最小单位
#[derive(Clone)]
pub struct Matcher<C> {
    pub name: &'static str,
    pub description: &'static str,
    pub handler: Arc<dyn MatcherHandler<C> + Sync + Send + 'static>,
}

impl<C> Matcher<C>
where
    C: Clone + Send + Sync + 'static,
{
    pub fn new(
        name: &'static str,
        description: &'static str,
        handler: impl MatcherHandler<C> + Sync + Send + 'static,
    ) -> Self {
        Self {
            name,
            description,
            handler: Arc::new(handler),
        }
    }

    pub fn new_with<H0, H1, F>(
        name: &'static str,
        description: &'static str,
        handler: H0,
        f: F,
    ) -> Self
    where
        H0: MatcherHandler<C> + Sync + Send + 'static,
        H1: MatcherHandler<C> + Sync + Send + 'static,
        F: FnOnce(H0) -> H1,
    {
        Self {
            name,
            description,
            handler: Arc::new(f(handler)),
        }
    }

    pub async fn call(&self, session: &Session<C>) {
        if self.handler._match(session) {
            let mut session = session.clone();
            if self.handler._pre_handle(&mut session) {
                let matcher = self.handler.clone();
                tokio::spawn(async move { matcher.handle(session).await });
            }
        }
    }
}

/// Matcher 使用的 Session
#[derive(Clone)]
pub struct Session<C> {
    pub bot: WalleBot,
    pub event: walle_core::event::BaseEvent<C>,
    pub config: Arc<MatchersConfig>,
    temp_matchers: TempMatchers,
}

/// EventContent 为 MessageEvent 的 Session
pub type MessageSession = Session<MessageContent>;
/// EventContent 为 NoticeEvent 的 Session
pub type NoticeSession = Session<NoticeContent>;
/// EventContent 为 RequestEvent 的 Session
pub type RequestSession = Session<RequestContent>;
/// EventContent 为 MetaEvent 的 Session
pub type MetaSession = Session<MetaContent>;

impl<C> Session<C> {
    pub fn new(
        bot: WalleBot,
        event: BaseEvent<C>,
        config: Arc<MatchersConfig>,
        temp_plugins: TempMatchers,
    ) -> Self {
        Self {
            bot,
            event,
            config,
            temp_matchers: temp_plugins,
        }
    }
}

impl Session<EventContent> {
    pub fn as_message_session(self) -> Option<Session<MessageContent>> {
        if let Ok(event) = self.event.try_into() {
            Some(Session {
                event,
                bot: self.bot,
                config: self.config,
                temp_matchers: self.temp_matchers,
            })
        } else {
            None
        }
    }
}

impl Session<MessageContent> {
    /// 根据 Event 回复消息
    pub async fn send<T: IntoMessage>(
        &self,
        message: T,
    ) -> WalleResult<Resp<SendMessageRespContent>> {
        if let Some(group_id) = self.event.group_id() {
            self.bot
                .send_group_msg(group_id.to_string(), message.into_message())
                .await
        } else {
            self.bot
                .send_private_msg(self.event.user_id().to_string(), message.into_message())
                .await
        }
    }

    /// 根据 Event 回复消息并获取 Event
    pub async fn get<T: IntoMessage>(
        &mut self,
        message: T,
        timeout: std::time::Duration,
    ) -> WalleResult<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (name, temp) = temp_matcher(
            self.event.user_id().to_string(),
            self.event.group_id().map(ToString::to_string),
            tx,
        );
        self.temp_matchers.lock().await.insert(name.clone(), temp);
        self.send(message).await?;
        match tokio::time::timeout(timeout, rx.recv()).await {
            Ok(Some(event)) => {
                self.event = event;
            }
            _ => {
                self.temp_matchers.lock().await.remove(&name);
            }
        };
        Ok(())
    }

    pub fn message(&self) -> &Message {
        &self.event.content.message
    }

    pub fn alt_message(&self) -> &str {
        &self.event.content.alt_message
    }

    pub fn message_mut(&mut self) -> &mut Message {
        &mut self.event.content.message
    }

    pub fn alt_message_mut(&mut self) -> &mut String {
        &mut self.event.content.alt_message
    }
}

pub(crate) struct TempMatcher {
    pub tx: tokio::sync::mpsc::Sender<BaseEvent<MessageContent>>,
}

#[async_trait]
impl MatcherHandler<EventContent> for TempMatcher {
    async fn handle(&self, session: Session<EventContent>) {
        let event = session.event;
        self.tx.send(event.try_into().unwrap()).await.unwrap();
    }
}

pub(crate) fn temp_matcher(
    user_id: String,
    group_id: Option<String>,
    tx: tokio::sync::mpsc::Sender<BaseEvent<MessageContent>>,
) -> (String, Matcher<EventContent>) {
    use crate::builtin::{group_id_check, user_id_check};
    let matcher = user_id_check(&user_id).layer(TempMatcher { tx });
    (
        format!("{}-{:?}", user_id, group_id),
        if let Some(group_id) = group_id {
            Matcher::new("", "", group_id_check(group_id).layer(matcher))
        } else {
            Matcher::new("", "", matcher)
        },
    )
}

/// Matcher 构造器
pub struct MatcherBuilder<H> {
    pub name: &'static str,
    pub description: &'static str,
    pub matcher: H,
}

impl<H> MatcherBuilder<H> {
    pub fn new<C>(name: &'static str, description: &'static str, matcher: H) -> Self
    where
        H: MatcherHandler<C> + Sync + Send + 'static,
        C: Clone + Send + Sync + 'static,
    {
        Self {
            name,
            description,
            matcher,
        }
    }

    pub fn map<F, C, H1>(self, f: F) -> MatcherBuilder<H1>
    where
        F: FnOnce(H) -> H1,
        H: MatcherHandler<C> + Sync + Send + 'static,
        H1: MatcherHandler<C> + Sync + Send + 'static,
        C: Clone + Send + Sync + 'static,
    {
        MatcherBuilder {
            name: self.name,
            description: self.description,
            matcher: f(self.matcher),
        }
    }

    pub fn build<C>(self) -> Matcher<C>
    where
        H: MatcherHandler<C> + Sync + Send + 'static,
        C: Clone + Send + Sync + 'static,
    {
        Matcher::new(self.name, self.description, self.matcher)
    }
}
