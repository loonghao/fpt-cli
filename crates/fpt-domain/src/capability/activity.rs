use fpt_core::{CommandSpec, RiskLevel};

const ACTIVITY_STREAM_EXAMPLES: &[&str] = &[
    "fpt activity stream Shot 123 --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt activity stream Version 456 --page-size 20 --site ...",
    "fpt activity stream Task 789 --entity-fields code,sg_status_list --site ...",
];

const EVENT_LOG_ENTRIES_EXAMPLES: &[&str] = &[
    "fpt event-log entries --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt event-log entries --fields id,event_type,created_at,entity --page-size 50 --site ...",
    "fpt event-log entries --filter 'event_type[is]=Shotgun_Shot_Change' --site ...",
];

const PREFERENCES_GET_EXAMPLES: &[&str] =
    &["fpt preferences get --site ... --auth-mode script --script-name ... --script-key ..."];

const ACTIVITY_STREAM_NOTES: &[&str] = &[
    "Uses the REST endpoint GET /entity/{type}/{id}/activity_stream",
    "Returns a paginated list of activity records for the specified entity",
    "Use --page-size to control the number of records returned per page",
    "Use --entity-fields to request additional fields on linked entities",
];

const EVENT_LOG_ENTRIES_NOTES: &[&str] = &[
    "Uses the REST endpoint GET /entity/event_log_entries",
    "Returns a paginated list of ShotGrid event log entries",
    "Supports standard entity find query parameters: fields, sort, page, filters",
    "Useful for auditing changes and building event-driven integrations",
];

const PREFERENCES_GET_NOTES: &[&str] = &[
    "Uses the REST endpoint GET /preferences",
    "Returns site-level ShotGrid preferences and configuration",
    "Read-only; no input required beyond authentication",
];

pub const ACTIVITY_STREAM_SPEC: CommandSpec = CommandSpec {
    name: "activity.stream",
    summary: "Fetch the activity stream for a specific entity record",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + optional query params",
    output: "json activity records",
    examples: ACTIVITY_STREAM_EXAMPLES,
    notes: ACTIVITY_STREAM_NOTES,
};

pub const EVENT_LOG_ENTRIES_SPEC: CommandSpec = CommandSpec {
    name: "event-log.entries",
    summary: "Query ShotGrid event log entries",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "optional query params (fields, sort, page, filters)",
    output: "json event log records",
    examples: EVENT_LOG_ENTRIES_EXAMPLES,
    notes: EVENT_LOG_ENTRIES_NOTES,
};

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
