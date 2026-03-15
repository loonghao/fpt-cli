use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Toon,
    #[default]
    Json,
    PrettyJson,
}

impl OutputFormat {
    pub const fn is_pretty(self) -> bool {
        matches!(self, Self::PrettyJson)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Read,
    Write,
    Destructive,
}
