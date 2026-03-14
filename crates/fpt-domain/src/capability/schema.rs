use fpt_core::{CommandSpec, RiskLevel};

const SCHEMA_ENTITIES_EXAMPLES: &[&str] = &[
    "fpt schema entities --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt schema entities --site ... --auth-mode user-password --username ... --password ...",
];
const SCHEMA_FIELDS_EXAMPLES: &[&str] = &[
    "fpt schema fields Shot --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEMA_NOTES: &[&str] = &[
    "Schema endpoints currently return raw ShotGrid REST JSON",
    "Auth configuration supports script, user-password, and session-token",
];

pub const SCHEMA_ENTITIES_SPEC: CommandSpec = CommandSpec {
    name: "schema.entities",
    summary: "Fetch schema for all entity types",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "authenticated profile",
    output: "json",
    examples: SCHEMA_ENTITIES_EXAMPLES,
    notes: SCHEMA_NOTES,
};

pub const SCHEMA_FIELDS_SPEC: CommandSpec = CommandSpec {
    name: "schema.fields",
    summary: "Fetch field schema for a specific entity",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "entity name",
    output: "json",
    examples: SCHEMA_FIELDS_EXAMPLES,
    notes: SCHEMA_NOTES,
};
