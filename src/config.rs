use serde::{Deserialize, Serialize};
pub use walle_core::config::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MatcherConfig {
    #[serde(default = "Vec::default")]
    pub nicknames: Vec<String>,
}
