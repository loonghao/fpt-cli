use fpt_core::{CommandSpec, RiskLevel};

const CAPABILITIES_EXAMPLES: &[&str] = &["fpt capabilities --output json"];
const INSPECT_EXAMPLES: &[&str] = &["fpt inspect command entity.update --output json"];
const AUTH_TEST_EXAMPLES: &[&str] = &[
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode script --script-name bot --script-key xxx",
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode user-password --username user@example.com --password secret",
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode session-token --session-token xxx",
];
const SCHEMA_ENTITIES_EXAMPLES: &[&str] = &[
    "fpt schema entities --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt schema entities --site ... --auth-mode user-password --username ... --password ...",
];
const SCHEMA_FIELDS_EXAMPLES: &[&str] = &[
    "fpt schema fields Shot --site ... --auth-mode script --script-name ... --script-key ...",
];
const ENTITY_GET_EXAMPLES: &[&str] = &[
    "fpt entity get Shot 123 --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity get Shot 123 --site ... --auth-mode session-token --session-token ...",
];
const ENTITY_FIND_EXAMPLES: &[&str] = &[
    "fpt entity find Asset --input @query.json --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity find Asset --filter-dsl 'sg_status_list == \"ip\" and (code ~ \"bunny\" or id > 100)' --site ... --auth-mode script --script-name ... --script-key ...",
];

const ENTITY_CREATE_EXAMPLES: &[&str] = &[
    "fpt entity create Version --input @payload.json --dry-run",
    "fpt entity create Version --input @payload.json --site ... --auth-mode user-password --username ... --password ...",
];
const ENTITY_UPDATE_EXAMPLES: &[&str] = &[
    "fpt entity update Task 42 --input @patch.json --dry-run",
    "fpt entity update Task 42 --input @patch.json --site ... --auth-mode user-password --username ... --password ...",
];
const ENTITY_DELETE_EXAMPLES: &[&str] = &[
    "fpt entity delete Playlist 99 --dry-run",
    "fpt entity delete Playlist 99 --yes --site ... --auth-mode script --script-name ... --script-key ...",
];
const ENTITY_BATCH_GET_EXAMPLES: &[&str] = &[
    "fpt entity batch get Shot --input '{\"ids\":[101,102],\"fields\":[\"code\",\"sg_status_list\"]}' --output json",
];
const ENTITY_BATCH_FIND_EXAMPLES: &[&str] = &[
    "fpt entity batch find Asset --input @batch_queries.json --output json",
];
const ENTITY_BATCH_CREATE_EXAMPLES: &[&str] = &[
    "fpt entity batch create Version --input @batch_payloads.json --dry-run --output json",
];
const ENTITY_BATCH_UPDATE_EXAMPLES: &[&str] = &[
    "fpt entity batch update Task --input @batch_updates.json --dry-run --output json",
];
const ENTITY_BATCH_DELETE_EXAMPLES: &[&str] = &[
    "fpt entity batch delete Playlist --input '{\"ids\":[99,100]}' --dry-run --output json",
    "fpt entity batch delete Playlist --input '{\"ids\":[99,100]}' --yes --output json",
];


const CAPABILITIES_NOTES: &[&str] = &[
    "Stable contract designed for OpenClaw integrations",
    "Default output format is TOON; JSON and pretty JSON remain available via `--output`",
    "Current release is REST-first, with RPC fallback planned",
];
const INSPECT_NOTES: &[&str] = &["Use dotted command names, for example `entity.update`"];
const AUTH_NOTES: &[&str] = &[
    "Uses REST OAuth to obtain an access token",
    "Supports three auth modes: script, user-password, and session-token",
    "If 2FA is enabled, pass `--auth-token` as an additional parameter",
];
const SCHEMA_NOTES: &[&str] = &[
    "Schema endpoints currently return raw ShotGrid REST JSON",
    "Auth configuration supports script, user-password, and session-token",
];
const ENTITY_NOTES: &[&str] = &[
    "create/update accepts raw REST JSON request bodies",
    "entity find supports `--filter-dsl` and automatically switches to `_search` for complex filters",
    "delete is blocked by default and requires explicit `--yes`",
    "All entity commands reuse the same auth configuration",
];
const ENTITY_BATCH_NOTES: &[&str] = &[
    "Batch commands are client-side orchestration over existing REST CRUD endpoints",
    "Each item returns its own ok/error envelope in `results`",
    "Batch create/update/delete supports `--dry-run`; batch delete still requires explicit `--yes`",
    "Batch execution reuses the same auth configuration and benefits from in-process token reuse",
    "Batch requests run with controlled concurrency; use `FPT_BATCH_CONCURRENCY` to tune throughput when needed",
];




