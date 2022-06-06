use tracing::info;
use walle::{
    builtin::strip_prefix, handler_fn, new_walle, Matcher, MatcherHandlerExt, Matchers,
    MessageContent, Session,
};
use walle_core::action::BotActionExt;

#[tokio::main]
async fn main() {
    let matchers = Matchers::default().add_message_matcher(test_plugin());
    let walle = new_walle(walle_core::config::AppConfig::default(), matchers);
    walle.run_block().await.unwrap();
}

fn test_plugin() -> Matcher<MessageContent> {
    Matcher::new(
        "test",
        "test description",
        handler_fn(|s: Session<MessageContent>| async move {
            info!(target: "api_test", "start api test");
            if let Ok(m) = s.send("hello world").await {
                s.bot.delete_message(m.data.message_id).await.unwrap();
            }
        })
        .pre_handle(strip_prefix("./test"), true),
    )
}
