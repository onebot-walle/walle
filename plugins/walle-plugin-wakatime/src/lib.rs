use walle::{
    builtin::{strip_prefix, strip_whitespace},
    handler_fn, may_fail_handler_fn,
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

async fn send_all(s: &Session<Message, MessageDeatilTypes>, r: Result<String, String>) {
    match r {
        Ok(msg) => {
            s.send(msg).await.ok();
        }
        Err(e) => {
            s.send(format!("error: {e}")).await.ok();
        }
    }
}

pub fn set_api_key() -> Matcher {
    strip_prefix("waka开卷")
        .with(strip_whitespace())
        .layer(may_fail_handler_fn(
            |s: Session<Message, MessageDeatilTypes>| {
                async move {
                    let mut data = users::load_users().await?;
                    let map = data.entry(session_id(&s)).or_default();
                    map.insert(
                        s.event.extra.get_downcast("user_name").unwrap_or_default(), //todo
                        s.event.ty.alt_message.clone(),
                    );
                    users::save_users(&data).await?;
                    s.send("设置完毕，可以开始卷哩").await.ok();
                    Ok::<_, String>(())
                }
            },
        ))
        .boxed()
}

async fn _today_rank(s: &Session<Message, MessageDeatilTypes>) -> Result<String, String> {
    let data = users::load_users().await?;
    let api_keys = data.get(&session_id(&s)).ok_or("api_keys not found")?;
    let today = data_source::get_today(api_keys).await;
    println!("{:?}", today);
    let mut oks = String::from("今日排行：\n");
    let mut errs = String::default();
    for (name, v) in today.iter() {
        match v {
            Ok(today) => oks.push_str(&format!("{}: {}\n", name, today.data.digital)),
            Err(e) => {
                errs.push_str(e);
                errs.push('\n');
            }
        }
    }
    if !errs.is_empty() {
        oks.push_str("\nerrors:\n");
        oks.push_str(&errs);
    }
    Ok(oks)
}

pub fn today_rank() -> Matcher {
    strip_prefix("waka今日排行")
        .with(strip_whitespace())
        .layer(handler_fn(
            |s: Session<Message, MessageDeatilTypes>| async move {
                send_all(&s, _today_rank(&s).await).await
            },
        ))
        .boxed()
}
