use tracing::info;
use walle::{
    builtin::strip_prefix, handler_fn, new_walle, Matcher, MatcherHandlerExt, Matchers,
    MessageContent, Session,
};
use walle_core::{action::BotActionExt, extended_map, MessageSegment};

#[tokio::main]
async fn main() {
    let matchers = Matchers::default()
        .add_message_matcher(recall_test_plugin())
        // .add_message_matcher(flash_test_plugin())
        .add_message_matcher(reply_test_plugin());
    let walle = new_walle(walle_core::config::AppConfig::default(), matchers);
    walle.run_block().await.unwrap();
}

fn recall_test_plugin() -> Matcher<MessageContent> {
    Matcher::new(
        "recall_test",
        "recall",
        handler_fn(|s: Session<MessageContent>| async move {
            info!(target: "api_test", "start api test");
            if let Ok(m) = s.send("hello world").await {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                s.bot.delete_message(m.data.message_id).await.unwrap();
            }
        })
        .pre_handle(strip_prefix("./recall"), true),
    )
}

#[allow(dead_code)]
fn flash_test_plugin() -> Matcher<MessageContent> {
    Matcher::new(
        "flash_test",
        "flash",
        handler_fn(|s| async move {
            let mut messages = s.event.message().into_iter();
            while let Some(MessageSegment::Image { file_id, .. }) = messages.next() {
                s.send(vec![MessageSegment::image_with_extend(
                    file_id.to_string(),
                    extended_map! {"flash":true},
                )])
                .await
                .unwrap();
            }
        }),
    )
}

fn reply_test_plugin() -> Matcher<MessageContent> {
    Matcher::new(
        "reply_test",
        "reply",
        handler_fn(|s: Session<MessageContent>| async move {
            s.send(vec![MessageSegment::Reply {
                message_id: s.event.message_id().to_string(),
                user_id: s.event.user_id().to_string(),
                extra: extended_map! {},
            }])
            .await
            .unwrap();
        })
        .pre_handle(strip_prefix("./reply"), true),
    )
}
