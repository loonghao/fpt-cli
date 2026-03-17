use fpt_core::{CommandSpec, RiskLevel};

const HIERARCHY_EXAMPLES: &[&str] = &[
    "fpt hierarchy search --input @hierarchy_query.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const HIERARCHY_NOTES: &[&str] = &[
    "Search the entity hierarchy tree using the ShotGrid hierarchy/_search endpoint",
    "Input must be a JSON body describing the hierarchy search criteria (root_entity, seed_entity_field, entity_fields, etc.)",
];

pub const HIERARCHY_SEARCH_SPEC: CommandSpec = CommandSpec {
    name: "hierarchy.search",
    summary: "Search the ShotGrid entity hierarchy navigation tree",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "hierarchy search body JSON",
    output: "json",
    examples: HIERARCHY_EXAMPLES,
    notes: HIERARCHY_NOTES,
};
