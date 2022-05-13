use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use walle_core::app::StandardArcBot;
use walle_core::{
    BaseEvent, EventContent, IntoMessage, Message, MessageContent, Resps, WalleResult,
};

mod matchers;
mod pre_handle;
mod rule;

pub use matchers::*;
pub use pre_handle::*;
pub use rule::*;

#[async_trait]
pub trait MatcherHandler<C>: Sync {
    fn _match(&self, _session: &Session<C>) -> bool {
        true
    }
    /// if matched will be called before handle, should never fail
    fn _pre_handle(&self, _session: &mut Session<C>) {}
    async fn handle(&self, session: Session<C>);
}

pub trait MatcherHandlerExt<C>: MatcherHandler<C> {
    fn rule<R>(self, rule: R) -> LayeredRule<R, Self>
    where
        Self: Sized,
        R: Rule<C>,
    {
        rule.layer(self)
    }
    fn pre_handle<P>(self, pre: P) -> LayeredPreHandler<P, Self>
    where
        Self: Sized,
        P: PreHandler<C>,
    {
        pre.layer(self)
    }
}

impl<C, H: MatcherHandler<C>> MatcherHandlerExt<C> for H {}

pub struct HandlerFn<I>(I);

pub fn handler_fn<I, C, Fut>(inner: I) -> HandlerFn<I>
where
    I: Fn(Session<C>) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
    C: Sync + Send + 'static,
{
    HandlerFn(inner)
}

impl<C, I, Fut> MatcherHandler<C> for HandlerFn<I>
where
    C: Sync + Send + 'static,
    I: Fn(Session<C>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn handle<'a, 'b>(
        &'a self,
        session: Session<C>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
        Self: 'b,
    {
        Box::pin(self.0(session))
    }
}

#[derive(Clone)]
pub struct Session<C> {
    pub bot: walle_core::app::StandardArcBot,
    pub event: walle_core::event::BaseEvent<C>,
    temp_matchers: TempMatchers,
}

impl<C> Session<C> {
    pub fn new(bot: StandardArcBot, event: BaseEvent<C>, temp_plugins: TempMatchers) -> Self {
        Self {
            bot,
            event,
            temp_matchers: temp_plugins,
        }
    }

    pub fn replace_evnet(&mut self, event: BaseEvent<C>) {
        self.event = event;
    }
}

impl Session<EventContent> {
    pub fn as_message_session(self) -> Option<Session<MessageContent>> {
        if let Ok(event) = self.event.try_into() {
            Some(Session {
                bot: self.bot,
                event,
                temp_matchers: self.temp_matchers,
            })
        } else {
            None
        }
    }
}

impl Session<MessageContent> {
    pub async fn send<T: IntoMessage>(&self, message: T) -> WalleResult<Resps> {
        if let Some(group_id) = self.event.group_id() {
            self.bot
                .send_group_message(group_id.to_string(), message.into_message())
                .await
        } else {
            self.bot
                .send_private_message(self.event.user_id().to_string(), message.into_message())
                .await
        }
    }

    pub async fn get<T: IntoMessage>(
        &mut self,
        message: T,
        timeout: std::time::Duration,
    ) -> WalleResult<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (name, temp) = TempMathcer::new(
            self.event.user_id().to_string(),
            self.event.group_id().map(ToString::to_string),
            tx,
        );
        self.temp_matchers.lock().await.insert(name.clone(), temp);
        self.send(message).await?;
        match tokio::time::timeout(timeout, rx.recv()).await {
            Ok(Some(event)) => {
                self.replace_evnet(event);
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

pub struct TempMathcer {
    pub tx: tokio::sync::mpsc::Sender<BaseEvent<MessageContent>>,
}

#[async_trait]
impl MatcherHandler<EventContent> for TempMathcer {
    async fn handle(&self, session: Session<EventContent>) {
        let event = session.event;
        self.tx.send(event.try_into().unwrap()).await.unwrap();
    }
}

impl TempMathcer {
    pub fn new(
        user_id: String,
        group_id: Option<String>,
        tx: tokio::sync::mpsc::Sender<BaseEvent<MessageContent>>,
    ) -> (String, Matcher<EventContent>) {
        use crate::builtin::{group_id_check, user_id_check};
        let name = format!("{}-{:?}", user_id, group_id);
        let matcher = user_id_check(user_id).layer(Self { tx });
        (
            name.clone(),
            if let Some(group_id) = group_id {
                Matcher::new(
                    name,
                    "".to_string(),
                    group_id_check(group_id).layer(matcher),
                )
            } else {
                Matcher::new(name, "".to_string(), matcher)
            },
        )
    }
}
