use fpt_core::{CommandSpec, RiskLevel};

const CAPABILITIES_EXAMPLES: &[&str] = &["fpt capabilities --output json"];
const INSPECT_EXAMPLES: &[&str] = &["fpt inspect command entity.update --output json"];

const CAPABILITIES_NOTES: &[&str] = &[
    "Stable contract designed for OpenClaw integrations",
    "Default output format is JSON for agent-friendly automation; TOON and pretty JSON remain available via `--output`",
    "Current release is REST-first, with RPC fallback planned",
];
const INSPECT_NOTES: &[&str] = &["Use dotted command names, for example `entity.update`"];

pub const CAPABILITIES_SPEC: CommandSpec = CommandSpec {
    name: "capabilities",
    summary: "Output CLI capabilities and command contracts",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "core",
    fallback_transport: None,
    input: "none",
    output: "json",
    examples: CAPABILITIES_EXAMPLES,
    notes: CAPABILITIES_NOTES,
};

pub const INSPECT_COMMAND_SPEC: CommandSpec = CommandSpec {
    name: "inspect.command",
    summary: "Inspect the detailed contract of a single command",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "core",
    fallback_transport: None,
    input: "command name",
    output: "json",
    examples: INSPECT_EXAMPLES,
    notes: INSPECT_NOTES,
};
