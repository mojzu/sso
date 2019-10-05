mod support;

use support::*;

// TODO(test): Integration tests.

#[test]
#[ignore]
fn cli_help() {
    let output = command_create().arg("--help").output().unwrap();
    String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
}

#[test]
#[ignore]
fn cli_version() {
    let output = command_create().arg("--version").output().unwrap();
    String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
}

#[test]
#[ignore]
fn cli_create_and_verify_secret_key() {
    let output = command_create()
        .args(&["create", "secret-key", "./tests/secret_key"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());

    let output = command_create()
        .args(&["verify", "secret-key", "./tests/secret_key"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());

    std::fs::remove_file("./tests/secret_key").unwrap();
}
