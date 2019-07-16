mod support;

use support::*;

#[test]
#[ignore]
fn cli_help() {
    let output = command_create().arg("--help").output().unwrap();
    String::from_utf8(output.stdout).unwrap();
}

#[test]
#[ignore]
fn cli_version() {
    let output = command_create().arg("--version").output().unwrap();
    String::from_utf8(output.stdout).unwrap();
}
