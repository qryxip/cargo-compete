use cargo_compete::{shell::Shell, Opt};
use duct::cmd;
use ignore::WalkBuilder;
use insta::{assert_json_snapshot, assert_snapshot};
use serde_json::json;
use std::{path::Path, str};
use structopt::StructOpt as _;

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
    let workspace = tempfile::Builder::new()
        .prefix("cargo-compete-test-workspace")
        .tempdir()?;

    let (output_file, output) = tempfile::Builder::new()
        .prefix("cargo-compete-test-output")
        .tempfile()?
        .into_parts();

    println!("{}", cmd!("git", "init", workspace.path()).read()?);

    let Opt::Compete(opt) = Opt::from_iter_safe(&["", "compete", "i"])?;

    cargo_compete::run(
        opt,
        cargo_compete::Context {
            cwd: workspace.path().to_owned(),
            shell: &mut Shell::from_read_write(Box::new(input.as_bytes()), Box::new(output_file)),
        },
    )?;

    let output_masked = std::fs::read_to_string(&output)?
        .replace(workspace.path().to_str().unwrap(), "{{ cwd }}")
        .replace(std::path::MAIN_SEPARATOR, "{{ separator }}");

    let tree = tree(workspace.as_ref())?;

    workspace.close()?;
    output.close()?;

    Ok((output_masked, tree))
}

fn tree(path: &Path) -> anyhow::Result<serde_json::Value> {
    let mut tree = serde_json::Map::new();

    for entry in WalkBuilder::new(path)
        .git_ignore(false)
        .sort_by_file_name(Ord::cmp)
        .build()
    {
        let entry = entry?;

        let components = entry
            .path()
            .strip_prefix(path)?
            .iter()
            .map(|p| p.to_str().unwrap())
            .collect::<Vec<_>>();

        let mut tree = &mut tree;
        if entry.path().is_dir() {
            for component in components {
                tree = tree
                    .entry(component)
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
                    .unwrap();
            }
        } else if let [components @ .., file_name] = &*components {
            for &component in components {
                tree = tree
                    .entry(component)
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
                    .unwrap();
            }
            tree.insert(
                (*file_name).to_owned(),
                json!(std::fs::read_to_string(entry.path())?),
            );
        } else {
            panic!();
        }
    }

    Ok(serde_json::Value::Object(tree))
}
