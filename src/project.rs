use crate::shell::Shell;
use anyhow::{bail, Context as _};
use cargo_metadata::{Metadata, MetadataCommand, Package, Resolve, Target};
use derivative::Derivative;
use easy_ext::ext;
use heck::KebabCase as _;
use indexmap::IndexMap;
use itertools::Itertools as _;
use serde::{de::Error as _, Deserialize, Deserializer};
use std::{
    collections::BTreeMap,
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

#[derive(Deserialize, Derivative)]
#[derivative(Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WorkspaceMetadataCargoCompete {
    pub(crate) new_workspace_member: NewWorkspaceMember,
    #[derivative(Debug = "ignore")]
    #[serde(deserialize_with = "deserialize_liquid_template_with_custom_filter")]
    pub(crate) test_suite: liquid::Template,
    pub(crate) open: Option<String>,
    pub(crate) template: WorkspaceMetadataCargoCompeteTemplate,
    pub(crate) platform: WorkspaceMetadataCargoCompetePlatform,
}

fn deserialize_liquid_template_with_custom_filter<'de, D>(
    deserializer: D,
) -> Result<liquid::Template, D::Error>
where
    D: Deserializer<'de>,
{
    liquid_template_with_custom_filter(&String::deserialize(deserializer)?)
        .map_err(D::Error::custom)
}

