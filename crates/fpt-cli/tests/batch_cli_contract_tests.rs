use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;

#[test]
fn capabilities_outputs_entity_batch_update_contract() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entity.batch.update\""));
}

#[rstest]
#[case("entity.batch.create")]
#[case("entity.batch.update")]
#[case("entity.batch.delete")]
fn inspect_batch_commands_show_dry_run_support(#[case] command_name: &str) {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", command_name, "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"supports_dry_run\":true"));
}

#[test]
fn entity_batch_update_dry_run_outputs_multiple_request_plans() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "entity",
        "batch",
        "update",
        "Task",
        "--input",
        "[{\"id\":42,\"body\":{\"data\":{\"type\":\"Task\",\"id\":42}}},{\"id\":43,\"body\":{\"data\":{\"type\":\"Task\",\"id\":43}}}]",
        "--dry-run",
        "--output",
        "json",
    ]);

    command.assert().success().stdout(
        predicate::str::contains("\"dry_run\":true")
            .and(predicate::str::contains(
                "\"operation\":\"entity.batch.update\"",
            ))
            .and(predicate::str::contains("/api/v1.1/entity/tasks/42"))
            .and(predicate::str::contains("/api/v1.1/entity/tasks/43")),
    );
}

#[test]
fn batch_delete_without_yes_is_blocked() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "entity", "batch", "delete", "Task", "--input", "[42,43]", "--output", "json",
    ]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("POLICY_BLOCKED"));
}
