use fpt_core::{CommandSpec, RiskLevel};

const SERVER_INFO_EXAMPLES: &[&str] =
    &["fpt server info --site https://example.shotgrid.autodesk.com --output json"];

const SERVER_INFO_NOTES: &[&str] = &[
    "Uses the ShotGrid RPC `info` method over `/api3/json`",
    "Requires only `--site`; auth flags are ignored for this command",
    "Returns raw server metadata such as version and authentication mode",
];

pub const SERVER_INFO_SPEC: CommandSpec = CommandSpec {
    name: "server.info",
    summary: "Fetch ShotGrid server metadata",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "site",
    output: "json",
    examples: SERVER_INFO_EXAMPLES,
    notes: SERVER_INFO_NOTES,
};
