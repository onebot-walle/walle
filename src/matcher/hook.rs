#[async_trait::async_trait]
pub trait MatchersHook: Sync {
    async fn on_start(&self) {}
    async fn on_finish(&self) {}
}