static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
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
    },
    CommandSpec {
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
    },
    CommandSpec {
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
    },
    CommandSpec {
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
    },
    CommandSpec {
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
    },
    CommandSpec {
        name: "entity.get",
        summary: "Read a single entity record",
        risk: RiskLevel::Read,
        implemented: true,
        supports_dry_run: false,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "entity + id",
        output: "json",
        examples: ENTITY_GET_EXAMPLES,
        notes: ENTITY_NOTES,
    },
    CommandSpec {
        name: "entity.find",
        summary: "Query a collection of entity records",
        risk: RiskLevel::Read,
        implemented: true,
        supports_dry_run: false,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "query JSON / --filter-dsl",

        output: "json",
        examples: ENTITY_FIND_EXAMPLES,
        notes: ENTITY_NOTES,
    },
    CommandSpec {
        name: "entity.create",
        summary: "Create an entity record",
        risk: RiskLevel::Write,
        implemented: true,
        supports_dry_run: true,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "raw REST request body",
        output: "json",
        examples: ENTITY_CREATE_EXAMPLES,
        notes: ENTITY_NOTES,
    },
    CommandSpec {
        name: "entity.update",
        summary: "Update an entity record",
        risk: RiskLevel::Write,
        implemented: true,
        supports_dry_run: true,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "raw REST request body",
        output: "json",
        examples: ENTITY_UPDATE_EXAMPLES,
        notes: ENTITY_NOTES,
    },
    CommandSpec {
        name: "entity.delete",
        summary: "Delete an entity record",
        risk: RiskLevel::Destructive,
        implemented: true,
        supports_dry_run: true,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "entity + id",
        output: "json",
        examples: ENTITY_DELETE_EXAMPLES,
        notes: ENTITY_NOTES,
    },
    CommandSpec {
        name: "entity.batch.get",
        summary: "Read multiple entity records by id",
        risk: RiskLevel::Read,
        implemented: true,
        supports_dry_run: false,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "ids array / object with ids + optional fields",
        output: "json",
        examples: ENTITY_BATCH_GET_EXAMPLES,
        notes: ENTITY_BATCH_NOTES,
    },
    CommandSpec {
        name: "entity.batch.find",
        summary: "Run multiple entity queries in one CLI invocation",
        risk: RiskLevel::Read,
        implemented: true,
        supports_dry_run: false,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "request object array / object with requests",
        output: "json",
        examples: ENTITY_BATCH_FIND_EXAMPLES,
        notes: ENTITY_BATCH_NOTES,
    },
    CommandSpec {
        name: "entity.batch.create",
        summary: "Create multiple entity records",
        risk: RiskLevel::Write,
        implemented: true,
        supports_dry_run: true,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "request body array / object with items",
        output: "json",
        examples: ENTITY_BATCH_CREATE_EXAMPLES,
        notes: ENTITY_BATCH_NOTES,
    },
    CommandSpec {
        name: "entity.batch.update",
        summary: "Update multiple entity records",
        risk: RiskLevel::Write,
        implemented: true,
        supports_dry_run: true,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "[{id, body}] / object with items",
        output: "json",
        examples: ENTITY_BATCH_UPDATE_EXAMPLES,
        notes: ENTITY_BATCH_NOTES,
    },
    CommandSpec {
        name: "entity.batch.delete",
        summary: "Delete multiple entity records",
        risk: RiskLevel::Destructive,
        implemented: true,
        supports_dry_run: true,
        preferred_transport: "rest",
        fallback_transport: Some("rpc"),
        input: "ids array / object with ids",
        output: "json",
        examples: ENTITY_BATCH_DELETE_EXAMPLES,
        notes: ENTITY_BATCH_NOTES,
    },
];


pub fn command_specs() -> &'static [CommandSpec] {
    COMMANDS
}

pub fn find_command_spec(name: &str) -> Option<&'static CommandSpec> {
    let normalized = normalize_command_name(name);
    COMMANDS.iter().find(|spec| spec.name == normalized)
}

fn normalize_command_name(name: &str) -> String {
    name.trim().replace(' ', ".").to_ascii_lowercase()
}
