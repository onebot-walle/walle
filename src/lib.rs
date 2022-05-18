use std::sync::Arc;

use walle_core::AppConfig;

pub mod builtin;
mod config;
mod matcher;
mod utils;
#[cfg(feature = "scheduler")]
mod scheduler;

pub use config::*;
pub use matcher::*;
#[cfg(feature = "scheduler")]
pub use scheduler::*;

pub type Walle = Arc<walle_core::app::StandardOneBot<Matchers>>;

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
