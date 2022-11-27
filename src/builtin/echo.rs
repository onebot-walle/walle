use crate::{matcher, on_command};
use crate::{MatcherHandler, Session};

on_command!(Echo, "echo", crate);

pub fn echo() -> impl MatcherHandler {
    matcher(|Echo(segs): Echo, session: Session| async move {
        session.reply(segs).await.ok();
    })
}
