use walle::{
    builtin::{echo2, Echo},
    Plugins, Walle,
};
use walle_core::AppConfig;

#[tokio::main]
async fn main() {
    let plugins = Plugins::new()
        .add_message_plugin(Echo::new())
        .add_message_plugin(echo2());
    let walle = Walle::new(AppConfig::default(), plugins);
    walle.start().await.unwrap();
}
