use super::_MatcherHandler;
use crate::{ActionCaller, Signal};
use crate::{MatchersConfig, MatchersHook};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::{sync::RwLock, task::JoinHandle};
use walle_core::prelude::WalleError;
use walle_core::{
    action::Action, error::WalleResult, event::Event, resp::Resp, ActionHandler, EventHandler,
    OneBot,
};

pub type Matcher = Box<dyn _MatcherHandler + Send + Sync + 'static>;
pub(crate) type TempMatchers = Arc<DashMap<String, Matcher>>;

pub struct Matchers {
    pub inner: Vec<Matcher>,
    pub config: Arc<MatchersConfig>,
    temps: TempMatchers,
    hooks: Vec<Box<dyn MatchersHook + Send + 'static>>,
    ob: tokio::sync::RwLock<Option<Arc<dyn ActionCaller + Send + 'static>>>,
}

impl Matchers {
    pub fn new(config: MatchersConfig) -> Self {
        Self {
            config: Arc::new(config),
            inner: Vec::new(),
            temps: Arc::default(),
            hooks: Vec::new(),
            ob: RwLock::default(),
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
        ob: &Arc<OneBot<AH, EH, 12>>,
        _config: (),
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
    {
        *self.ob.write().await = Some(ob.clone());
        Ok(vec![])
    }
    async fn call(&self, event: Event) -> WalleResult<()> {
        let ob: Arc<dyn ActionCaller + Send + 'static> =
            self.ob.read().await.clone().ok_or(WalleError::NotStarted)?;
        if let Some(k) = self.temps.iter().find_map(|i| {
            if i.value()
                .call(event.clone(), &self.config, &ob, &self.temps)
                != Signal::NotMatch
            {
                Some(i.key().to_string())
            } else {
                None
            }
        }) {
            self.temps.remove(&k);
            return Ok(());
        }
        for matcher in &self.inner {
            if matcher.call(event.clone(), &self.config, &ob, &self.temps) == Signal::MatchAndBlock
            {
                return Ok(());
            }
        }
        Ok(())
    }
    async fn shutdown(&self) {
        *self.ob.write().await = None;
    }
}
