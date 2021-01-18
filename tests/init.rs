pub mod common;

use ignore::overrides::Override;
use insta::{assert_json_snapshot, assert_snapshot};
use std::str;

#[test]
fn atcoder_no_crate() -> anyhow::Result<()> {
    let (output, tree) = run("atcoder", ".", "1\n")?;
    assert_snapshot!("atcoder_no_crate_output", output);
    assert_json_snapshot!("atcoder_no_crate_file_tree", tree);
    Ok(())
}

#[test]
fn atcoder_use_crate() -> anyhow::Result<()> {
    let (output, tree) = run("atcoder", ".", "2\n")?;
    assert_snapshot!("atcoder_use_crate_output", output);
    assert_json_snapshot!("atcoder_use_crate_file_tree", tree);
    Ok(())
}

#[test]
fn atcoder_use_crate_via_bianry() -> anyhow::Result<()> {
    let (output, tree) = run("atcoder", ".", "3\n")?;
    assert_snapshot!("atcoder_use_crate_via_bianry_output", output);
    assert_json_snapshot!("atcoder_use_crate_via_bianry_file_tree", tree);
    Ok(())
}

#[test]
fn codeforces() -> anyhow::Result<()> {
    let (output, tree) = run("codeforces", ".", "")?;
    assert_snapshot!("codeforces_output", output);
    assert_json_snapshot!("codeforces_file_tree", tree);
    Ok(())
}

#[test]
fn codeforces_with_path() -> anyhow::Result<()> {
    let (output, tree) = run("codeforces", "codeforces", "")?;
    assert_snapshot!("codeforces_with_path_output", output);
    assert_json_snapshot!("codeforces_with_path_file_tree", tree);
    Ok(())
}

fn run(
    platform: &str,
    path: &str,
    input: &'static str,
) -> anyhow::Result<(String, serde_json::Value)> {
    common::run(
        |_| Ok(()),
        input.as_bytes(),
        &["", "compete", "i", platform, path],
        |cwd, output| {
            output
                .replace(&*cwd.to_string_lossy(), "{{ cwd }}")
                .replace(std::path::MAIN_SEPARATOR, "{{ main_path_separator }}")
        },
        |_| Ok(Override::empty()),
    )
}
