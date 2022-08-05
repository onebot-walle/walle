use walle::{
    builtin::{strip_prefix, strip_whitespace},
    may_fail_handler_fn,
    walle_core::{
        event::{Message, MessageDeatilTypes},
        util::ValueMapExt,
    },
    Matcher, MatcherHandlerExt, PreHandler, ReplyAbleSession, Session,
};

mod data_source;
mod users;

fn session_id(s: &Session<Message, MessageDeatilTypes>) -> String {
    match &s.event.detail_type {
        MessageDeatilTypes::Group(group) => {
            format!("{}:{}", group.group_id, s.event.ty.user_id)
        }
        MessageDeatilTypes::Private(_) => format!(":{}", s.event.ty.user_id),
    }
}

pub fn set_api_key() -> Matcher {
    strip_prefix("waka开卷")
        .with(strip_whitespace())
        .layer(may_fail_handler_fn(
            |s: &Session<Message, MessageDeatilTypes>| {
                Box::pin(async move {
                    let mut data = users::load_users().await?;
                    let map = data.entry(session_id(&s)).or_default();
                    map.insert(
                        s.event.extra.get_downcast("user_name").unwrap_or_default(), //todo
                        s.event.ty.alt_message.clone(),
                    );
                    users::save_users(&data).await?;
                    s.send("设置完毕，可以开始卷哩").await.ok();
                    Ok::<_, String>(())
                })
            },
        ))
        .boxed()
}

pub fn today_rank() -> Matcher {
    strip_prefix("waka今日排行")
        .layer(may_fail_handler_fn(
            |s: &Session<Message, MessageDeatilTypes>| {
                Box::pin(async move {
                    let data = users::load_users().await?;
                    let api_keys = data.get(&session_id(&s)).ok_or("api_keys not found")?;
                    let today = data_source::get_today(api_keys).await;
                    let mut oks = String::from("今日排行: ");
                    let mut errs = String::default();
                    for (name, v) in today.iter() {
                        match v {
                            Ok(today) => {
                                oks.push_str(&format!("\n{}: {}", name, today.data.digital))
                            }
                            Err(e) => {
                                errs.push('\n');
                                errs.push_str(e);
                            }
                        }
                    }
                    if !errs.is_empty() {
                        oks.push_str("\nerrors:\n");
                        oks.push_str(&errs);
                    }
                    s.send(oks).await.ok();
                    Ok::<_, String>(())
                })
            },
        ))
        .boxed()
}

pub fn weeks_rank() -> Matcher {
    strip_prefix("waka本周排行")
        .layer(may_fail_handler_fn(
            |s: &Session<Message, MessageDeatilTypes>| {
                Box::pin(async move {
                    let data = users::load_users().await?;
                    let api_keys = data.get(&session_id(&s)).ok_or("api_keys not found")?;
                    let weeks = data_source::get_weekdays(api_keys).await;
                    let mut oks = String::from("本周排行: ");
                    let mut errs = String::default();
                    for (name, v) in weeks.iter() {
                        match v {
                            Ok(v) => {
                                oks.push_str(&format!("\n{}: {}h", name, v.total_seconds / 3600.0))
                            }
                            Err(e) => {
                                errs.push('\n');
                                errs.push_str(e);
                            }
                        }
                    }
                    if !errs.is_empty() {
                        oks.push_str("\nerrors:\n");
                        oks.push_str(&errs);
                    }
                    s.send(oks).await.ok();
                    Ok::<_, String>(())
                })
            },
        ))
        .boxed()
}
