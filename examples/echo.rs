use walle::{
    builtin::{echo, echo2, on_to_me},
    config::AppConfig,
    new_walle, Matchers, OneMinutePassed, Scheduler,
};

#[tokio::main]
async fn main() {
    let matchers = Matchers::default()
        .add_message_matcher(echo().map(|h| on_to_me(h)).build())
        .add_message_matcher(echo2().build());
    let walle = new_walle(AppConfig::default(), matchers);
    let mut sche = Scheduler::new(walle.clone());
    sche.add(OneMinutePassed);
    sche.start();
    walle.run_block().await.unwrap();
}
