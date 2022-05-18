use walle::{
    builtin::{on_command, on_to_me, Echo},
    new_walle, Matcher, Matchers, OneMinutePassed, Scheduler,
};
use walle_core::AppConfig;

#[tokio::main]
async fn main() {
    let matchers = Matchers::default().add_message_matcher(Matcher::new_with(
        "echo to me",
        "description",
        on_command("echo", Echo),
        |h| on_to_me(h),
    ));
    // .add_message_matcher(echo2());
    let walle = new_walle(AppConfig::default(), matchers);
    let mut sche = Scheduler::new(walle.clone());
    sche.add(OneMinutePassed);
    sche.start();
    walle.run_block().await.unwrap();
}
