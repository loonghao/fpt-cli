use fpt_domain::entity_collection_path;
use rstest::rstest;

#[rstest]
#[case("Shot", "shots")]
#[case("Asset", "assets")]
#[case("Version", "versions")]
#[case("CustomNonProjectEntity01", "custom_non_project_entity_01s")]
// Single-word entities
#[case("Task", "tasks")]
#[case("Note", "notes")]
#[case("Playlist", "playlists")]
// Already plural entities
#[case("EventLogEntries", "event_log_entries")]
// Multi-uppercase sequences
#[case("HumanUser", "human_users")]
#[case("ApiUser", "api_users")]
// Entities with trailing digits
#[case("CustomEntity02", "custom_entity_02s")]
#[case("CustomEntity10", "custom_entity_10s")]
// Whitespace trimming
#[case("  Shot  ", "shots")]
// Hyphenated input
#[case("My-Entity", "my_entitys")]
// Entity ending with 's'
#[case("Address", "address")]
#[case("Status", "status")]
fn converts_entity_type_to_rest_collection_path(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(entity_collection_path(input), expected);
}
