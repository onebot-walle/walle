use std::sync::Arc;

use walle_core::config::AppConfig;

mod matcher;
#[cfg(feature = "scheduler")]
mod scheduler;
mod utils;

pub mod builtin;
pub mod config;

pub use config::*;
pub use matcher::*;
#[cfg(feature = "scheduler")]
pub use scheduler::*;

pub type Walle = Arc<walle_core::app::StandardOneBot<Matchers>>;
pub type MessageContent = walle_core::event::MessageContent<walle_core::event::MessageEventDetail>;

pub fn new_walle(config: AppConfig, matchers: Matchers) -> Walle {
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(
        time::UtcOffset::from_hms(8, 0, 0).unwrap(),
        time::format_description::parse(
            "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]",
        )
        .unwrap(),
    );
    let env = tracing_subscriber::EnvFilter::from("debug");
    tracing_subscriber::fmt()
        .with_env_filter(env)
        .with_timer(timer)
        .init();
    Arc::new(walle_core::app::StandardOneBot::new(config, matchers))
}
