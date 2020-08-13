#![cfg(feature = "__test_with_credentials")]

pub mod common;

use insta::{assert_json_snapshot, assert_snapshot};

#[test]
fn atcoder() -> anyhow::Result<()> {
    let (output, tree) = run("atcoder")?;
    assert_snapshot!("atcoder_output", output);
    assert_json_snapshot!("atcoder_file_tree", tree);
    Ok(())
}

fn run(platform: &str) -> anyhow::Result<(String, serde_json::Value)> {
    common::run(
        |_| Ok(()),
        common::atcoder_credentials()?,
        &["", "compete", "l", platform],
        |_, output| output,
    )
}
