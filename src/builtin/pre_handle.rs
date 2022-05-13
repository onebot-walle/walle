use walle_core::{MessageContent, MessageSegment};

use crate::{pre_handle_fn, PreHandler, Session};

pub struct StripPrefix {
    pub prefix: String,
}

impl PreHandler<MessageContent> for StripPrefix {
    fn pre_handle(&self, session: &mut Session<MessageContent>) {
        if let Some(s) = session.alt_message().strip_prefix(&self.prefix) {
            *session.alt_message_mut() = s.to_string();
        }
        if let Some(MessageSegment::Text { text, .. }) = session.message_mut().first_mut() {
            if let Some(s) = text.strip_prefix(&self.prefix) {
                *text = s.to_string();
            }
        }
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
        let mut alt = session.alt_message();
        while let Some(s) = alt.strip_prefix(" ") {
            alt = s;
        }
        while let Some(s) = alt.strip_suffix(" ") {
            alt = s;
        }
        *session.alt_message_mut() = alt.to_string();
        if let Some(MessageSegment::Text { text, .. }) = session.message_mut().first_mut() {
            let mut str: &str = text;
            while let Some(s) = str.strip_prefix(" ") {
                str = s;
            }
            *text = str.to_string();
        }
        if let Some(MessageSegment::Text { text, .. }) = session.message_mut().last_mut() {
            let mut str: &str = text;
            while let Some(s) = str.strip_suffix(" ") {
                str = s;
            }
            *text = str.to_string();
        }
    })
}
