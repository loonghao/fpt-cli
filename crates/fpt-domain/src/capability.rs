mod activity;
mod auth;
mod core;
mod entity;
mod follow;
mod hierarchy;
mod note;
mod schema;
mod self_update;
mod server;
mod upload;
mod work_schedule;

use fpt_core::CommandSpec;

static COMMANDS: &[CommandSpec] = &[
    core::CAPABILITIES_SPEC,
    core::INSPECT_COMMAND_SPEC,
    auth::AUTH_TEST_SPEC,
    server::SERVER_INFO_SPEC,
    schema::SCHEMA_ENTITIES_SPEC,
    schema::SCHEMA_FIELDS_SPEC,
    schema::SCHEMA_FIELD_CREATE_SPEC,
    schema::SCHEMA_FIELD_UPDATE_SPEC,
    schema::SCHEMA_FIELD_DELETE_SPEC,
    schema::SCHEMA_FIELD_READ_SPEC,
    entity::ENTITY_GET_SPEC,
    entity::ENTITY_FIND_SPEC,
    entity::ENTITY_FIND_ONE_SPEC,
    entity::ENTITY_SUMMARIZE_SPEC,
    entity::ENTITY_CREATE_SPEC,
    entity::ENTITY_UPDATE_SPEC,
    entity::ENTITY_DELETE_SPEC,
    entity::ENTITY_REVIVE_SPEC,
    entity::ENTITY_TEXT_SEARCH_SPEC,
    entity::ENTITY_BATCH_GET_SPEC,
    entity::ENTITY_BATCH_FIND_SPEC,
    entity::ENTITY_BATCH_CREATE_SPEC,
    entity::ENTITY_BATCH_UPDATE_SPEC,
    entity::ENTITY_BATCH_DELETE_SPEC,
    entity::ENTITY_BATCH_REVIVE_SPEC,
    follow::ENTITY_FOLLOWERS_SPEC,
    follow::ENTITY_FOLLOW_SPEC,
    follow::ENTITY_UNFOLLOW_SPEC,
    follow::USER_FOLLOWING_SPEC,
    note::NOTE_THREADS_SPEC,
    note::NOTE_REPLY_CREATE_SPEC,
    hierarchy::HIERARCHY_SEARCH_SPEC,
    work_schedule::WORK_SCHEDULE_READ_SPEC,
    work_schedule::WORK_SCHEDULE_UPDATE_SPEC,
    upload::UPLOAD_URL_SPEC,
    upload::DOWNLOAD_URL_SPEC,
    upload::THUMBNAIL_URL_SPEC,
    activity::ACTIVITY_STREAM_SPEC,
    activity::EVENT_LOG_ENTRIES_SPEC,
    activity::PREFERENCES_GET_SPEC,
    entity::ENTITY_RELATIONSHIP_SPEC,
    entity::ENTITY_UPDATE_LAST_ACCESSED_SPEC,
    schema::SCHEMA_ENTITY_READ_SPEC,
    schema::SCHEMA_FIELD_REVIVE_SPEC,
    self_update::SELF_UPDATE_SPEC,
    self_update::CONFIG_GET_SPEC,
    self_update::CONFIG_PATH_SPEC,
    self_update::CONFIG_SET_SPEC,
    self_update::CONFIG_CLEAR_SPEC,
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
