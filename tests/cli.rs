use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn once_echo_outputs_ack() {
    let mut cmd = Command::cargo_bin("femtoclaw").unwrap();
    cmd.env("FEMTO_BRAIN", "echo").arg("once").arg("hello");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ACK: hello"));
}
