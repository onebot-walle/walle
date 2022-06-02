use super::*;
use crate::{MatcherHandler, MatcherHandlerExt, MessageContent};

pub fn on_command<H>(command: &str, handler: H) -> impl MatcherHandler<MessageContent>
where
    H: MatcherHandler<MessageContent> + Sync,
{
    handler
        .pre_handle_before(strip_whitespace(), false)
        .pre_handle_before(strip_prefix(command), true)
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
    handler
        .pre_handle_before(strip_whitespace(), false)
        .pre_handle_before(remove_mention_me(), true)
}

pub fn on_to_me<H>(handler: H) -> impl MatcherHandler<MessageContent>
where
    H: MatcherHandler<MessageContent> + Sync,
{
    handler
        .pre_handle_before(strip_whitespace(), false)
        .pre_handle_before(remote_to_me(), true)
}
