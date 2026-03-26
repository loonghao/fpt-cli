use fpt_core::{CommandSpec, RiskLevel};

const SCHEDULE_WORK_DAY_RULES_EXAMPLES: &[&str] = &[
    "fpt schedule work-day-rules --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt schedule work-day-rules --input '{\"page\":{\"size\":50}}' --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEDULE_WORK_DAY_RULES_NOTES: &[&str] = &[
    "Read the site-level or project-level work day rules from ShotGrid",
    "Uses the REST schedule/work_day_rules endpoint",
];

const SCHEDULE_WORK_DAY_RULES_UPDATE_EXAMPLES: &[&str] = &[
    "fpt schedule work-day-rules-update 42 --input @rule.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEDULE_WORK_DAY_RULES_UPDATE_NOTES: &[&str] = &[
    "Update a specific work day rule by its record id",
    "Uses the REST PUT schedule/work_day_rules/{id} endpoint",
];

pub const SCHEDULE_WORK_DAY_RULES_SPEC: CommandSpec = CommandSpec {
    name: "schedule.work-day-rules",
    summary: "Read work day rules from the ShotGrid scheduling system",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "optional query params JSON (page, etc.)",
    output: "json",
    examples: SCHEDULE_WORK_DAY_RULES_EXAMPLES,
    notes: SCHEDULE_WORK_DAY_RULES_NOTES,
};

pub const SCHEDULE_WORK_DAY_RULES_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "schedule.work-day-rules-update",
    summary: "Update a specific work day rule in the ShotGrid scheduling system",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "work day rule update body JSON",
    output: "json",
    examples: SCHEDULE_WORK_DAY_RULES_UPDATE_EXAMPLES,
    notes: SCHEDULE_WORK_DAY_RULES_UPDATE_NOTES,
};
