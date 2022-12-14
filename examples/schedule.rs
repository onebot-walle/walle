use walle::{
    new_walle, walle_core::config::AppConfig, Matchers, MatchersConfig, OneMinutePassed, Scheduler,
};

#[tokio::main]
async fn main() {
    let matchers = Matchers::default();
    let walle = new_walle(matchers);
    let mut scheduler = Scheduler::new(walle.clone());
    scheduler.add(OneMinutePassed);
    scheduler.start();
    let joins = walle
        .start(AppConfig::default(), MatchersConfig::default(), true)
        .await
        .unwrap();
    for join in joins {
        join.await.ok();
    }
}
