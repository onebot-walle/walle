use std::sync::Arc;

use crate::ActionCaller;

#[async_trait::async_trait]
pub trait MatchersHook: Sync {
    async fn on_start(&self, _caller: &Arc<dyn ActionCaller + Send + 'static>) {}
    async fn on_shutdown(&self, _caller: &Arc<dyn ActionCaller + Send + 'static>) {}
}
