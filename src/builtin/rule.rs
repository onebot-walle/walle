use crate::Signal;
use crate::{rule_fn, Rule, Session};
use walle_core::event::{DetailTypeDeclare, Group, Message, MessageDeatilTypes};

pub struct UserIdChecker {
    pub user_id: String,
}

impl<D, S, P, I> Rule<Message, D, S, P, I> for UserIdChecker {
    fn rule(&self, session: &Session<Message, D, S, P, I>) -> Signal {
        if session.event.ty.user_id == self.user_id {
            Signal::Matched
        } else {
            Signal::NotMatch
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

impl<S, P, I> Rule<Message, Group, S, P, I> for GroupIdChecker {
    fn rule(&self, session: &Session<Message, Group, S, P, I>) -> Signal {
        if session.event.detail_type.group_id == self.group_id {
            Signal::Matched
        } else {
            Signal::NotMatch
        }
    }
}

impl<S, P, I> Rule<Message, MessageDeatilTypes, S, P, I> for GroupIdChecker {
    fn rule(&self, session: &Session<Message, MessageDeatilTypes, S, P, I>) -> Signal {
        if let MessageDeatilTypes::Group(Group { group_id }) = &session.event.detail_type {
            if group_id == self.group_id.as_str() {
                return Signal::Matched;
            }
        }
        Signal::NotMatch
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

pub fn start_with<D, S, P, I>(pat: &str) -> impl Rule<Message, D, S, P, I> {
    let word = pat.to_string();
    rule_fn(move |session: &Session<Message, D, S, P, I>| -> Signal {
        if session.event.ty.alt_message.starts_with(&word) {
            Signal::Matched
        } else {
            Signal::NotMatch
        }
    })
}

fn _mention_me<D, S, P, I>(session: &Session<Message, D, S, P, I>) -> Signal {
    use walle_core::segment::{Mention, MessageExt};
    let alt = &session.event.ty.alt_message;
    for nickname in &session.config.nicknames {
        if alt.starts_with(nickname) {
            return Signal::Matched;
        }
    }
    for mention in session.event.ty.message.clone().extract::<Mention>() {
        if mention.user_id == session.event.self_id {
            return Signal::Matched;
        }
    }
    Signal::NotMatch
}

pub fn mention_me_rule<D, S, P, I>() -> impl Rule<Message, D, S, P, I> {
    rule_fn(_mention_me)
}

pub fn to_me_rule<D: DetailTypeDeclare, S, P, I>() -> impl Rule<Message, D, S, P, I> {
    rule_fn(|session: &Session<Message, D, S, P, I>| {
        if session.event.detail_type.detail_type() == "private" {
            Signal::Matched
        } else {
            _mention_me(session)
        }
    })
}

pub fn allways_matched<T, D, S, P, I>() -> impl Rule<T, D, S, P, I> {
    rule_fn(|_session| Signal::Matched)
}
