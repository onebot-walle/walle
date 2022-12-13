use crate::{pre_handle_fn, MatchersConfig, PreHandler, Session, Signal};
use walle_core::{
    segment::{MessageMutExt, MsgSegmentMut},
    util::{Value, ValueMapExt},
};

pub struct StripPrefix {
    pub prefix: String,
}

impl PreHandler for StripPrefix {
    fn pre_handle(&self, session: &mut Session) -> Signal {
        if let Some(text) = session
            .event
            .extra
            .try_get_as_mut::<&mut Vec<Value>>("message")
            .ok()
            .and_then(|v| v.try_first_text_mut().ok())
        {
            if let Some(s) = text.strip_prefix(&self.prefix) {
                *text = s.to_string();
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

pub fn trim(always_match: bool) -> impl PreHandler {
    pre_handle_fn(move |session| {
        let mut sig = if always_match {
            Signal::Matched
        } else {
            Signal::NotMatch
        };
        let Ok(segs) = session
            .event
            .extra
            .try_get_as_mut::<&mut Vec<Value>>("message") else {
                return sig;
            };
        if let Ok(text) = segs.try_first_text_mut() {
            if text.starts_with(' ') {
                sig = Signal::Matched;
            }
            *text = text.trim_start().to_owned();
        }
        if let Ok(text) = segs.try_last_text_mut() {
            if text.ends_with(' ') {
                sig = Signal::Matched;
            }
            *text = text.trim_end().to_owned();
        }
        sig
    })
}

fn _mention_me(session: &mut Session) -> Signal {
    let self_id = session.event.selft().unwrap_or_default().user_id;
    let Ok(segs) = session.event.extra.try_get_as_mut::<&mut Vec<Value>>("message") else {
        return Signal::NotMatch
    };
    _mention_user(segs, self_id) | _nickname(&session.config, segs)
}

fn _mention_user(segs: &mut Vec<Value>, user_id: String) -> Signal {
    let mut mentioned_index = None;
    let Ok(seg_muts) = segs.try_as_mut() else { return Signal::NotMatch };
    for (index, seg) in seg_muts.into_iter().enumerate() {
        match seg {
            MsgSegmentMut::Mention {
                user_id: mention_id,
            } if mention_id.as_str() == &user_id => {
                mentioned_index = Some(index);
                break;
            }
            _ => {}
        }
    }
    if let Some(index) = mentioned_index {
        segs.remove(index);
        Signal::Matched
    } else {
        Signal::NotMatch
    }
}

fn _nickname(config: &MatchersConfig, segs: &mut Vec<Value>) -> Signal {
    if let Ok(text) = segs.try_first_text_mut() {
        for nickname in &config.nicknames {
            if let Some(s) = text.strip_prefix(nickname) {
                if !s.is_empty() {
                    *text = s.to_owned();
                } else {
                    segs.remove(0);
                }
                return Signal::Matched;
            }
        }
    }
    Signal::NotMatch
}

pub fn mention_user(user_id: String) -> impl PreHandler {
    pre_handle_fn(move |session| {
        let Ok(segs) = session.event.extra.try_get_as_mut::<&mut Vec<Value>>("message") else {
            return Signal::NotMatch
        };
        _mention_user(segs, user_id.clone())
    })
}

pub fn mention_me() -> impl PreHandler {
    pre_handle_fn(_mention_me)
}

pub fn to_me() -> impl PreHandler {
    pre_handle_fn(|session| {
        let sig = if &session.event.detail_type == "private" {
            Signal::Matched
        } else {
            Signal::NotMatch
        };
        sig | _mention_me(session)
    })
}
