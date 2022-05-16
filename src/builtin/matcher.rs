use walle_core::MessageContent;

use super::{mention_me, remove_mention_me, start_with, strip_prefix, strip_whitespace};
use crate::{MatcherHandler, MatcherHandlerExt};

pub fn on_command<H>(command: &str, handler: H) -> impl MatcherHandler<MessageContent>
where
    H: MatcherHandler<MessageContent> + Sync,
{
    handler
        .rule(start_with(command))
        .pre_handle(strip_prefix(command))
        .pre_handle(strip_whitespace())
}

pub fn on_start_with<H>(pat: &str, handler: H) -> impl MatcherHandler<MessageContent>
where
    H: MatcherHandler<MessageContent> + Sync,
{
    handler.rule(start_with(pat))
}

pub fn on_mention_me<H>(handler: H) -> impl MatcherHandler<MessageContent>
where
    H: MatcherHandler<MessageContent> + Sync,
{
    handler.rule(mention_me()).pre_handle(remove_mention_me())
}
