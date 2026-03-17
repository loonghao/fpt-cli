use fpt_core::{CommandSpec, RiskLevel};

const FOLLOWERS_EXAMPLES: &[&str] = &[
    "fpt followers list Shot 123 --site ... --auth-mode script --script-name ... --script-key ...",
];

const FOLLOW_EXAMPLES: &[&str] = &[
    "fpt followers follow Shot 123 --input '{\"type\":\"HumanUser\",\"id\":456}' --site ...",
];

const UNFOLLOW_EXAMPLES: &[&str] = &[
    "fpt followers unfollow Shot 123 --input '{\"type\":\"HumanUser\",\"id\":456}' --site ...",
];

const FOLLOW_NOTES: &[&str] = &[
    "Followers endpoints manage which users are following a given entity record",
    "The user payload must be a JSON object with `type` and `id` fields",
];

pub const ENTITY_FOLLOWERS_SPEC: CommandSpec = CommandSpec {
    name: "followers.list",
    summary: "List all followers of an entity record",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id",
    output: "json",
    examples: FOLLOWERS_EXAMPLES,
    notes: FOLLOW_NOTES,
};

pub const ENTITY_FOLLOW_SPEC: CommandSpec = CommandSpec {
    name: "followers.follow",
    summary: "Add a user as a follower of an entity record",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + user JSON",
    output: "json",
    examples: FOLLOW_EXAMPLES,
    notes: FOLLOW_NOTES,
};

pub const ENTITY_UNFOLLOW_SPEC: CommandSpec = CommandSpec {
    name: "followers.unfollow",
    summary: "Remove a user from the followers of an entity record",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + user JSON",
    output: "json",
    examples: UNFOLLOW_EXAMPLES,
    notes: FOLLOW_NOTES,
};
