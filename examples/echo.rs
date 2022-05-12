use walle::{
    builtin::{echo2, Echo},
    Matchers, Walle,
};
use walle_core::AppConfig;

#[tokio::main]
async fn main() {
    let plugins = Matchers::new()
        .add_message_matcher(Echo::new())
        .add_message_matcher(echo2());
    let walle = Walle::new(AppConfig::default(), plugins);
    walle.start().await.unwrap();
}
