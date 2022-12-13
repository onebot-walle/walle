use std::sync::Arc;

use walle_core::{action::Action, obc::AppOBC, prelude::Event, resp::Resp, OneBot};

pub mod matcher;
// #[cfg(feature = "scheduler")]
// mod scheduler;
mod bot;
mod caller;
mod utils;

// pub mod builtin;
pub mod config;

pub use bot::Bot;
pub use caller::{ActionCaller, ActionCallerExt};
pub use config::*;
pub use matcher::*;
pub use walle_core;
// #[cfg(feature = "scheduler")]
// pub use scheduler::*;
#[doc(hidden)]
pub use tokio;
#[doc(hidden)]
pub use tracing;

pub mod builtin;

/// 构造一个新的 Walle 实例
pub fn new_walle(matchers: Matchers) -> Arc<OneBot<AppOBC<Action, Resp>, Matchers>> {
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
    Arc::new(walle_core::OneBot::new(AppOBC::new(), matchers))
}

pub fn test_walle(
    matchers: Matchers,
) -> Arc<OneBot<walle_core::alt::TracingHandler<Event, Action, Resp>, Matchers>> {
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
    Arc::new(walle_core::OneBot::new(
        walle_core::alt::TracingHandler::default(),
        matchers,
    ))
}
