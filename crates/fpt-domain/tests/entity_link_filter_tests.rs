use fpt_domain::parse_filter_dsl;
use rstest::rstest;
use serde_json::json;

// ---------------------------------------------------------------
// Issue #42: entity-link shorthand in filter_dsl
// ---------------------------------------------------------------

#[test]
fn dsl_entity_link_shorthand_project() {
    let filters = parse_filter_dsl("project is Project:123").expect("dsl should parse");
    assert_eq!(
        filters,
        json!({
            "logical_operator": "and",
            "conditions": [
                ["project", "is", {"type": "Project", "id": 123}]
            ]
        })
    );
}

#[test]
fn dsl_entity_link_shorthand_sequence() {
    let filters = parse_filter_dsl("sg_sequence is Sequence:45").expect("dsl should parse");
    assert_eq!(
        filters,
        json!({
            "logical_operator": "and",
            "conditions": [
                ["sg_sequence", "is", {"type": "Sequence", "id": 45}]
            ]
        })
    );
}

#[test]
fn dsl_entity_link_shorthand_in_complex_expression() {
    let filters = parse_filter_dsl(
        "project is Project:100 and (sg_status_list == 'ip' or sg_sequence is Sequence:10)",
    )
    .expect("dsl should parse");
    assert_eq!(
        filters,
        json!({
            "logical_operator": "and",
            "conditions": [
                ["project", "is", {"type": "Project", "id": 100}],
                {
                    "logical_operator": "or",
                    "conditions": [
                        ["sg_status_list", "is", "ip"],
                        ["sg_sequence", "is", {"type": "Sequence", "id": 10}]
                    ]
                }
            ]
        })
    );
}

#[test]
fn dsl_entity_link_shorthand_is_not() {
    let filters = parse_filter_dsl("project != Project:99").expect("dsl should parse");
    assert_eq!(
        filters,
        json!({
            "logical_operator": "and",
            "conditions": [
                ["project", "is_not", {"type": "Project", "id": 99}]
            ]
        })
    );
}

#[test]
fn dsl_entity_link_shorthand_custom_type() {
    let filters = parse_filter_dsl("entity is CustomEntity:42").expect("dsl should parse");
    assert_eq!(
        filters,
        json!({
            "logical_operator": "and",
            "conditions": [
                ["entity", "is", {"type": "CustomEntity", "id": 42}]
            ]
        })
    );
}

// The full JSON object form should still work
#[test]
fn dsl_entity_link_full_json_object() {
    let filters = parse_filter_dsl("project is {\"type\": \"Project\", \"id\": 123}")
        .expect("dsl should parse");
    assert_eq!(
        filters,
        json!({
            "logical_operator": "and",
            "conditions": [
                ["project", "is", {"type": "Project", "id": 123}]
            ]
        })
    );
}

// ---------------------------------------------------------------
// Improved error messages for common mistakes
// ---------------------------------------------------------------

#[test]
fn dsl_unquoted_string_value_gives_helpful_error() {
    let err = parse_filter_dsl("code == hello").expect_err("should fail for unquoted string");
    let msg = err.envelope().message;
    assert!(
        msg.contains("hello") || msg.contains("Wrap string values"),
        "error should mention the value or suggest quoting: {msg}"
    );
}

// ---------------------------------------------------------------
// Existing DSL tests continue to pass
// ---------------------------------------------------------------

#[test]
fn parses_nested_boolean_expression_to_search_filters() {
    let filters = parse_filter_dsl("sg_status_list == 'ip' and (code ~ 'bunny' or id > 100)")
        .expect("dsl should parse");
    assert_eq!(
        filters,
        json!({
            "logical_operator": "and",
            "conditions": [
                ["sg_status_list", "is", "ip"],
                {
                    "logical_operator": "or",
                    "conditions": [
                        ["code", "contains", "bunny"],
                        ["id", "greater_than", 100]
                    ]
                }
            ]
        })
    );
}

#[test]
fn parses_keyword_operators_and_array_values() {
    let filters =
        parse_filter_dsl("sg_status in ['ip', 'wtg'] and id >= 10").expect("dsl should parse");
    assert_eq!(
        filters,
        json!({
            "logical_operator": "and",
            "conditions": [
                ["sg_status", "in", ["ip", "wtg"]],
                ["id", "greater_than_or_equal", 10]
            ]
        })
    );
}

#[rstest]
#[case("", "cannot be empty")]
#[case("(sg_status == 'ip'", "missing closing parenthesis")]
#[case("sg_status ==", "missing operand")]
fn rejects_invalid_dsl(#[case] input: &str, #[case] expected_message: &str) {
    let error = parse_filter_dsl(input).expect_err("dsl should fail");
    let message = error.envelope().message;
    assert!(
        message.contains(expected_message),
        "message `{message}` should contain `{expected_message}`"
    );
}
