pub mod common;

use insta::{assert_json_snapshot, assert_snapshot};
use liquid::object;
use std::{env, io::BufRead};

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
    let (output, tree) = run(credentials()?, "practice")?;
    assert_snapshot!("atcoder_practice_output", output);
    assert_json_snapshot!("atcoder_practice_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

#[cfg(feature = "__test_with_credentials")]
fn credentials() -> anyhow::Result<impl BufRead> {
    use anyhow::{ensure, Context as _};
    use std::io::Cursor;

    let username =
        env::var("ATCODER_USERNAME").with_context(|| "could not read `$ATCODER_USERNAME`")?;

    let password =
        env::var("ATCODER_PASSWORD").with_context(|| "could not read `$ATCODER_PASSWORD`")?;

    let (username, password) = (username.trim(), password.trim());

    ensure!(!username.is_empty(), "`$ATCODER_USERNAME` is empty");
    ensure!(!password.is_empty(), "`$ATCODER_PASSWORD` is empty");

    Ok(Cursor::new(
        format!("{}\n{}\n", username, password).into_bytes(),
    ))
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
    )
}
