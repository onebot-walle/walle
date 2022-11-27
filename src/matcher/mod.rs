use walle_core::prelude::{async_trait, Event};

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

struct TempMatcher {
    pub tx: tokio::sync::mpsc::UnboundedSender<Event>,
}

#[async_trait]
impl MatcherHandler for TempMatcher {
    async fn handle(&self, session: Session) -> Signal {
        self.tx.send(session.event).ok();
        Signal::MatchAndBlock
    }
}
