use walle::{new_walle, walle_core::config::AppConfig, Matchers, MatchersConfig};
use walle_plugin_wakatime::*;

#[tokio::main]
async fn main() {
    let matchers = Matchers::default()
        .add_matcher(set_api_key())
        .add_matcher(today_rank())
        .add_matcher(weeks_rank());
    let walle = new_walle(matchers);
    let joins = walle
        .start(AppConfig::default(), MatchersConfig::default(), true)
        .await
        .unwrap();
    for join in joins {
        join.await.ok();
    }
}