fn liquid_template_with_custom_filter(text: &str) -> Result<liquid::Template, String> {
    use liquid::ParserBuilder;
    use liquid_core::{Filter, Runtime, Value, ValueView};
    use liquid_derive::{Display_filter, FilterReflection, ParseFilter};

    return ParserBuilder::with_stdlib()
        .filter(Kebabcase)
        .build()
        .map_err(|e| e.to_string())?
        .parse(text)
        .map_err(|e| e.to_string());

    #[derive(Clone, ParseFilter, FilterReflection)]
    #[filter(
        name = "kebabcase",
        description = "Returns the absolute value of a number.",
        parsed(KebabcaseFilter) // A struct that implements `Filter` (must implement `Default`)
   )]
    struct Kebabcase;

    #[derive(Default, Debug, Display_filter)]
    #[name = "kebabcase"]
    struct KebabcaseFilter;

    impl Filter for KebabcaseFilter {
        fn evaluate(&self, input: &dyn ValueView, _: &Runtime) -> liquid_core::Result<Value> {
            Ok(Value::scalar(input.to_kstr().to_kebab_case()))
        }
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NewWorkspaceMember {
    Include,
    Focus,
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
    pub(crate) target: String,
    pub(crate) cross: Option<PathBuf>,
    pub(crate) strip: Option<PathBuf>,
    pub(crate) upx: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct PackageMetadataCargoCompete {
    pub(crate) bin: IndexMap<String, PackageMetadataCargoCompeteBin>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct PackageMetadataCargoCompeteBin {
    pub(crate) name: String,
    pub(crate) problem: TargetProblem,
}

#[derive(Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "kebab-case", tag = "platform")]
pub(crate) enum TargetProblem {
    Atcoder {
        contest: String,
        index: String,
        url: Option<Url>,
    },
    Codeforces {
        contest: String,
        index: String,
        url: Option<Url>,
    },
    Yukicoder(TargetProblemYukicoder),
}

impl TargetProblem {
    pub(crate) fn url(&self) -> Option<&Url> {
        match self {
            Self::Atcoder { url, .. }
            | Self::Codeforces { url, .. }
            | Self::Yukicoder(TargetProblemYukicoder::Problem { url, .. })
            | Self::Yukicoder(TargetProblemYukicoder::Contest { url, .. }) => url.as_ref(),
        }
    }
}

#[derive(Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub(crate) enum TargetProblemYukicoder {
    Problem {
        no: u64,
        url: Option<Url>,
    },
    Contest {
        contest: String,
        index: String,
        url: Option<Url>,
    },
}

#[ext(MetadataExt)]
impl Metadata {
    pub(crate) fn read_workspace_metadata(&self) -> anyhow::Result<WorkspaceMetadataCargoCompete> {
        let path = self.workspace_root.join("workspace-metadata.toml");
        let WorkspaceMetadata { cargo_compete } = crate::fs::read_toml(path)?;
        Ok(cargo_compete)
    }

    pub(crate) fn query_for_member<'a, S: AsRef<str>>(
        &'a self,
        spec: Option<S>,
    ) -> anyhow::Result<&'a Package> {
        let spec = spec.as_ref().map(AsRef::as_ref);

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
        problems: &BTreeMap<&str, &Url>,
        problems_are_yukicoder_no: bool,
        shell: &mut Shell,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let (workspace_metadata, workspace_metadata_edit) =
            self.read_workspace_metadata_preserving()?;

        let mut manifest = format!(
            r#"[package]
name = ""
version = "0.1.0"
edition = "2018"
publish = false

[package.metadata.cargo-compete.bin]
{}"#,
            problems
                .keys()
                .map(|problem_index| format!(
                    r#"{} = {{ name = "", problem = {{ {} }} }}
"#,
                    escape_key(&problem_index.to_kebab_case()),
                    match (&workspace_metadata.platform, problems_are_yukicoder_no) {
                        (WorkspaceMetadataCargoCompetePlatform::Atcoder { .. }, _)
                        | (WorkspaceMetadataCargoCompetePlatform::Codeforces, _) => {
                            r#"platform = "", contest = "", index = "", url = """#
                        }
                        (WorkspaceMetadataCargoCompetePlatform::Yukicoder, true) => {
                            r#"platform = "", kind = "no", no = "", url = """#
                        }
                        (WorkspaceMetadataCargoCompetePlatform::Yukicoder, false) => {
                            r#"platform = "", kind = "contest", contest = "", index = "", url = """#
                        }
                    }
                ))
                .join("")
        )
        .parse::<toml_edit::Document>()?;

        manifest["package"]["name"] = toml_edit::value(package_name);

        let tbl = &mut manifest["package"]["metadata"]["cargo-compete"]["bin"];
        for (problem_index, problem_url) in problems {
            tbl[problem_index.to_kebab_case()]["name"] = toml_edit::value(format!(
                "{}-{}",
                package_name,
                problem_index.to_kebab_case(),
            ));

            let tbl = &mut tbl[problem_index.to_kebab_case()]["problem"];

            match workspace_metadata.platform {
                WorkspaceMetadataCargoCompetePlatform::Atcoder { .. } => {
                    tbl["platform"] = toml_edit::value("atcoder");
                    tbl["contest"] = toml_edit::value(package_name);
                    tbl["index"] = toml_edit::value(&**problem_index);
                    tbl["url"] = toml_edit::value(problem_url.as_str());
                }
                WorkspaceMetadataCargoCompetePlatform::Codeforces => {
                    tbl["platform"] = toml_edit::value("codeforces");
                    tbl["contest"] = toml_edit::value(package_name);
                    tbl["index"] = toml_edit::value(&**problem_index);
                    tbl["url"] = toml_edit::value(problem_url.as_str());
                }
                WorkspaceMetadataCargoCompetePlatform::Yukicoder => {
                    tbl["platform"] = toml_edit::value("yukicoder");
                    if problems_are_yukicoder_no {
                        tbl["no"] = toml_edit::value(&**problem_index);
                    } else {
                        tbl["contest"] = toml_edit::value(package_name);
                        tbl["index"] = toml_edit::value(&**problem_index);
                    }
                    tbl["url"] = toml_edit::value(problem_url.as_str());
                }
            }
        }

        if let Ok(new_manifest) = manifest
            .to_string()
            .replace("\"} }", "\" } }")
            .parse::<toml_edit::Document>()
        {
            manifest = new_manifest;
        }

        manifest["bin"] = toml_edit::Item::ArrayOfTables({
            let mut arr = toml_edit::ArrayOfTables::new();
            for problem_index in problems.keys() {
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

        let src_paths = problems
            .keys()
            .map(|problem_index| {
                src_bin
                    .join(problem_index.to_kebab_case())
                    .with_extension("rs")
            })
            .collect::<Vec<_>>();

        for src_path in &src_paths {
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

        match workspace_metadata.new_workspace_member {
            NewWorkspaceMember::Include => {
                cargo_member::Include::new(&self.workspace_root, &[pkg_manifest_dir])
                    .stderr(shell.err())
                    .exec()
            }
            NewWorkspaceMember::Focus => {
                cargo_member::Focus::new(&self.workspace_root, &pkg_manifest_dir)
                    .stderr(shell.err())
                    .exec()
            }
        }?;

        return Ok(src_paths);

        fn escape_key(s: &str) -> String {
            if s.chars().any(|c| c.is_whitespace() || c.is_control()) {
                return toml::Value::String(s.to_owned()).to_string();
            }

            let mut doc = toml_edit::Document::new();
            doc[s] = toml_edit::value(0);
            doc.to_string()
                .trim_end()
                .trim_end_matches('0')
                .trim_end()
                .trim_end_matches('=')
                .trim_end()
                .to_owned()
        }
    }
}

#[ext]
impl Metadata {
    pub(crate) fn read_workspace_metadata_preserving(
        &self,
    ) -> anyhow::Result<(WorkspaceMetadataCargoCompete, toml_edit::Item)> {
        let path = self.workspace_root.join("workspace-metadata.toml");
        let (WorkspaceMetadata { cargo_compete }, edit) = crate::fs::read_toml_preserving(path)?;
        Ok((cargo_compete, edit["cargo-compete"].clone()))
    }
}

#[ext(PackageExt)]
impl Package {
    pub(crate) fn read_package_metadata(&self) -> anyhow::Result<PackageMetadataCargoCompete> {
        let CargoToml {
            package:
                CargoTomlPackage {
                    metadata: CargoTomlPackageMetadata { cargo_compete },
                },
        } = crate::fs::read_toml(&self.manifest_path)?;
        return Ok(cargo_compete);

        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct CargoToml {
            package: CargoTomlPackage,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct CargoTomlPackage {
            metadata: CargoTomlPackageMetadata,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct CargoTomlPackageMetadata {
            cargo_compete: PackageMetadataCargoCompete,
        }
    }

    pub(crate) fn bin_target<'a>(&'a self, name: &str) -> anyhow::Result<&'a Target> {
        self.targets
            .iter()
            .find(|t| t.name == name && t.kind == ["bin".to_owned()])
            .with_context(|| format!("no bin target named `{}` in `{}`", name, self.name))
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

#[cfg(test)]
mod tests {
    use liquid::object;
    use pretty_assertions::assert_eq;

    #[test]
    fn liquid_template_with_custom_filter() -> anyhow::Result<()> {
        let output = super::liquid_template_with_custom_filter("{{ s | kebabcase }}")
            .map_err(anyhow::Error::msg)?
            .render(&object!({ "s": "FooBarBaz" }))?;
        assert_eq!("foo-bar-baz", output);
        Ok(())
    }
}
