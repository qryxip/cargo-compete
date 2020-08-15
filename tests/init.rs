pub mod common;

use duct::cmd;
use ignore::overrides::Override;
use insta::{assert_json_snapshot, assert_snapshot};
use std::str;

#[test]
fn no_crate() -> anyhow::Result<()> {
    let (output, tree) = run("\n1\n")?;
    assert_snapshot!("no_crate_output", output);
    assert_json_snapshot!("no_crate_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

#[test]
fn use_crate() -> anyhow::Result<()> {
    let (output, tree) = run("\n2\n")?;
    assert_snapshot!("use_crate_output", output);
    assert_json_snapshot!("use_crate_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

#[test]
fn use_crate_via_bianry() -> anyhow::Result<()> {
    let (output, tree) = run("\n3\n")?;
    assert_snapshot!("use_crate_via_bianry_output", output);
    assert_json_snapshot!("use_crate_via_bianry_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

fn run(input: &'static str) -> anyhow::Result<(String, serde_json::Value)> {
    common::run(
        |workspace_root| -> _ {
            println!("{}", cmd!("git", "init", workspace_root).read()?);
            Ok(())
        },
        input.as_bytes(),
        &["", "compete", "i"],
        |workspace_root, output| {
            output
                .replace(workspace_root.to_str().unwrap(), "{{ cwd }}")
                .replace(std::path::MAIN_SEPARATOR, "{{ main_path_separator }}")
        },
        |_| Ok(Override::empty()),
    )
}
