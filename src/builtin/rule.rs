use crate::{rule_fn, Rule, Session};
use crate::{rule_fn_unwarp, Signal};
use walle_core::segment::MsgSegmentRef;
use walle_core::util::{Value, ValueMapExt};
use walle_core::WalleResult;

pub struct UserIdChecker {
    pub user_id: String,
}

impl Rule for UserIdChecker {
    fn rule(&self, session: &Session) -> Signal {
        if session
            .event
            .extra
            .try_get_as_ref::<&str>("user_id")
            .unwrap_or_default()
            == self.user_id.as_str()
        {
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

impl Rule for GroupIdChecker {
    fn rule(&self, session: &Session) -> Signal {
        if session
            .event
            .extra
            .try_get_as_ref::<&str>("group_id")
            .unwrap_or_default()
            == self.group_id.as_str()
        {
            Signal::Matched
        } else {
            Signal::NotMatch
        }
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

pub fn channel_id_check(guild_id: &str, channel_id: &str) -> impl Rule {
    let guild_id2 = guild_id.to_string();
    let channel_id2 = channel_id.to_string();
    rule_fn_unwarp(move |session| {
        if session.event.extra.try_get_as_ref::<&str>("guild_id")? == guild_id2.as_str()
            && session.event.extra.try_get_as_ref::<&str>("channel_id")? == channel_id2.as_str()
        {
            Ok(Signal::Matched)
        } else {
            Ok(Signal::NotMatch)
        }
    })
}

pub fn start_with(pat: &str) -> impl Rule {
    let word = pat.to_string();
    rule_fn(move |session: &Session| {
        if session
            .event
            .extra
            .try_get_as_ref::<&str>("alt_message")
            .unwrap_or_default()
            .starts_with(&word)
        {
            Signal::Matched
        } else {
            Signal::NotMatch
        }
    })
}

fn _mention_me(session: &Session) -> WalleResult<Signal> {
    let alt = &session.event.extra.try_get_as_ref::<&str>("alt_message")?;
    for nickname in &session.config.nicknames {
        if alt.starts_with(nickname) {
            return Ok(Signal::Matched);
        }
    }
    for user_id in session
        .event
        .extra
        .try_get_as_ref::<&Vec<Value>>("message")?
        .iter()
        .filter_map(|v| match v.try_as_ref::<MsgSegmentRef<'_>>() {
            Ok(MsgSegmentRef::Mention { user_id, .. }) => Some(user_id),
            _ => None,
        })
    {
        if user_id == session.event.selft().unwrap_or_default().user_id {
            return Ok(Signal::Matched);
        }
    }
    Ok(Signal::NotMatch)
}

pub fn mention_me_rule<D, S, P, I>() -> impl Rule {
    rule_fn_unwarp(_mention_me)
}

pub fn to_me_rule() -> impl Rule {
    rule_fn_unwarp(|session: &Session| {
        if session.event.detail_type.as_str() == "private" {
            Ok(Signal::Matched)
        } else {
            _mention_me(session)
        }
    })
}
