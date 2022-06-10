use serde::{Deserialize, Serialize};
pub use walle_core::config::*;

/// Matchers 可配置项
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MatchersConfig {
    #[serde(default = "Vec::default")]
    pub nicknames: Vec<String>,
}
