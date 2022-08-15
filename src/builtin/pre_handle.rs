use crate::{pre_handle_fn, PreHandler, Session, Signal};
use walle_core::{
    event::{BaseEvent, Message},
    prelude::MessageSegment,
    util::Value,
};

fn text_mut(seg: &mut MessageSegment) -> Option<&mut String> {
    if seg.ty.as_str() == "text" {
        seg.data.get_mut("text").and_then(|v| v.as_str_mut())
    } else {
        None
    }
}

fn first_text_mut<D, S, P, I>(event: &mut BaseEvent<Message, D, S, P, I>) -> Option<&mut String> {
    event.ty.message.first_mut().and_then(text_mut)
}

fn last_text_mut<D, S, P, I>(event: &mut BaseEvent<Message, D, S, P, I>) -> Option<&mut String> {
    event.ty.message.last_mut().and_then(text_mut)
}

pub struct StripPrefix {
    pub prefix: String,
}

impl<D, S, P, I> PreHandler<Message, D, S, P, I> for StripPrefix {
    fn pre_handle(&self, session: &mut Session<Message, D, S, P, I>) -> Signal {
        if let Some(text) = first_text_mut(&mut session.event) {
            if let Some(s) = text.strip_prefix(&self.prefix) {
                *text = s.to_string();
                session.update_alt();
                return Signal::Matched;
            }
        }
        Signal::NotMatch
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

pub fn strip_whitespace<D, S, P, I>() -> impl PreHandler<Message, D, S, P, I> {
    pre_handle_fn(|session| {
        let mut sig = Signal::NotMatch;
        if let Some(text) = first_text_mut(&mut session.event) {
            while let Some(s) = text.strip_prefix(' ') {
                *text = s.to_string();
                sig = Signal::Matched;
            }
        }
        if let Some(text) = last_text_mut(&mut session.event) {
            while let Some(s) = text.strip_suffix(' ') {
                *text = s.to_string();
                sig = Signal::Matched;
            }
        }
        if sig != Signal::NotMatch {
            session.update_alt();
        }
        sig
    })
}

fn _mention_me<D, S, P, I>(session: &mut Session<Message, D, S, P, I>) -> Signal {
    let segments = &mut session.event.ty.message;
    let self_id = Value::Str(session.event.ty.selft.user_id.clone());
    for i in 0..segments.len() {
        let seg = segments.get(i).unwrap();
        if seg.ty.as_str() == "mention" {
            if seg.data.get("user_id") == Some(&self_id) {
                session.update_alt();
                return Signal::Matched;
            }
        }
    }
    Signal::NotMatch
}

pub fn mention_me<D, S, P, I>() -> impl PreHandler<Message, D, S, P, I> {
    pre_handle_fn(_mention_me)
}

pub fn to_me<D, S, P, I>() -> impl PreHandler<Message, D, S, P, I> {
    pre_handle_fn(|session| {
        if let Some(text) = first_text_mut(&mut session.event) {
            for nickname in &session.config.nicknames {
                if let Some(s) = text.strip_prefix(nickname) {
                    *text = s.to_string();
                    session.update_alt();
                    return Signal::Matched;
                }
            }
        }
        _mention_me(session)
    })
}
