use crate::{
    config::{CargoCompeteConfig, CargoCompeteConfigNew},
    oj_api,
    shell::{ColorChoice, Shell},
};
use anyhow::{bail, Context as _};
use camino::{Utf8Path, Utf8PathBuf};
use heck::KebabCase as _;
use itertools::Itertools as _;
use liquid::object;
use snowchains_core::web::{PlatformKind, ProblemsInContest, YukicoderRetrieveTestCasesTargets};
use std::collections::BTreeMap;
use structopt::StructOpt;
use strum::VariantNames as _;
use url::Url;

#[derive(StructOpt, Debug)]
pub struct OptCompeteNew {
    /// Retrieve system test cases
    #[structopt(long)]
    pub full: bool,

    /// Open URLs and files
    #[structopt(long)]
    pub open: bool,

    /// Retrieve only the problems
    #[structopt(long, value_name("INDEX"))]
    pub problems: Option<Vec<String>>,

    /// Path to `compete.toml`
    #[structopt(long, value_name("PATH"))]
    pub config: Option<Utf8PathBuf>,

    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,

    /// Contest ID. Required for some platforms
    pub contest: Option<String>,
}

pub fn run(opt: OptCompeteNew, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteNew {
        full,
        open,
        problems,
        config,
        color,
        contest,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let cargo_compete_config_path = crate::config::locate(cwd, config)?;
    let cargo_compete_dir = cargo_compete_config_path.with_file_name("");
    let cargo_compete_config = crate::config::load(&cargo_compete_config_path, shell)?;

    match &cargo_compete_config.new {
        CargoCompeteConfigNew::None => {
            bail!("`new` is `none`: {}", cargo_compete_config_path)
        }
        CargoCompeteConfigNew::CargoCompete {
            platform: PlatformKind::Atcoder,
            ..
        } => {
            let contest = contest.with_context(|| "`contest` is required for AtCoder")?;
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome = crate::web::retrieve_testcases::dl_from_atcoder(
                ProblemsInContest::Indexes { contest, problems },
                full,
                &cookies_path,
                shell,
            )?;

            let group = Group::Atcoder(crate::web::url::atcoder_contest(
                outcome
                    .get(0)
                    .and_then(|p| p.contest_url.as_ref())
                    .with_context(|| "empty result")?,
            )?);

            let problems = outcome.iter().map(|p| (&*p.index, &p.url)).collect();

            let urls = urls(&outcome);

            let (manifest_dir, src_paths) = create_new_package(
                &cargo_compete_config_path,
                &cargo_compete_config,
                &group,
                &problems,
                shell,
            )?;

            let file_paths = itertools::zip_eq(
                src_paths,
                crate::web::retrieve_testcases::save_test_cases(
                    &cargo_compete_dir,
                    &manifest_dir,
                    &cargo_compete_config.test_suite,
                    true,
                    outcome,
                    |_, index| vec![group.package_name() + "-" + &index.to_kebab_case()],
                    |_, index| vec![index.to_kebab_case()],
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &manifest_dir,
                    &cargo_compete_dir,
                    shell,
                )?;
            }
        }
        CargoCompeteConfigNew::CargoCompete {
            platform: PlatformKind::Codeforces,
            ..
        } => {
            let contest = contest.with_context(|| "`contest` is required for Codeforces")?;
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome = crate::web::retrieve_testcases::dl_from_codeforces(
                ProblemsInContest::Indexes { contest, problems },
                &cookies_path,
                shell,
            )?;

            let group = Group::Codeforces(crate::web::url::codeforces_contest(
                outcome
                    .get(0)
                    .and_then(|p| p.contest_url.as_ref())
                    .with_context(|| "empty result")?,
            )?);

            let problems = outcome.iter().map(|p| (&*p.index, &p.url)).collect();

            let urls = urls(&outcome);

            let (manifest_dir, src_paths) = create_new_package(
                &cargo_compete_config_path,
                &cargo_compete_config,
                &group,
                &problems,
                shell,
            )?;

            let file_paths = itertools::zip_eq(
                src_paths,
                crate::web::retrieve_testcases::save_test_cases(
                    &cargo_compete_dir,
                    &manifest_dir,
                    &cargo_compete_config.test_suite,
                    true,
                    outcome,
                    |_, index| vec![group.package_name() + "-" + &index.to_kebab_case()],
                    |_, index| vec![index.to_kebab_case()],
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &manifest_dir,
                    &cargo_compete_dir,
                    shell,
                )?;
            }
        }
        CargoCompeteConfigNew::CargoCompete {
            platform: PlatformKind::Yukicoder,
            ..
        } => {
            let contest = contest.as_deref();
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome = crate::web::retrieve_testcases::dl_from_yukicoder(
                if let Some(contest) = contest {
                    YukicoderRetrieveTestCasesTargets::Contest(contest.to_owned(), problems)
                } else if let Some(problems) = problems {
                    YukicoderRetrieveTestCasesTargets::ProblemNos(problems)
                } else {
                    bail!("either of `<contest>` or `--problems` required for yukicoder");
                },
                full,
                shell,
            )?;

            let contest = outcome
                .get(0)
                .and_then(|p| p.contest_url.as_ref())
                .map(crate::web::url::yukicoder_contest)
                .transpose()?;
            let group = match contest {
                None => Group::YukicoderProblems,
                Some(contest) => Group::YukicoderContest(contest),
            };

            let problems = outcome.iter().map(|p| (&*p.index, &p.url)).collect();

            let urls = urls(&outcome);

            let (manifest_dir, src_paths) = create_new_package(
                &cargo_compete_config_path,
                &cargo_compete_config,
                &group,
                &problems,
                shell,
            )?;

            let file_paths = itertools::zip_eq(
                src_paths,
                crate::web::retrieve_testcases::save_test_cases(
                    &cargo_compete_dir,
                    &manifest_dir,
                    &cargo_compete_config.test_suite,
                    true,
                    outcome,
                    |_, index| vec![group.package_name() + "-" + &index.to_kebab_case()],
                    |_, index| vec![index.to_kebab_case()],
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &manifest_dir,
                    &cargo_compete_dir,
                    shell,
                )?;
            }
        }
        CargoCompeteConfigNew::OjApi {
            url: contest_url, ..
        } => {
            if problems.is_some() {
                bail!("`--problems` option is not allowed for `oj-api`");
            }

            let contest_id = contest.with_context(|| "`contest` is required for oj-api")?;
            let contest_url = &contest_url
                .render(&object!({
                    "id": &contest_id,
                }))?
                .parse()?;

            let outcome = oj_api::get_contest(contest_url, &cargo_compete_dir, shell)?
                .into_iter()
                .map(|(problem_url, _alphabet)| {
                    let problem =
                        oj_api::get_problem(&problem_url, full, &cargo_compete_dir, shell)?;
                    let problem =
                        crate::web::retrieve_testcases::Problem::from_oj_api_with_alphabet(
                            problem, full,
                        )?;
                    Ok((problem_url, problem))
                })
                .collect::<anyhow::Result<BTreeMap<_, _>>>()?;

            let group = &Group::OjApi(contest_id);

            let (manifest_dir, src_paths) = create_new_package(
                &cargo_compete_config_path,
                &cargo_compete_config,
                group,
                &outcome.iter().map(|(u, p)| (&*p.index, u)).collect(),
                shell,
            )?;

            let (urls, problems) = {
                let (mut urls, mut problems) = (vec![], vec![]);
                for (url, problem) in outcome {
                    urls.push(url);
                    problems.push(problem);
                }
                (urls, problems)
            };

            let file_paths = itertools::zip_eq(
                src_paths,
                crate::web::retrieve_testcases::save_test_cases(
                    &cargo_compete_dir,
                    &manifest_dir,
                    &cargo_compete_config.test_suite,
                    true,
                    problems,
                    |_, index| vec![group.package_name() + "-" + &index.to_kebab_case()],
                    |_, index| vec![index.to_kebab_case()],
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &manifest_dir,
                    &cargo_compete_dir,
                    shell,
                )?;
            }
        }
    }
    Ok(())
}

fn urls(outcome: &[crate::web::retrieve_testcases::Problem<impl Sized>]) -> Vec<Url> {
    outcome.iter().map(|p| p.url.clone()).collect()
}

#[derive(Clone, Debug)]
enum Group {
    Atcoder(String),
    Codeforces(String),
    YukicoderProblems,
    YukicoderContest(String),
    OjApi(String),
}

impl Group {
    fn contest(&self) -> Option<&str> {
        match self {
            Self::Atcoder(contest)
            | Self::Codeforces(contest)
            | Self::YukicoderContest(contest)
            | Self::OjApi(contest) => Some(contest),
            Self::YukicoderProblems => None,
        }
    }

    fn package_name(&self) -> String {
        let mut package_name = self.contest().unwrap_or("problems").to_owned();
        if package_name.starts_with(|c| ('0'..='9').contains(&c)) {
            package_name = format!("contest{}", package_name);
        }
        package_name
    }
}

fn create_new_package(
    cargo_compete_config_path: &Utf8Path,
    cargo_compete_config: &CargoCompeteConfig,
    group: &Group,
    problems: &BTreeMap<&str, &Url>,
    shell: &mut Shell,
) -> anyhow::Result<(Utf8PathBuf, Vec<Utf8PathBuf>)> {
    let template = cargo_compete_config.template(cargo_compete_config_path, shell)?;
    let template_new = template.new.as_ref().with_context(|| {
        format!(
            "`template.new` is required for the command: {}",
            cargo_compete_config_path,
        )
    })?;

    let manifest_dir = cargo_compete_config
        .new
        .path()
        .with_context(|| format!("`new` is `none`: {}", cargo_compete_config_path))?
        .render(&object!({
            "contest": group.contest(),
            "package_name": group.package_name(),
        }))?;
    let manifest_dir = Utf8Path::new(&manifest_dir);
    let manifest_dir = cargo_compete_config_path
        .with_file_name(".")
        .join(manifest_dir.strip_prefix(".").unwrap_or(manifest_dir));

    let manifest_path = manifest_dir.join("Cargo.toml");

    if manifest_dir.exists() {
        bail!(
            "could not create a new package. `{}` already exists",
            manifest_dir,
        );
    }

    crate::process::process(crate::process::cargo_exe()?)
        .arg("new")
        .arg("-q")
        .arg("--vcs")
        .arg("none")
        .arg("--name")
        .arg(group.package_name())
        .arg(&manifest_dir)
        .cwd(cargo_compete_config_path.with_file_name(""))
        .exec()?;

    let mut package_metadata_cargo_compete_bin = problems
        .keys()
        .map(|problem_index| {
            format!(
                r#"{} = {{ alias = "", problem = "" }}
"#,
                escape_key(&format!(
                    "{}-{}",
                    group.package_name(),
                    problem_index.to_kebab_case(),
                )),
            )
        })
        .join("")
        .parse::<toml_edit::Document>()?;

    for (problem_index, problem_url) in problems {
        let bin_name = &format!("{}-{}", group.package_name(), problem_index.to_kebab_case());
        let bin_alias = problem_index.to_kebab_case();
        let problem_url = problem_url.as_str();
        package_metadata_cargo_compete_bin[bin_name]["alias"] = toml_edit::value(bin_alias);
        package_metadata_cargo_compete_bin[bin_name]["problem"] = toml_edit::value(problem_url);
    }

    let bin = toml_edit::Item::ArrayOfTables({
        let mut arr = toml_edit::ArrayOfTables::new();
        for problem_index in problems.keys() {
            let mut tbl = toml_edit::Table::new();
            tbl["name"] = toml_edit::value(format!(
                "{}-{}",
                group.package_name(),
                problem_index.to_kebab_case(),
            ));
            tbl["path"] = toml_edit::value(format!("src/bin/{}.rs", problem_index.to_kebab_case()));
            arr.append(tbl);
        }
        arr
    });

    static DEFAULT_MANIFEST_END: &str = r"
# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
";

    let mut manifest = crate::fs::read_to_string(&manifest_path)?;
    if manifest.ends_with(DEFAULT_MANIFEST_END) {
        manifest = manifest.replace(
            DEFAULT_MANIFEST_END,
            r"
[bin]

[dependencies]

[dev-dependencies]
",
        );
    }
    let mut manifest = manifest.parse::<toml_edit::Document>()?;

    if !template_new.profile.as_table().is_empty() {
        let mut profile = template_new.profile.clone();
        profile.as_table_mut().set_implicit(true);
        let mut head = toml_edit::Document::new();
        head["profile"] = profile.root;
        manifest = format!("{}\n{}", head, manifest).parse()?;
    }

    set_implicit_table_if_none(&mut manifest["package"]["metadata"]);
    set_implicit_table_if_none(&mut manifest["package"]["metadata"]["cargo-compete"]);
    set_implicit_table_if_none(&mut manifest["package"]["metadata"]["cargo-compete"]["bin"]);

    for (key, val) in package_metadata_cargo_compete_bin.as_table().iter() {
        manifest["package"]["metadata"]["cargo-compete"]["bin"][key] = val.clone();
    }

    manifest["bin"] = bin;
    manifest["dependencies"] = template_new.dependencies.root.clone();
    manifest["dev-dependencies"] = template_new.dev_dependencies.root.clone();

    if let Ok(new_manifest) = manifest
        .to_string()
        .replace("\"}", "\" }")
        .parse::<toml_edit::Document>()
    {
        manifest = new_manifest;
    }

    crate::fs::write(&manifest_path, manifest.to_string_in_original_order())?;

    let src_bin_dir = manifest_dir.join("src").join("bin");

    crate::fs::create_dir_all(&src_bin_dir)?;

    let src_paths = problems
        .keys()
        .map(|problem_index| {
            src_bin_dir
                .join(problem_index.to_kebab_case())
                .with_extension("rs")
        })
        .collect::<Vec<_>>();

    for src_path in &src_paths {
        crate::fs::write(src_path, &template.src)?;
    }
    crate::fs::remove_file(manifest_dir.join("src").join("main.rs"))?;

    for (from, to) in &template_new.copy_files {
        let from = cargo_compete_config_path.with_file_name("").join(from);
        let to = manifest_dir.join(to);
        crate::fs::copy(from, to)?;
    }

    shell.status(
        "Created",
        format!("`{}` package at {}", group.package_name(), manifest_dir),
    )?;

    Ok((manifest_dir, src_paths))
}

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

fn set_implicit_table_if_none(item: &mut toml_edit::Item) {
    if item.is_none() {
        *item = {
            let mut tbl = toml_edit::Table::new();
            tbl.set_implicit(true);
            toml_edit::Item::Table(tbl)
        };
    }
}
