use fpt_core::{CommandSpec, RiskLevel};

const LICENSE_EXAMPLES: &[&str] = &[
    "fpt license get --site ... --auth-mode script --script-name ... --script-key ...",
];

const LICENSE_NOTES: &[&str] = &[
    "Retrieve the current ShotGrid site license information",
    "Uses the REST license endpoint to query active license details",
];

pub const LICENSE_GET_SPEC: CommandSpec = CommandSpec {
    name: "license.get",
    summary: "Read the ShotGrid site license information",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "none",
    output: "json",
    examples: LICENSE_EXAMPLES,
    notes: LICENSE_NOTES,
};
