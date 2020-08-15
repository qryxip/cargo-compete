pub mod common;

use ignore::overrides::OverrideBuilder;
use insta::{assert_json_snapshot, assert_snapshot};
use liquid::object;
use once_cell::sync::Lazy;
use regex::Regex;
use snowchains_core::web::PlatformKind;
use std::io;

#[test]
fn atcoder_practice_a() -> anyhow::Result<()> {
    let (output, tree) = run(
        PlatformKind::Atcoder,
        "practice",
        "a",
        r#"---
type: Batch
timelimit: 2s
match: Lines

cases:
  - name: sample1
    in: |
      1
      2 3
      test
    out: |
      6 test
  - name: sample2
    in: |
      72
      128 256
      myonmyon
    out: |
      456 myonmyon

extend: []
"#,
        r#"use proconio::input;

fn main() {
    input! {
        a: u32,
        b: u32,
        c: u32,
        s: String,
    }

    println!("{} {}", a + b + c, s);
}
"#,
    )?;

    assert_snapshot!("atcoder_practice_a_output", output);
    assert_json_snapshot!("atcoder_practice_a_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

fn run(
    platform: PlatformKind,
    contest: &str,
    problem: &str,
    test_suite: &str,
    code: &str,
) -> anyhow::Result<(String, serde_json::Value)> {
    common::run(
        |workspace_root| -> _ {
            std::fs::write(
                workspace_root.join("Cargo.toml"),
                format!(
                    r#"[workspace]
members = ["{}"]
"#,
                    contest,
                ),
            )?;

            std::fs::write(
                workspace_root.join("compete.toml"),
                liquid::ParserBuilder::with_stdlib()
                    .build()?
                    .parse(include_str!("../resources/compete.toml.liquid"))?
                    .render(&object!({
                        "template_platform": platform.to_kebab_case_str(),
                        "submit_via_binary": false,
                    }))?,
            )?;

            std::fs::create_dir_all(workspace_root.join("testcases").join(contest))?;

            std::fs::write(
                workspace_root
                    .join("testcases")
                    .join(contest)
                    .join(problem)
                    .with_extension("yml"),
                test_suite,
            )?;

            std::fs::create_dir_all(workspace_root.join(contest).join("src").join("bin"))?;

            std::fs::write(
                workspace_root.join(contest).join("Cargo.toml"),
                format!(
                    r#"[package]
name = "problems"
version = "0.1.0"
edition = "2018"

[package.metadata.cargo-compete.bin]
{problem} = {{ name = "{contest}-{problem}", problem = {{ platform = "{platform}", contest = "{contest}", index = "{problem}" }} }}

[[bin]]
name = "{contest}-{problem}"
path = "src/bin/{problem}.rs"

[dependencies]
proconio = "0.3.6"
"#,
                    contest = contest,
                    problem = problem,
                    platform = platform.to_kebab_case_str(),
                ),
            )?;

            std::fs::write(
                workspace_root
                    .join(contest)
                    .join("src")
                    .join("bin")
                    .join(problem)
                    .with_extension("rs"),
                code,
            )?;
            Ok(())
        },
        io::empty(),
        &[
            "",
            "compete",
            "t",
            problem,
            "--manifest-path",
            &format!("./{}/Cargo.toml", contest),
        ],
        |_, output| {
            macro_rules! lazy_regex(($regex:literal) => (Lazy::new(|| Regex::new($regex).unwrap())));

            static RUNNING: Lazy<Regex> = lazy_regex!("^     Running `[^`]+`");
            static ACCEPTED: Lazy<Regex> = lazy_regex!(r"Accepted \([0-9]+ ms\)");

            let output = RUNNING.replace(&output, "     Running {{ command }}");
            let output = ACCEPTED.replace_all(&output, "Accepted ({{ elapsed }}) ms)");
            output.into_owned()
        },
        |workspace_root| {
            OverrideBuilder::new(workspace_root)
                .add("!/target")?
                .build()
        },
    )
}
