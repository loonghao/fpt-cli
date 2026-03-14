use fpt_domain::parse_filter_dsl;
use rstest::rstest;
use serde_json::json;

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
    let filters = parse_filter_dsl("sg_status in ['ip', 'wtg'] and id >= 10")
        .expect("dsl should parse");

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
