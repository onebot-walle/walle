use crate::{pre_handle_fn, PreHandler, Session, Signal};
use walle_core::{
    prelude::Event,
    segment::MsgSegmentMut,
    util::{Value, ValueMapExt},
};

fn seg_mut_iter<'a>(vec_value: &'a mut Vec<Value>) -> impl Iterator<Item = MsgSegmentMut<'a>> {
    vec_value.into_iter().filter_map(|v| {
        if let Ok(seg) = v.try_as_mut() {
            Some(seg)
        } else {
            None
        }
    })
}

// fn seg_text_mut_iter<'a>(vec_value: &'a mut Vec<Value>) -> impl Iterator<Item = &'a mut String> {
//     seg_mut_iter(vec_value).filter_map(|seg| {
//         if let MsgSegmentMut::Text { text } = seg {
//             Some(text)
//         } else {
//             None
//         }
//     })
// }

fn first_text_mut(event: &mut Event) -> Option<&mut String> {
    if let Some(MsgSegmentMut::Text { text, .. }) = event
        .extra
        .try_get_as_mut::<&mut Vec<Value>>("message")
        .ok()
        .and_then(|v| v.first_mut())
        .and_then(|v| v.try_as_mut::<MsgSegmentMut<'_>>().ok())
    {
        Some(text)
    } else {
        None
    }
}

fn last_text_mut(event: &mut Event) -> Option<&mut String> {
    if let Some(MsgSegmentMut::Text { text, .. }) = event
        .extra
        .try_get_as_mut::<&mut Vec<Value>>("message")
        .ok()
        .and_then(|v| v.last_mut())
        .and_then(|v| v.try_as_mut::<MsgSegmentMut<'_>>().ok())
    {
        Some(text)
    } else {
        None
    }
}

pub struct StripPrefix {
    pub prefix: String,
}

impl PreHandler for StripPrefix {
    fn pre_handle(&self, session: &mut Session) -> Signal {
        if let Some(text) = first_text_mut(&mut session.event) {
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

pub fn strip_whitespace(always_match: bool) -> impl PreHandler {
    pre_handle_fn(move |session| {
        let mut sig = if always_match {
            Signal::Matched
        } else {
            Signal::NotMatch
        };
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
        sig
    })
}

fn _mention_me(session: &mut Session) -> Signal {
    let self_id = session.event.selft().unwrap_or_default().user_id;
    let Ok(segs) = session.event.extra.try_get_as_mut::<&mut Vec<Value>>("message") else {
        return Signal::NotMatch
    };
    let mut mentioned_index = None;
    for (index, seg) in seg_mut_iter(segs).enumerate() {
        match seg {
            MsgSegmentMut::Mention { user_id } => {
                if user_id.as_str() == &self_id {
                    mentioned_index = Some(index);
                    break;
                }
            }
            MsgSegmentMut::Text { text } if index == 0 => {
                for nickname in &session.config.nicknames {
                    if let Some(s) = text.strip_prefix(nickname) {
                        if s.is_empty() {
                            mentioned_index = Some(index);
                            break;
                        }
                        *text = s.to_string();
                        return Signal::Matched;
                    }
                }
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
