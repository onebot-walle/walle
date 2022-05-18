use std::{future::Future, pin::Pin};

use super::on_command;
use crate::{handler_fn, Matcher, MatcherHandler, Session};
use async_trait::async_trait;
use walle_core::MessageContent;

pub struct Echo;

#[async_trait]
impl MatcherHandler<MessageContent> for Echo {
    async fn handle(&self, session: Session<MessageContent>) {
        let _ = session.send(session.event.message().clone()).await;
    }
}

pub fn echo() -> Matcher<MessageContent> {
    Matcher::new("echo", "echo description", on_command("echo", Echo))
}

fn _echo2(
    mut session: Session<MessageContent>,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    Box::pin(async move {
        let _ = session
            .get("input message", std::time::Duration::from_secs(10))
            .await;
        let _ = session.send(session.event.message().clone()).await;
    })
}

pub fn echo2() -> Matcher<MessageContent> {
    Matcher::new(
        "echo2",
        "echo2 description",
        on_command("echo2", handler_fn(_echo2)),
    )
}
