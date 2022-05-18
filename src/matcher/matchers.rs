use crate::{MatcherConfig, MatcherHandler, Session};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use walle_core::app::StandardArcBot;
use walle_core::{
    EventContent, EventHandler, MessageContent, MetaContent, NoticeContent, RequestContent, Resps,
    StandardAction, StandardEvent,
};

pub(crate) type TempMatchers = Arc<Mutex<HashMap<String, Matcher<EventContent>>>>;

#[derive(Default)]
pub struct Matchers {
    pub message: Vec<Matcher<MessageContent>>,
    pub notice: Vec<Matcher<NoticeContent>>,
    pub request: Vec<Matcher<RequestContent>>,
    pub meta: Vec<Matcher<MetaContent>>,
    pub config: Arc<MatcherConfig>,
    temp: TempMatchers,
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
}

#[async_trait]
impl EventHandler<StandardEvent, StandardAction, Resps> for Matchers {
    async fn handle(&self, bot: StandardArcBot, event: StandardEvent) {
        let session = Session::new(bot, event, self.config.clone(), self.temp.clone());
        if let Some(p) = {
            let mut temp_plugins = self.temp.lock().await;
            let mut found: Option<String> = None;
            for (k, plugin) in temp_plugins.iter() {
                if plugin.matcher._match(&session) {
                    found = Some(k.clone());
                    break;
                }
            }
            found.and_then(|i| temp_plugins.remove(&i))
        } {
            p.matcher.handle(session).await;
            return;
        }
        let (bot, event) = (session.bot, session.event);
        if let Ok(event) = event.try_into() {
            let session = Session::new(bot, event, self.config.clone(), self.temp.clone());
            for plugin in &self.message {
                plugin.handle(&session).await;
            }
        }
    }
}

#[derive(Clone)]
pub struct Matcher<C> {
    pub name: String,
    pub description: String,
    pub matcher: Arc<dyn MatcherHandler<C> + Sync + Send + 'static>,
}

impl<C> Matcher<C>
where
    C: Clone + Send + Sync + 'static,
{
    pub fn new<T0: ToString, T1: ToString>(
        name: T0,
        description: T1,
        matcher: impl MatcherHandler<C> + Sync + Send + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            matcher: Arc::new(matcher),
        }
    }

    pub fn new_with<T0, T1, H0, H1, F>(name: T0, description: T1, matcher: H0, f: F) -> Self
    where
        T0: ToString,
        T1: ToString,
        H0: MatcherHandler<C> + Sync + Send + 'static,
        H1: MatcherHandler<C> + Sync + Send + 'static,
        F: FnOnce(H0) -> H1,
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            matcher: Arc::new(f(matcher)),
        }
    }

    pub async fn handle(&self, session: &Session<C>) {
        if self.matcher._match(session) {
            let mut session = session.clone();
            if self.matcher._pre_handle(&mut session) {
                let matcher = self.matcher.clone();
                tokio::spawn(async move { matcher.handle(session).await });
            }
        }
    }
}
