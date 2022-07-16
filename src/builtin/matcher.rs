use super::*;
use crate::{MatcherHandler, PreHandler, Rule};
use walle_core::event::Message;

pub fn on_command<H, D, S, P, I>(
    command: &str,
    handler: H,
) -> impl MatcherHandler<Message, D, S, P, I>
where
    H: MatcherHandler<Message, D, S, P, I> + Sync,
{
    strip_whitespace()
        .with(strip_prefix(command))
        .layer(handler)
}

pub fn on_start_with<H, D, S, P, I>(
    pat: &str,
    handler: H,
) -> impl MatcherHandler<Message, D, S, P, I>
where
    H: MatcherHandler<Message, D, S, P, I> + Sync,
{
    start_with(pat).layer(handler)
}

pub fn on_mention_me<H, D, S, P, I>(handler: H) -> impl MatcherHandler<Message, D, S, P, I>
where
    H: MatcherHandler<Message, D, S, P, I> + Sync,
{
    strip_whitespace().with(mention_me()).layer(handler)
}

pub fn on_to_me<H, D, S, P, I>(handler: H) -> impl MatcherHandler<Message, D, S, P, I>
where
    H: MatcherHandler<Message, D, S, P, I> + Sync,
{
    strip_whitespace().with(to_me()).layer(handler)
}
