use walle::{builtin::echo, new_walle, MatcherHandler, Matchers, MatchersConfig};
use walle_core::config::AppConfig;

#[tokio::main]
async fn main() {
    let matchers = Matchers::default().add_matcher(echo().boxed());
    let walle = new_walle(matchers, "debug");
    let joins = walle
        .start(AppConfig::default(), MatchersConfig::default(), true)
        .await
        .unwrap();
    for join in joins {
        join.await.ok();
    }
}
