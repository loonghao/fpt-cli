use fpt_core::{CommandSpec, RiskLevel};

const NOTE_THREADS_EXAMPLES: &[&str] =
    &["fpt note threads 456 --site ... --auth-mode script --script-name ... --script-key ..."];

const NOTE_NOTES: &[&str] = &[
    "Retrieve the full reply thread for a top-level Note entity",
    "The positional note_id must be a top-level Note record id accepted by GET /entity/notes/{record_id}/thread_contents",
    "Supports optional query parameters via --input JSON (fields, entity_fields, etc.)",
];

pub const NOTE_THREADS_SPEC: CommandSpec = CommandSpec {
    name: "note.threads",
    summary: "Get the reply thread contents for a top-level Note record",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "top-level note_id + optional query params JSON",
    output: "json",
    examples: NOTE_THREADS_EXAMPLES,
    notes: NOTE_NOTES,
};

const NOTE_REPLY_CREATE_EXAMPLES: &[&str] = &[
    "fpt note reply-create 456 --input '{\"content\":\"Looks great!\"}' --site ... --auth-mode script --script-name ... --script-key ...",
];

const NOTE_REPLY_CREATE_NOTES: &[&str] = &[
    "Creates a new reply in a top-level Note thread via POST /entity/notes/{note_id}/thread_contents",
    "Input JSON must include a `content` field with the reply text",
    "The note_id must be a top-level Note record id",
];

pub const NOTE_REPLY_CREATE_SPEC: CommandSpec = CommandSpec {
    name: "note.reply-create",
    summary: "Create a reply in a top-level Note thread",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "top-level note_id + reply body JSON with content",
    output: "json",
    examples: NOTE_REPLY_CREATE_EXAMPLES,
    notes: NOTE_REPLY_CREATE_NOTES,
};
