use walle_core::{event::BaseEvent, prelude::async_trait};

mod handle;
mod hook;
mod matchers;
mod pre_handle;
mod rule;
mod session;

pub use handle::*;
pub use hook::*;
pub use matchers::*;
pub use pre_handle::*;
pub use rule::*;
pub use session::*;

struct TempMatcher<T, D, S, P, I> {
    pub tx: tokio::sync::mpsc::UnboundedSender<BaseEvent<T, D, S, P, I>>,
}

#[async_trait]
impl<T, D, S, P, I> MatcherHandler<T, D, S, P, I> for TempMatcher<T, D, S, P, I>
where
    T: Send + 'static,
    D: Send + 'static,
    S: Send + 'static,
    P: Send + 'static,
    I: Send + 'static,
{
    async fn handle(&self, session: Session<T, D, S, P, I>) {
        self.tx.send(session.event).ok();
    }
}
