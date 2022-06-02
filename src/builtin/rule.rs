use crate::{rule_fn, MessageContent, Rule, Session};
use walle_core::{EventContent, MessageEventDetail, MessageSegment};

pub struct UserIdChecker {
    pub user_id: String,
}

impl Rule<MessageContent> for UserIdChecker {
    fn rule(&self, session: &Session<MessageContent>) -> bool {
        session.event.user_id() == self.user_id
    }
}

impl Rule<EventContent> for UserIdChecker {
    fn rule(&self, session: &Session<EventContent>) -> bool {
        if let EventContent::Message(ref c) = session.event.content {
            c.user_id == self.user_id
        } else {
            false
        }
    }
}

pub fn user_id_check<S>(user_id: S) -> UserIdChecker
where
    S: ToString,
{
    UserIdChecker {
        user_id: user_id.to_string(),
    }
}

pub struct GroupIdChecker {
    pub group_id: String,
}

impl Rule<MessageContent> for GroupIdChecker {
    fn rule(&self, session: &Session<MessageContent>) -> bool {
        session.event.group_id() == Some(&self.group_id)
    }
}

impl Rule<EventContent> for GroupIdChecker {
    fn rule(&self, session: &Session<EventContent>) -> bool {
        if let EventContent::Message(ref c) = session.event.content {
            if c.detail.group_id() == Some(&self.group_id) {
                return true;
            }
        }
        false
    }
}

pub fn group_id_check<S>(group_id: S) -> GroupIdChecker
where
    S: ToString,
{
    GroupIdChecker {
        group_id: group_id.to_string(),
    }
}

pub fn start_with(pat: &str) -> impl Rule<MessageContent> {
    let word = pat.to_string();
    rule_fn(move |session: &Session<MessageContent>| -> bool {
        session.event.content.alt_message.starts_with(&word)
    })
}

fn _mention_me(session: &Session<MessageContent>) -> bool {
    if let Some(MessageSegment::Text { text, .. }) = session.message().first() {
        for nickname in &session.config.nicknames {
            if text.starts_with(nickname) {
                return true;
            }
        }
    }
    for seg in session.event.content.message.iter() {
        if let MessageSegment::Mention { user_id, .. } = seg {
            if user_id == &session.bot.self_id {
                return true;
            }
        }
    }
    false
}

pub fn mention_me() -> impl Rule<MessageContent> {
    rule_fn(_mention_me)
}

pub fn to_me() -> impl Rule<MessageContent> {
    rule_fn(|session: &Session<MessageContent>| {
        if let MessageEventDetail::Group { .. } = session.event.content.detail {
            _mention_me(session)
        } else {
            true
        }
    })
}
