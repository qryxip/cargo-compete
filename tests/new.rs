pub mod common;

use ignore::overrides::Override;
use insta::{assert_json_snapshot, assert_snapshot};
use snowchains_core::web::PlatformKind;
use std::io::BufRead;

#[test]
fn atcoder_abc003() -> anyhow::Result<()> {
    let (output, tree) = run(PlatformKind::Atcoder, "abc003", &b""[..])?;
    assert_snapshot!("atcoder_abc003_output", output);
    assert_json_snapshot!("atcoder_abc003_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

#[test]
fn atcoder_abc007() -> anyhow::Result<()> {
    let (output, tree) = run(PlatformKind::Atcoder, "abc007", &b""[..])?;
    assert_snapshot!("atcoder_abc007_output", output);
    assert_json_snapshot!("atcoder_abc007_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

#[test]
fn atcoder_agc047() -> anyhow::Result<()> {
    let (output, tree) = run(PlatformKind::Atcoder, "agc047", &b""[..])?;
    assert_snapshot!("atcoder_agc047_output", output);
    assert_json_snapshot!("atcoder_agc047_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

#[cfg(feature = "__test_with_credentials")]
#[test]
fn atcoder_practice() -> anyhow::Result<()> {
    let (output, tree) = run(
        PlatformKind::Atcoder,
        "practice",
        common::atcoder_credentials()?,
    )?;
    assert_snapshot!("atcoder_practice_output", output);
    assert_json_snapshot!("atcoder_practice_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

#[test]
fn yukicoder_contest_100() -> anyhow::Result<()> {
    let (output, tree) = run(PlatformKind::Yukicoder, "100", &b""[..])?;
    assert_snapshot!("yukicoder_contest_100_output", output);
    assert_json_snapshot!("yukicoder_contest_100_file_tree", tree, { r#".**["Cargo.lock"]"# => ".." });
    Ok(())
}

fn run(
    platform: PlatformKind,
    contest: &str,
    input: impl BufRead + 'static,
) -> anyhow::Result<(String, serde_json::Value)> {
    common::run(
        |cwd| -> _ {
            std::fs::write(
                cwd.join("compete.toml"),
                format!(
                    r#"test-suite = "{{{{ manifest_dir }}}}/testcases/{{{{ bin_alias | kebabcase }}}}.yml"

[new]
platform = "{}"
path = "./{{{{ package_name }}}}"

[new.template.dependencies]
kind = "inline"
content = '''
proconio = "=0.3.6"
'''

[new.template.src]
kind = "inline"
content = '''
fn main() {{
    todo!();
}}
'''
"#,
                    platform.to_kebab_case_str(),
                ),
            )?;

            std::fs::create_dir(cwd.join(".cargo"))?;

            std::fs::write(
                cwd.join(".cargo").join("config.toml"),
                r#"[cargo-new]
name = ""
email = ""
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
