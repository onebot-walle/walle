use walle::{
    builtin::{echo2, Echo},
    new_walle, Matchers, OneMinutePassed, Scheduler,
};
use walle_core::AppConfig;

#[tokio::main]
async fn main() {
    let matchers = Matchers::new()
        .add_message_matcher(Echo::new())
        .add_message_matcher(echo2());
    let walle = new_walle(AppConfig::default(), matchers);
    let mut sche = Scheduler::new(walle.clone());
    sche.add(OneMinutePassed);
    sche.start();
    walle.run_block().await.unwrap();
}
