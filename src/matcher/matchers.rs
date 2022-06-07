use super::{Matcher, Session};
use crate::{MatcherConfig, MatchersHook, MessageContent};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use walle_core::app::StandardArcBot;
use walle_core::{
    BaseEvent, EventContent, EventHandler, MetaContent, NoticeContent, RequestContent, Resps,
    StandardAction, StandardEvent,
};

pub(crate) type TempMatchers = Arc<Mutex<HashMap<String, Matcher<EventContent>>>>;

pub type MessageMatcher = Matcher<MessageContent>;
pub type NoticeMatcher = Matcher<NoticeContent>;
pub type RequestMatcher = Matcher<RequestContent>;
pub type MetaMatcher = Matcher<MetaContent>;

#[derive(Default)]
pub struct Matchers {
    pub message: Vec<Matcher<MessageContent>>,
    pub notice: Vec<Matcher<NoticeContent>>,
    pub request: Vec<Matcher<RequestContent>>,
    pub meta: Vec<Matcher<MetaContent>>,
    pub config: Arc<MatcherConfig>,
    temp: TempMatchers,
    hooks: Vec<Box<dyn MatchersHook + Send + 'static>>,
}

impl Matchers {
    pub fn new(config: MatcherConfig) -> Self {
        Self {
            config: Arc::new(config),
            ..Default::default()
        }
    }
    pub fn add_message_matcher(mut self, plugin: Matcher<MessageContent>) -> Self {
        self.message.push(plugin);
        self
    }
    pub fn add_notice_matcher(mut self, plugin: Matcher<NoticeContent>) -> Self {
        self.notice.push(plugin);
        self
    }
    pub fn add_request_matcher(mut self, plugin: Matcher<RequestContent>) -> Self {
        self.request.push(plugin);
        self
    }
    pub fn add_meta_matcher(mut self, plugin: Matcher<MetaContent>) -> Self {
        self.meta.push(plugin);
        self
    }
    async fn _event_call<C>(
        &self,
        bot: &StandardArcBot,
        event: StandardEvent,
        matchers: &Vec<Matcher<C>>,
    ) -> Option<StandardEvent>
    where
        BaseEvent<C>: TryFrom<StandardEvent, Error = StandardEvent>,
        C: Clone + Send + Sync + 'static,
    {
        match event.try_into() {
            Ok(event) => {
                let session =
                    Session::new(bot.clone(), event, self.config.clone(), self.temp.clone());
                for matcher in matchers {
                    matcher.call(&session).await;
                }
                None
            }
            Err(event) => Some(event),
        }
    }
    async fn on_start(&self) {
        for hook in &self.hooks {
            hook.on_start().await;
        }
    }
    async fn on_finish(&self) {
        for hook in &self.hooks {
            hook.on_finish().await;
        }
    }
}

#[async_trait]
impl EventHandler<StandardEvent, StandardAction, Resps<StandardEvent>> for Matchers {
    async fn handle(&self, bot: StandardArcBot, event: StandardEvent) {
        let session = Session::new(bot, event, self.config.clone(), self.temp.clone());
        if let Some(p) = {
            let mut temp_plugins = self.temp.lock().await;
            let mut found: Option<String> = None;
            for (k, plugin) in temp_plugins.iter() {
                if plugin.handler._match(&session) {
                    found = Some(k.clone());
                    break;
                }
            }
            found.and_then(|i| temp_plugins.remove(&i))
        } {
            p.handler.handle(session).await;
            return;
        }
        let (bot, event) = (session.bot, session.event);
        self.on_start().await;
        if let Some(event) = self._event_call(&bot, event, &self.message).await {
            if let Some(event) = self._event_call(&bot, event, &self.meta).await {
                if let Some(event) = self._event_call(&bot, event, &self.notice).await {
                    self._event_call(&bot, event, &self.request).await;
                }
            }
        } // ugly..
        self.on_finish().await;
    }
}
