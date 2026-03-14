use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn capabilities_output_includes_self_update_contract() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"self-update\""));
}

#[test]
fn inspect_self_update_mentions_checksum_verification() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "self-update", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("fpt-checksums.txt"));
}

#[test]
fn self_update_help_mentions_check_mode() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["self-update", "--help"]);

    command.assert().success().stdout(
        predicate::str::contains("released fpt binary")
            .and(predicate::str::contains("--check"))
            .and(predicate::str::contains("--repository")),
    );
}
