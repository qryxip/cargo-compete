pub mod common;

use ignore::overrides::Override;
use insta::{assert_json_snapshot, assert_snapshot};
use std::io::BufRead;

#[test]
fn atcoder_agc047() -> anyhow::Result<()> {
    let (output, tree) = run(&b""[..], "agc047")?;
    assert_snapshot!("atcoder_agc047_output", output);
    assert_json_snapshot!("atcoder_agc047_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

#[cfg(feature = "__test_with_credentials")]
#[test]
fn atcoder_practice() -> anyhow::Result<()> {
    let (output, tree) = run(common::atcoder_credentials()?, "practice")?;
    assert_snapshot!("atcoder_practice_output", output);
    assert_json_snapshot!("atcoder_practice_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

fn run(
    input: impl BufRead + 'static,
    contest: &str,
) -> anyhow::Result<(String, serde_json::Value)> {
    common::run(
        |workspace_root| -> _ {
            std::fs::write(
                workspace_root.join("Cargo.toml"),
                r#"[workspace]
"#,
            )?;

            std::fs::write(
                workspace_root.join("compete.toml"),
                r#"new-workspace-member = "include"
test-suite = "./testcases/{{ contest }}/{{ problem | kebabcase }}.yml"

[template]
platform = "atcoder"
manifest = "./template-manifest.toml"
src = "./template-code.rs"
"#,
            )?;

            std::fs::write(
                workspace_root.join("template-manifest.toml"),
                r#"[package]
name = "cargo-compete-template"
version = "0.1.0"
edition = "2018"
"#,
            )?;

            std::fs::write(
                workspace_root.join("template-code.rs"),
                r#"fn main() {
    todo!();
}
"#,
            )?;
            Ok(())
        },
        input,
        &["", "compete", "n", contest],
        |workspace_root, output| {
            output
                .replace(workspace_root.to_str().unwrap(), "{{ cwd }}")
                .replace('/', "{{ slash_or_backslash }}")
                .replace('\\', "{{ slash_or_backslash }}")
        },
        |_| Ok(Override::empty()),
    )
}
