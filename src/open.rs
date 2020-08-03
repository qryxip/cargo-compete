use crate::{project::Open, shell::Shell};
use std::{borrow::Borrow, path::Path};
use url::Url;

pub(crate) fn open(
    urls: &[impl Borrow<Url>],
    open: Option<Open>,
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
        let mut cmd = match open {
            Open::Vscode => crate::process::with_which("code", cwd)?,
            Open::Emacsclient => {
                let mut cmd = crate::process::with_which("emacsclient", cwd)?;
                cmd.arg("-n");
                cmd
            }
        };

        for (src_path, test_suite_path) in paths {
            cmd.arg(src_path.as_ref());
            cmd.arg(test_suite_path.as_ref());
        }

        if open == Open::Vscode {
            cmd.arg("-a");
            cmd.arg(pkg_manifest_dir);
        }

        cmd.exec_with_shell_status(shell)?;
    }
    Ok(())
}
