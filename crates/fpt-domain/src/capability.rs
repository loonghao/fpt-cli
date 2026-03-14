mod auth;
mod core;
mod entity;
mod schema;
mod self_update;
mod server;
mod work_schedule;

use fpt_core::CommandSpec;

static COMMANDS: &[CommandSpec] = &[
    core::CAPABILITIES_SPEC,
    core::INSPECT_COMMAND_SPEC,
    auth::AUTH_TEST_SPEC,
    server::SERVER_INFO_SPEC,
    schema::SCHEMA_ENTITIES_SPEC,
    schema::SCHEMA_FIELDS_SPEC,
    entity::ENTITY_GET_SPEC,
    entity::ENTITY_FIND_SPEC,
    entity::ENTITY_FIND_ONE_SPEC,
    entity::ENTITY_SUMMARIZE_SPEC,
    entity::ENTITY_CREATE_SPEC,

    entity::ENTITY_UPDATE_SPEC,
    entity::ENTITY_DELETE_SPEC,
    entity::ENTITY_REVIVE_SPEC,
    entity::ENTITY_BATCH_GET_SPEC,
    entity::ENTITY_BATCH_FIND_SPEC,
    entity::ENTITY_BATCH_CREATE_SPEC,
    entity::ENTITY_BATCH_UPDATE_SPEC,
    entity::ENTITY_BATCH_DELETE_SPEC,
    work_schedule::WORK_SCHEDULE_READ_SPEC,
    self_update::SELF_UPDATE_SPEC,
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
