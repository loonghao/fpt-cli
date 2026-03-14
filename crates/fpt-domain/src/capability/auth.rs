use fpt_core::{CommandSpec, RiskLevel};

const AUTH_TEST_EXAMPLES: &[&str] = &[
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode script --script-name bot --script-key xxx",
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode user-password --username user@example.com --password secret",
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode session-token --session-token xxx",
];

const AUTH_NOTES: &[&str] = &[
    "Uses REST OAuth to obtain an access token",
    "Supports three auth modes: script, user-password, and session-token",
    "If 2FA is enabled, pass `--auth-token` as an additional parameter",
];

pub const AUTH_TEST_SPEC: CommandSpec = CommandSpec {
    name: "auth.test",
    summary: "Validate ShotGrid authentication via REST OAuth",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "site + one auth profile (script / user-password / session-token)",
    output: "json",
    examples: AUTH_TEST_EXAMPLES,
    notes: AUTH_NOTES,
};
