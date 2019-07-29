use std::process::Command;

fn env_test_ark_kv_bin() -> String {
    std::env::var("TEST_ARK_KV_BIN")
        .expect("TEST_ARK_KV_BIN is undefined, integration test disabled")
}

pub fn command_create() -> Command {
    std::env::set_var("DATABASE_URL", "test.sqlite3");
    Command::new(env_test_ark_kv_bin())
}
