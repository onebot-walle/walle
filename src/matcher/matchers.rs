use super::_MatcherHandler;
use crate::Signal;
use crate::{MatchersConfig, MatchersHook};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;
use walle_core::{
    action::Action, error::WalleResult, event::Event, resp::Resp, ActionHandler, EventHandler,
    OneBot,
};

pub type Matcher = Box<dyn _MatcherHandler + Send + Sync + 'static>;
pub(crate) type TempMatchers = Arc<DashMap<String, Matcher>>;

#[derive(Default)]
pub struct Matchers {
    pub inner: Vec<Matcher>,
    pub config: Arc<MatchersConfig>,
    temps: TempMatchers,
    hooks: Vec<Box<dyn MatchersHook + Send + 'static>>,
}

impl Matchers {
    pub fn new(config: MatchersConfig) -> Self {
        Self {
            config: Arc::new(config),
            ..Default::default()
        }
    }
    pub fn add_matcher(mut self, matcher: Matcher) -> Self {
        self.inner.push(matcher);
        self
    }
}

#[async_trait]
impl EventHandler<Event, Action, Resp, 12> for Matchers {
    type Config = ();
    async fn start<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH, 12>>,
        _config: (),
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
    {
        Ok(vec![])
    }
    async fn call<AH, EH>(&self, event: Event, ob: &Arc<OneBot<AH, EH, 12>>)
    where
        AH: ActionHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
    {
        if let Some(k) = self.temps.iter().find_map(|i| {
            if i.value().call(event, &self.config, ob, &self.temps) != Signal::NotMatch {
                Some(i.key().to_string())
            } else {
                None
            }
        }) {
            self.temps.remove(&k);
            return;
        }
        for matcher in &self.inner {
            if matcher.call(event, &self.config, ob, &self.temps) == Signal::MatchAndBlock {
                break;
            }
        }
    }
}
