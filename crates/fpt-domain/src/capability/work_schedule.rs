use fpt_core::{CommandSpec, RiskLevel};

const WORK_SCHEDULE_READ_EXAMPLES: &[&str] = &[
    "fpt work-schedule read --input @schedule.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const WORK_SCHEDULE_NOTES: &[&str] = &[
    "Uses the ShotGrid RPC `work_schedule_read` method over `/api3/json`",
    "Input JSON must include `start_date` and `end_date`, with optional `project` and `user` entity links",
    "Returns the per-day working calendar exactly as provided by ShotGrid",
];

pub const WORK_SCHEDULE_READ_SPEC: CommandSpec = CommandSpec {
    name: "work-schedule.read",
    summary: "Read the ShotGrid work schedule for a date range",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "input JSON with start_date/end_date and optional project/user",
    output: "json",
    examples: WORK_SCHEDULE_READ_EXAMPLES,
    notes: WORK_SCHEDULE_NOTES,
};
