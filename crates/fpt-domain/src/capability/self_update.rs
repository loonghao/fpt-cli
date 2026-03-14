use fpt_core::{CommandSpec, RiskLevel};

const SELF_UPDATE_EXAMPLES: &[&str] = &[
    "fpt self-update --check --output pretty-json",
    "fpt self-update",
    "fpt self-update --version 0.1.0",
];

const SELF_UPDATE_NOTES: &[&str] = &[
    "Downloads release assets over HTTPS from GitHub Releases",
    "Selects the release package that matches the current operating system and CPU architecture",
    "Verifies the downloaded archive against `fpt-checksums.txt` when the checksum asset is present",
    "`--check` is read-only; without it the command replaces the current executable in place",
];

pub const SELF_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "self-update",
    summary: "Check or install the released CLI binary for the current platform",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "https",
    fallback_transport: None,
    input: "optional version / repository override",
    output: "json",
    examples: SELF_UPDATE_EXAMPLES,
    notes: SELF_UPDATE_NOTES,
};
