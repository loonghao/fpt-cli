use fpt_domain::entity_collection_path;
use rstest::rstest;

#[rstest]
#[case("Shot", "shots")]
#[case("Asset", "assets")]
#[case("Version", "versions")]
#[case("CustomNonProjectEntity01", "custom_non_project_entity_01s")]
fn converts_entity_type_to_rest_collection_path(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(entity_collection_path(input), expected);
}
