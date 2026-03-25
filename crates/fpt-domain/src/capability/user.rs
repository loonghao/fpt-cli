use fpt_core::{CommandSpec, RiskLevel};

const CURRENT_USER_EXAMPLES: &[&str] = &[
    "fpt user current --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt user current --user-type api --site ...",
    "fpt user current --input '{\"fields\":\"login,name,email\"}' --site ...",
];

const CURRENT_USER_NOTES: &[&str] = &[
    "Returns the currently authenticated user via GET /entity/{collection}/current",
    "Defaults to HumanUser; pass --user-type api for ApiUser",
    "Supports optional query parameters via --input JSON (fields, etc.)",
];

pub const CURRENT_USER_SPEC: CommandSpec = CommandSpec {
    name: "user.current",
    summary: "Get the currently authenticated user",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "optional user_type + optional query params JSON",
    output: "json",
    examples: CURRENT_USER_EXAMPLES,
    notes: CURRENT_USER_NOTES,
};
