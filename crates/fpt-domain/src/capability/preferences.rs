use fpt_core::{CommandSpec, RiskLevel};

const PREFERENCES_GET_EXAMPLES: &[&str] =
    &["fpt preferences get --site ... --auth-mode script --script-name ... --script-key ..."];

const PREFERENCES_GET_NOTES: &[&str] = &[
    "Uses the REST endpoint GET /preferences",
    "Returns site-level ShotGrid preferences and configuration",
    "Read-only; no input required beyond authentication",
];

pub const PREFERENCES_GET_SPEC: CommandSpec = CommandSpec {
    name: "preferences.get",
    summary: "Read site-level ShotGrid preferences",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "none",
    output: "json preferences object",
    examples: PREFERENCES_GET_EXAMPLES,
    notes: PREFERENCES_GET_NOTES,
};

const PREFERENCES_UPDATE_EXAMPLES: &[&str] = &[
    "fpt preferences update --input '{\"name\":\"value\"}' --site ... --auth-mode script --script-name ... --script-key ...",
];

const PREFERENCES_UPDATE_NOTES: &[&str] = &[
    "Uses the REST endpoint PUT /preferences",
    "Updates site-level ShotGrid preferences",
    "Input must be a JSON object with preference key-value pairs",
];

pub const PREFERENCES_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "preferences.update",
    summary: "Update site-level ShotGrid preferences",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "JSON object with preference key-value pairs",
    output: "json preferences object",
    examples: PREFERENCES_UPDATE_EXAMPLES,
    notes: PREFERENCES_UPDATE_NOTES,
};

const PREFERENCES_CUSTOM_ENTITY_EXAMPLES: &[&str] = &[
    "fpt preferences custom-entity --input '{\"entity_type\":\"CustomEntity01\"}' --site ... --auth-mode script --script-name ... --script-key ...",
];

const PREFERENCES_CUSTOM_ENTITY_NOTES: &[&str] = &[
    "Uses the REST endpoint POST /preferences/custom_entity",
    "Enables a custom entity type on the ShotGrid site",
    "Input must be a JSON object containing the `entity_type` to enable",
];

pub const PREFERENCES_CUSTOM_ENTITY_SPEC: CommandSpec = CommandSpec {
    name: "preferences.custom-entity",
    summary: "Enable a custom entity type on the ShotGrid site",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "JSON object with entity_type",
    output: "json",
    examples: PREFERENCES_CUSTOM_ENTITY_EXAMPLES,
    notes: PREFERENCES_CUSTOM_ENTITY_NOTES,
};
