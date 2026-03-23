use fpt_core::{CommandSpec, RiskLevel};

const SCHEMA_ENTITIES_EXAMPLES: &[&str] = &[
    "fpt schema entities --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt schema entities --site ... --auth-mode user-password --username ... --password ...",
];
const SCHEMA_FIELDS_EXAMPLES: &[&str] =
    &["fpt schema fields Shot --site ... --auth-mode script --script-name ... --script-key ..."];

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

const SCHEMA_FIELD_CREATE_EXAMPLES: &[&str] = &[
    "fpt schema field-create Shot --input '{\"data_type\":\"text\",\"properties\":{\"name\":{\"value\":\"Custom Field\"}}}' --site ...",
];
const SCHEMA_FIELD_UPDATE_EXAMPLES: &[&str] = &[
    "fpt schema field-update Shot sg_custom_field --input '{\"properties\":{\"name\":{\"value\":\"Renamed Field\"}}}' --site ...",
];
const SCHEMA_FIELD_DELETE_EXAMPLES: &[&str] =
    &["fpt schema field-delete Shot sg_custom_field --site ..."];

const SCHEMA_FIELD_NOTES: &[&str] = &[
    "Schema field mutations require admin-level credentials",
    "field-create accepts a JSON body with `data_type` and `properties`",
    "field-delete is irreversible and requires careful use",
];

pub const SCHEMA_FIELD_CREATE_SPEC: CommandSpec = CommandSpec {
    name: "schema.field-create",
    summary: "Create a new custom field on an entity type",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + field definition JSON",
    output: "json",
    examples: SCHEMA_FIELD_CREATE_EXAMPLES,
    notes: SCHEMA_FIELD_NOTES,
};

pub const SCHEMA_FIELD_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "schema.field-update",
    summary: "Update properties of an existing custom field",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + field_name + properties JSON",
    output: "json",
    examples: SCHEMA_FIELD_UPDATE_EXAMPLES,
    notes: SCHEMA_FIELD_NOTES,
};

pub const SCHEMA_FIELD_DELETE_SPEC: CommandSpec = CommandSpec {
    name: "schema.field-delete",
    summary: "Delete a custom field from an entity type",
    risk: RiskLevel::Destructive,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + field_name",
    output: "json",
    examples: SCHEMA_FIELD_DELETE_EXAMPLES,
    notes: SCHEMA_FIELD_NOTES,
};

const SCHEMA_FIELD_READ_EXAMPLES: &[&str] = &["fpt schema field-read Shot code --site ..."];

pub const SCHEMA_FIELD_READ_SPEC: CommandSpec = CommandSpec {
    name: "schema.field-read",
    summary: "Read the schema definition of a single field on an entity type",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + field_name",
    output: "json",
    examples: SCHEMA_FIELD_READ_EXAMPLES,
    notes: SCHEMA_NOTES,
};

const SCHEMA_ENTITY_READ_EXAMPLES: &[&str] = &[
    "fpt schema entity-read Shot --site ... --auth-mode script --script-name ... --script-key ...",
];

pub const SCHEMA_ENTITY_READ_SPEC: CommandSpec = CommandSpec {
    name: "schema.entity-read",
    summary: "Read the full schema definition of an entity type",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity name",
    output: "json",
    examples: SCHEMA_ENTITY_READ_EXAMPLES,
    notes: SCHEMA_NOTES,
};

const SCHEMA_FIELD_REVIVE_EXAMPLES: &[&str] = &[
    "fpt schema field-revive Shot sg_deleted_field --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEMA_FIELD_REVIVE_NOTES: &[&str] = &[
    "Revives a previously deleted custom field via POST /api/{ver}/schema/{entity}/fields/{field_name}?revive=true",
    "The field must have been previously deleted and not yet purged",
];

pub const SCHEMA_FIELD_REVIVE_SPEC: CommandSpec = CommandSpec {
    name: "schema.field-revive",
    summary: "Revive a previously deleted custom field on an entity type",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + field_name",
    output: "json",
    examples: SCHEMA_FIELD_REVIVE_EXAMPLES,
    notes: SCHEMA_FIELD_REVIVE_NOTES,
};
