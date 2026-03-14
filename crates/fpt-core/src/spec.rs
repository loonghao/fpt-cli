use crate::model::RiskLevel;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub risk: RiskLevel,
    pub implemented: bool,
    pub supports_dry_run: bool,
    pub preferred_transport: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_transport: Option<&'static str>,
    pub input: &'static str,
    pub output: &'static str,
    pub examples: &'static [&'static str],
    pub notes: &'static [&'static str],
}
