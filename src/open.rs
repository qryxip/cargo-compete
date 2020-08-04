use crate::shell::Shell;
use anyhow::{ensure, Context as _};
use serde_json::json;
use std::{borrow::Borrow, path::Path};
use url::Url;

pub(crate) fn open(
    urls: &[impl Borrow<Url>],
    open: Option<impl AsRef<str>>,
    paths: &[(impl AsRef<Path>, impl AsRef<Path>)],
    pkg_manifest_dir: &Path,
    cwd: &Path,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    for url in urls {
        let url = url.borrow();
        shell.status("Opening", url)?;
        opener::open(url.as_str())?;
    }

    if let Some(open) = open {
        fn ensure_utf8(path: &Path) -> anyhow::Result<&str> {
            path.to_str()
                .with_context(|| format!("must be UTF-8: {:?}", path.display()))
        }

        let input = json!({
            "manifest_dir": ensure_utf8(pkg_manifest_dir)?,
            "paths": paths
                .iter()
                .map(|(src_path, test_suite_path)| {
                    let src_path = ensure_utf8(src_path.as_ref())?;
                    let test_suite_path = ensure_utf8(test_suite_path.as_ref())?;
                    Ok(json!({
                        "src": src_path,
                        "test_suite": test_suite_path
                    }))
                })
                .collect::<anyhow::Result<Vec<_>>>()?
        })
        .to_string();

        let jq = crate::process::which("jq", cwd).with_context(|| {
            "`jq` not found. install `jq` from https://github.com/stedolan/jq/releases"
        })?;

        let output = crate::process::process(jq, &cwd)
            .args(&["-c", open.as_ref()])
            .pipe_input(Some(input))
            .read_with_shell_status(shell)?;

        let args = serde_json::from_str::<Vec<String>>(&output)
            .with_context(|| "expected string array")?;

        ensure!(!args.is_empty(), "empty command");

        crate::process::with_which(&args[0], cwd)?
            .args(&args[1..])
            .exec_with_shell_status(shell)?;
    }
    Ok(())
}
