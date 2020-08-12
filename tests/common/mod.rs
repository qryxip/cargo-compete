use cargo_compete::{shell::Shell, Opt};
use ignore::WalkBuilder;
use serde_json::json;
use std::{io::BufRead, path::Path};
use structopt::StructOpt as _;

pub fn run(
    before: impl FnOnce(&Path) -> anyhow::Result<()>,
    input: impl BufRead + 'static,
    args: &[&str],
    process_output: impl FnOnce(&Path, String) -> String,
) -> anyhow::Result<(String, serde_json::Value)> {
    let workspace = tempfile::Builder::new()
        .prefix("cargo-compete-test-workspace")
        .tempdir()?;

    let cookies_jsonl = tempfile::Builder::new()
        .prefix("cargo-compete-test-cookies")
        .suffix(".jsonl")
        .tempfile()?
        .into_temp_path();

    let (output_file, output) = tempfile::Builder::new()
        .prefix("cargo-compete-test-output")
        .tempfile()?
        .into_parts();

    before(workspace.path())?;

    let Opt::Compete(opt) = Opt::from_iter_safe(args)?;

    cargo_compete::run(
        opt,
        cargo_compete::Context {
            cwd: workspace.path().to_owned(),
            cookies_path: Path::new(&cookies_jsonl).to_owned(),
            shell: &mut Shell::from_read_write(Box::new(input), Box::new(output_file)),
        },
    )?;

    let output_content = process_output(workspace.path(), std::fs::read_to_string(&output)?);
    let tree = tree(workspace.as_ref())?;

    workspace.close()?;
    output.close()?;

    Ok((output_content, tree))
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

        macro_rules! enter {
            ($components:expr) => {
                for &component in $components {
                    tree = tree
                        .entry(component)
                        .or_insert_with(|| json!({}))
                        .as_object_mut()
                        .unwrap();
                }
            };
        }

        if entry.path().is_dir() {
            enter!(&components);
        } else if let [components @ .., file_name] = &*components {
            enter!(components);
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
