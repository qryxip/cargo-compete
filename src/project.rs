use crate::shell::Shell;
use anyhow::{bail, Context as _};
use cargo_metadata::{Metadata, MetadataCommand, Package, Resolve};
use easy_ext::ext;
use heck::KebabCase as _;
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    env,
    path::{Path, PathBuf},
    str,
};
use url::Url;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct WorkspaceMetadata {
    cargo_compete: WorkspaceMetadataCargoCompete,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WorkspaceMetadataCargoCompete {
    pub(crate) workspace_members: WorkspaceMembers,
    pub(crate) template: WorkspaceMetadataCargoCompeteTemplate,
    pub(crate) platform: WorkspaceMetadataCargoCompetePlatform,
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum WorkspaceMembers {
    IncludeAll,
    ExcludeAll,
    FocusOne,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WorkspaceMetadataCargoCompeteTemplate {
    pub(crate) code: PathBuf,
    //dependencies: Option<_>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub(crate) enum WorkspaceMetadataCargoCompetePlatform {
    Atcoder {
        #[serde(rename = "via-binary")]
        via_binary: Option<WorkspaceMetadataCargoCompetePlatformViaBinary>,
    },
    Codeforces,
    Yukicoder,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WorkspaceMetadataCargoCompetePlatformViaBinary {
    target: String,
    use_cross: bool,
    strip_exe: Option<PathBuf>,
    upx_exe: Option<PathBuf>,
}

#[ext(MetadataExt)]
impl Metadata {
    pub(crate) fn read_workspace_metadata(&self) -> anyhow::Result<WorkspaceMetadataCargoCompete> {
        let path = self.workspace_root.join("workspace-metadata.toml");
        let WorkspaceMetadata { cargo_compete } = crate::fs::read_toml(path)?;
        Ok(cargo_compete)
    }

    pub(crate) fn query_for_member<'a>(
        &'a self,
        spec: Option<&str>,
    ) -> anyhow::Result<&'a Package> {
        let cargo_exe = env::var_os("CARGO").with_context(|| "`$CARGO` should be present")?;

        let manifest_path = self
            .resolve
            .as_ref()
            .and_then(|Resolve { root, .. }| root.as_ref())
            .map(|id| self[id].manifest_path.clone())
            .unwrap_or_else(|| self.workspace_root.join("Cargo.toml"));

        let output = std::process::Command::new(cargo_exe)
            .arg("pkgid")
            .arg("--manifest-path")
            .arg(manifest_path)
            .args(spec)
            .output()?;
        let stdout = str::from_utf8(&output.stdout)?.trim_end();
        let stderr = str::from_utf8(&output.stderr)?.trim_end();
        if !output.status.success() {
            bail!("{}", stderr.trim_start_matches("error: "));
        }

        let url = stdout.parse::<Url>()?;
        let fragment = url.fragment().expect("the URL should contain fragment");
        let name = match *fragment.splitn(2, ':').collect::<Vec<_>>() {
            [name, _] => name,
            [_] => url
                .path_segments()
                .and_then(Iterator::last)
                .expect("should contain name"),
            _ => unreachable!(),
        };

        self.packages
            .iter()
            .filter(move |Package { id, .. }| self.workspace_members.contains(id))
            .find(|p| p.name == name)
            .with_context(|| {
                let spec = spec.expect("should be present here");
                format!("`{}` is not a member of the workspace", spec)
            })
    }

    pub(crate) fn add_member(
        self,
        package_name: &str,
        problem_indexes: &BTreeSet<String>,
        problems_are_yukicoder_no: bool,
        shell: &mut Shell,
    ) -> anyhow::Result<()> {
        let (workspace_metadata, workspace_metadata_edit) =
            self.read_workspace_metadata_preserving()?;

        let mut manifest = r#"[package]
name = ""
version = "0.1.0"
edition = "2018"
publish = false

[package.metadata.cargo-compete.problems]
"#
        .parse::<toml_edit::Document>()
        .unwrap();

        manifest["package"]["name"] = toml_edit::value(package_name);

        manifest["package"]["metadata"]["cargo-compete"]["problems"] = toml_edit::Item::Table({
            let mut tbl = toml_edit::Table::new();
            for problem_index in problem_indexes {
                let bin_name = format!("{}-{}", package_name, problem_index.to_kebab_case());

                match workspace_metadata.platform {
                    WorkspaceMetadataCargoCompetePlatform::Atcoder { .. } => {
                        tbl[&*bin_name]["platform"] = toml_edit::value("atcoder");
                        tbl[&*bin_name]["contest"] = toml_edit::value(package_name);
                        tbl[&*bin_name]["index"] = toml_edit::value(&**problem_index);
                    }
                    WorkspaceMetadataCargoCompetePlatform::Codeforces => {
                        tbl[&*bin_name]["platform"] = toml_edit::value("codeforces");
                        tbl[&*bin_name]["contest"] = toml_edit::value(package_name);
                        tbl[&*bin_name]["index"] = toml_edit::value(&**problem_index);
                    }
                    WorkspaceMetadataCargoCompetePlatform::Yukicoder => {
                        tbl[&*bin_name]["platform"] = toml_edit::value("yukicoder");
                        if problems_are_yukicoder_no {
                            tbl[&*bin_name]["no"] = toml_edit::value(&**problem_index);
                        } else {
                            tbl[&*bin_name]["contest"] = toml_edit::value(package_name);
                            tbl[&*bin_name]["index"] = toml_edit::value(&**problem_index);
                        }
                    }
                }
            }
            tbl
        });

        manifest["bin"] = toml_edit::Item::ArrayOfTables({
            let mut arr = toml_edit::ArrayOfTables::new();
            for problem_index in problem_indexes {
                let mut tbl = toml_edit::Table::new();
                tbl["name"] = toml_edit::value(format!(
                    "{}-{}",
                    package_name,
                    problem_index.to_kebab_case(),
                ));
                tbl["path"] =
                    toml_edit::value(format!("src/bin/{}.rs", problem_index.to_kebab_case()));
                arr.append(tbl);
            }
            arr
        });

        if workspace_metadata_edit["template"]["dependencies"].is_table() {
            manifest["dependencies"] = workspace_metadata_edit["template"]["dependencies"].clone();
        }

        let pkg_manifest_dir = self.workspace_root.join(package_name);

        if pkg_manifest_dir.exists() {
            bail!("`{}` already exists", pkg_manifest_dir.display());
        }
        crate::fs::create_dir_all(&pkg_manifest_dir)?;

        let pkg_manifest_path = pkg_manifest_dir.join("Cargo.toml");
        crate::fs::write(&pkg_manifest_path, manifest.to_string())?;

        let src_bin = pkg_manifest_dir.join("src").join("bin");
        crate::fs::create_dir_all(&src_bin)?;

        let template_code =
            crate::fs::read_to_string(self.workspace_root.join(workspace_metadata.template.code))?;

        for problem_index in problem_indexes {
            let src_path = src_bin
                .join(problem_index.to_kebab_case())
                .with_extension("rs");
            crate::fs::write(src_path, &template_code)?;
        }

        shell.status(
            "Created",
            format!(
                "`{}` package at {}",
                package_name,
                pkg_manifest_dir.display()
            ),
        )?;

        let all_pkg_manifest_dirs = std::fs::read_dir(&self.workspace_root)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("Cargo.toml").exists());

        match workspace_metadata.workspace_members {
            WorkspaceMembers::IncludeAll => {
                cargo_member::Include::new(&self.workspace_root, all_pkg_manifest_dirs)
                    .stderr(shell.err())
                    .exec()
            }
            WorkspaceMembers::ExcludeAll => {
                cargo_member::Exclude::new(&self.workspace_root, all_pkg_manifest_dirs)
                    .stderr(shell.err())
                    .exec()
            }
            WorkspaceMembers::FocusOne => {
                cargo_member::Focus::new(&self.workspace_root, &pkg_manifest_dir)
                    .stderr(shell.err())
                    .exec()
            }
        }
    }
}

#[ext]
impl Metadata {
    fn read_workspace_metadata_preserving(
        &self,
    ) -> anyhow::Result<(WorkspaceMetadataCargoCompete, toml_edit::Item)> {
        let path = self.workspace_root.join("workspace-metadata.toml");
        let (WorkspaceMetadata { cargo_compete }, edit) = crate::fs::read_toml_preserving(path)?;
        Ok((cargo_compete, edit["cargo-compete"].clone()))
    }
}

pub(crate) fn locate_project(cwd: &Path) -> anyhow::Result<PathBuf> {
    cwd.ancestors()
        .map(|p| p.join("Cargo.toml"))
        .find(|p| p.exists())
        .with_context(|| {
            format!(
                "could not find `Cargo.toml` in `{}` or any parent directory. first, run \
                 `cargo compete init` and `cd` to a workspace",
                cwd.display(),
            )
        })
}

pub(crate) fn cargo_metadata(manifest_path: impl AsRef<Path>) -> cargo_metadata::Result<Metadata> {
    MetadataCommand::new()
        .manifest_path(manifest_path.as_ref())
        .exec()
}
