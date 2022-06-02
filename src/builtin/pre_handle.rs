use walle_core::{MessageAlt, MessageEventDetail, MessageSegment};

use crate::{pre_handle_fn, MessageContent, PreHandler, Session};

fn update_alt(session: &mut Session<MessageContent>) -> bool {
    *session.alt_message_mut() = session.message().alt();
    return true;
}

pub struct StripPrefix {
    pub prefix: String,
}

impl PreHandler<MessageContent> for StripPrefix {
    fn pre_handle(&self, session: &mut Session<MessageContent>) -> bool {
        if let Some(MessageSegment::Text { text, .. }) = session.message_mut().first_mut() {
            if let Some(s) = text.strip_prefix(&self.prefix) {
                *text = s.to_string();
                return update_alt(session);
            }
        }
        false
    }
}

pub fn strip_prefix<S>(prefix: S) -> StripPrefix
where
    S: ToString,
{
    StripPrefix {
        prefix: prefix.to_string(),
    }
}

pub fn strip_whitespace() -> impl PreHandler<MessageContent> {
    pre_handle_fn(|session| {
        if let Some(MessageSegment::Text { text, .. }) = session.message_mut().first_mut() {
            let mut str: &str = text;
            while let Some(s) = str.strip_prefix(' ') {
                str = s;
            }
            *text = str.to_string();
            return update_alt(session);
        }
        if let Some(MessageSegment::Text { text, .. }) = session.message_mut().last_mut() {
            let mut str: &str = text;
            while let Some(s) = str.strip_suffix(' ') {
                str = s;
            }
            *text = str.to_string();
            return update_alt(session);
        }
        false
    })
}

fn _remove_mention_me(session: &mut Session<MessageContent>) -> bool {
    for i in 0..session.message().len() {
        if let MessageSegment::Mention { ref user_id, .. } = session.message()[i] {
            if user_id == &session.event.self_id {
                session.message_mut().remove(i);
                return update_alt(session);
            }
        }
    }
    false
}

pub fn remove_mention_me() -> impl PreHandler<MessageContent> {
    pre_handle_fn(_remove_mention_me)
}

fn _remove_nickname(session: &mut Session<MessageContent>) -> bool {
    let mut s = String::default();
    if let Some(MessageSegment::Text { text, .. }) = session.message().first() {
        for nickname in &session.config.nicknames {
            if let Some(striped) = text.strip_prefix(nickname) {
                s.push_str(striped);
            }
        }
    }
    match s.as_str() {
        "" => {
            session.message_mut().remove(0);
            return update_alt(session);
        }
        _ => {
            if let Some(MessageSegment::Text { text, .. }) = session.message_mut().first_mut() {
                *text = s;
                return update_alt(session);
            }
        }
    }
    false
}

pub fn remove_nickname() -> impl PreHandler<MessageContent> {
    pre_handle_fn(_remove_nickname)
}

pub fn remove_to_me() -> impl PreHandler<MessageContent> {
    pre_handle_fn(|session: &mut Session<MessageContent>| {
        if let MessageEventDetail::Private { .. } = session.event.content.detail {
            return true;
        } else {
            return _remove_nickname(session) || _remove_mention_me(session);
        }
    })
}
