use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MatcherConfig {
    #[serde(default = "Vec::default")]
    pub nicknames: Vec<String>,
}
