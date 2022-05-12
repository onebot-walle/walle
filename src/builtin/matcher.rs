use walle_core::MessageContent;

use super::{start_with, strip_prefix};
use crate::{MatcherHandler, MatcherHandlerExt};

pub fn on_command<H>(command: &str, handler: H) -> impl MatcherHandler<MessageContent>
where
    H: MatcherHandler<MessageContent> + Sync,
{
    handler
        .rule(start_with(command))
        .pre_handle(strip_prefix(command))
}
