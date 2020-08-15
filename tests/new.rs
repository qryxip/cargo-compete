pub mod common;

use ignore::overrides::Override;
use insta::{assert_json_snapshot, assert_snapshot};
use liquid::object;
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
            std::fs::create_dir_all(workspace_root.join("cargo-compete-template").join("src"))?;

            std::fs::write(
                workspace_root.join("Cargo.toml"),
                r#"[workspace]
members = ["cargo-compete-template"]
"#,
            )?;

            std::fs::write(
                workspace_root.join("compete.toml"),
                liquid::ParserBuilder::with_stdlib()
                    .build()?
                    .parse(include_str!("../resources/compete.toml.liquid"))?
                    .render(&object!({
                        "template_platform": "atcoder",
                        "submit_via_binary": false,
                    }))?,
            )?;

            std::fs::write(
                workspace_root
                    .join("cargo-compete-template")
                    .join("Cargo.toml"),
                r#"[package]
name = "cargo-compete-template"
version = "0.1.0"
edition = "2018"
"#,
            )?;

            std::fs::write(
                workspace_root
                    .join("cargo-compete-template")
                    .join("src")
                    .join("main.rs"),
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
