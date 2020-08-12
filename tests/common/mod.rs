use ignore::WalkBuilder;
use serde_json::json;
use std::path::Path;

pub fn tree(path: &Path) -> anyhow::Result<serde_json::Value> {
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
