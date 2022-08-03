use std::{future::Future, pin::Pin};

use super::on_command;
use crate::{MatcherHandler, ReplyAbleSession, Session};
use async_trait::async_trait;
use walle_core::event::{Message, MessageDeatilTypes};

pub struct Echo;

#[async_trait]
impl MatcherHandler<Message, MessageDeatilTypes> for Echo {
    async fn handle(&self, session: Session<Message, MessageDeatilTypes>) {
        println!("{:?}", session.event);
        let _ = session.send(session.message().clone()).await;
    }
}

pub fn echo() -> impl MatcherHandler<Message, MessageDeatilTypes> {
    on_command("echo", Echo)
}

fn _echo2(
    mut session: Session<Message, MessageDeatilTypes>,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    Box::pin(async move {
        let _ = session.get("input message", None).await;
        let _ = session.send(session.event.ty.message.clone()).await;
    })
}
