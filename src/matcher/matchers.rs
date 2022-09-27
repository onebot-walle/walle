use super::RawMatcherHandler;
use crate::{ActionCaller, Signal};
use crate::{MatchersConfig, MatchersHook};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use tracing::info;
use walle_core::prelude::WalleError;
use walle_core::{
    action::Action, error::WalleResult, event::Event, resp::Resp, ActionHandler, EventHandler,
    OneBot,
};

pub type Matcher = Box<dyn RawMatcherHandler + Send + Sync + 'static>;
pub type TempMatchers = Arc<Mutex<HashMap<String, Matcher>>>;

#[derive(Default)]
pub struct Matchers {
    pub inner: Vec<Matcher>,
    pub config: RwLock<Arc<MatchersConfig>>,
    temps: TempMatchers,
    hooks: Vec<Box<dyn MatchersHook + Send + 'static>>,
    ob: RwLock<Option<Arc<dyn ActionCaller + Send + 'static>>>,
}

impl Matchers {
    pub fn add_matcher(mut self, matcher: Matcher) -> Self {
        self.inner.push(matcher);
        self
    }
    async fn temp_call(
        &self,
        event: &Event,
        config: &Arc<MatchersConfig>,
        ob: &Arc<dyn ActionCaller + Send + 'static>,
    ) -> bool {
        let mut matched_temp_key: Option<String> = None;
        let mut temps = self.temps.lock().await;
        for temp in temps.iter() {
            if temp.1.call(event.clone(), &config, ob, &self.temps).await != Signal::NotMatch {
                matched_temp_key = Some(temp.0.to_owned());
                break;
            }
        }
        if let Some(key) = matched_temp_key {
            temps.remove(&key);
            true
        } else {
            false
        }
    }
}

#[async_trait]
impl EventHandler<Event, Action, Resp> for Matchers {
    type Config = MatchersConfig;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: MatchersConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        *self.ob.write().await = Some(Arc::new(ob.clone()));
        *self.config.write().await = Arc::new(config);
        let ob = self.ob.read().await.clone().unwrap();
        for hook in self.hooks.iter() {
            hook.on_start(&ob).await
        }
        Ok(vec![])
    }
    async fn call<AH, EH>(&self, event: Event, _: &Arc<OneBot<AH, EH>>) -> WalleResult<()>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        use walle_core::alt::ColoredAlt;
        if event.ty.as_str() != "meta" {
            info!(target: "Walle", "{}", event.colored_alt());
        }
        let ob: Arc<dyn ActionCaller + Send + 'static> =
            self.ob.read().await.clone().ok_or(WalleError::NotStarted)?;
        let config = self.config.read().await.clone();
        if self.temp_call(&event, &config, &ob).await {
            return Ok(());
        }
        for matcher in &self.inner {
            if matcher.call(event.clone(), &config, &ob, &self.temps).await == Signal::MatchAndBlock
            {
                return Ok(());
            }
        }
        Ok(())
    }
    async fn shutdown(&self) {
        let ob = self.ob.read().await.clone().unwrap();
        for hook in self.hooks.iter() {
            hook.on_shutdown(&ob).await;
        }
        *self.ob.write().await = None;
    }
}
