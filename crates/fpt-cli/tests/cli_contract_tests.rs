use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn capabilities_uses_json_as_default_output() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\":\"fpt\""));
}

#[test]
fn capabilities_still_supports_explicit_toon_output() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "toon"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("name: fpt"));
}


#[test]
fn capabilities_outputs_entity_update_contract() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entity.update\""));
}


#[test]
fn capabilities_lists_user_password_auth_mode() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command.assert().success().stdout(
        predicate::str::contains("\"user_password\"")
            .and(predicate::str::contains("\"FPT_USERNAME\""))
            .and(predicate::str::contains("\"FPT_SESSION_TOKEN\"")),
    );
}

#[test]
fn inspect_command_shows_dry_run_support() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "entity.update", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"supports_dry_run\":true"));
}

#[test]
fn inspect_auth_test_mentions_session_token_mode() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "auth.test", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("session-token"));
}

#[test]
fn inspect_entity_find_mentions_structured_search() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "entity.find", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(
            predicate::str::contains("filter-dsl")
                .and(predicate::str::contains("structured `search` object"))
                .and(predicate::str::contains("additional_filter_presets")),
        );
}

#[test]
fn inspect_entity_find_one_is_registered() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "entity.find-one", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(
            predicate::str::contains("first matching record or null")
                .and(predicate::str::contains("same as entity.find")),
        );
}

#[test]
fn inspect_entity_summarize_is_registered() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "entity.summarize", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(
            predicate::str::contains("ShotGrid summary operators")
                .and(predicate::str::contains("summary_fields"))
                .and(predicate::str::contains("summarize")),
        );
}

#[test]
fn entity_update_dry_run_outputs_rest_request_plan() {


    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "entity",
        "update",
        "Task",
        "42",
        "--input",
        "{\"data\":{\"type\":\"Task\",\"id\":42}}",
        "--dry-run",
        "--output",
        "json",
    ]);

    command.assert().success().stdout(
        predicate::str::contains("\"dry_run\":true")
            .and(predicate::str::contains("/api/v1.1/entity/tasks/42")),
    );
}

#[test]
fn work_schedule_read_requires_input_json() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["work-schedule", "read", "--output", "json"]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments were not provided"));
}

#[test]
fn delete_without_yes_is_blocked() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["entity", "delete", "Task", "42", "--output", "json"]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("POLICY_BLOCKED"));
}

