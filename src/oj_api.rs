use crate::shell::Shell;
use anyhow::{bail, Context as _};
use camino::Utf8Path;
use itertools::Itertools as _;
use serde::{
    de::{DeserializeOwned, Error as _},
    Deserialize, Deserializer,
};
use std::{env, ffi::OsStr, path::Path};
use url::Url;

pub(crate) fn get_problem(
    url: &Url,
    system: bool,
    cwd: &Utf8Path,
    shell: &mut Shell,
) -> anyhow::Result<Problem> {
    let args = &mut vec!["get-problem", url.as_ref()];
    if system {
        args.push("--system".as_ref());
    }
    call(args, cwd, shell)
}

pub(crate) fn get_contest(
    url: &Url,
    cwd: &Utf8Path,
    shell: &mut Shell,
) -> anyhow::Result<Vec<(Url, Option<String>)>> {
    let Contest { problems } = call(&["get-contest", url.as_ref()], cwd, shell)?;
    return Ok(problems
        .into_iter()
        .map(|ContestProblem { url, context }| (url, context.alphabet))
        .collect());

    #[derive(Deserialize)]
    struct Contest {
        problems: Vec<ContestProblem>,
    }

    #[derive(Deserialize)]
    struct ContestProblem {
        url: Url,
        context: ContestProblemContext,
    }

    #[derive(Deserialize)]
    struct ContestProblemContext {
        alphabet: Option<String>,
    }
}

pub(crate) fn guess_language_id(
    url: &Url,
    file: &Path,
    cwd: &Utf8Path,
    shell: &mut Shell,
) -> anyhow::Result<String> {
    return call(
        &[
            OsStr::new("guess-language-id"),
            url.as_str().as_ref(),
            "--file".as_ref(),
            file.as_ref(),
        ],
        cwd,
        shell,
    )
    .map(|GuessLanguageId { id }| id);

    #[derive(Deserialize)]
    struct GuessLanguageId {
        id: String,
    }
}

pub(crate) fn submit_code(
    url: &Url,
    file: &Path,
    language: &str,
    cwd: &Utf8Path,
    shell: &mut Shell,
) -> anyhow::Result<Url> {
    return call(
        &[
            "submit-code".as_ref(),
            url.as_str().as_ref(),
            "--file".as_ref(),
            file.as_os_str(),
            "--language".as_ref(),
            language.as_ref(),
        ],
        cwd,
        shell,
    )
    .map(|SubmitCode { url }| url);

    #[derive(Deserialize)]
    struct SubmitCode {
        url: Url,
    }
}

fn call<T: DeserializeOwned, S: AsRef<OsStr>>(
    args: &[S],
    cwd: &Utf8Path,
    shell: &mut Shell,
) -> anyhow::Result<T> {
    let oj_api_exe = which::which_in("oj-api", env::var_os("PATH"), cwd)
        .with_context(|| "`oj-api` not found")?;

    let output = crate::process::process(oj_api_exe)
        .args(args)
        .cwd(cwd)
        .display_cwd()
        .read_with_shell_status(shell)?;

    let Outcome { result, messages } = serde_json::from_str(&output)
        .with_context(|| "could not parse the output from `oj-api`")?;

    return if let Ok(result) = result {
        for message in messages {
            shell.warn(format!("oj-api: {}", message))?;
        }
        Ok(result)
    } else {
        bail!(
            "`oj-api` returned error:\n{}",
            messages.iter().map(|s| format!("- {}\n", s)).join(""),
        );
    };

    struct Outcome<T> {
        result: Result<T, ()>,
        messages: Vec<String>,
    }

    impl<'de, T: DeserializeOwned> Deserialize<'de> for Outcome<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let Repr {
                status,
                messages,
                result,
            } = Repr::deserialize(deserializer)?;

            return match &*status {
                "ok" => Ok(Self {
                    result: Ok(result),
                    messages,
                }),
                "error" => Ok(Self {
                    result: Err(()),
                    messages,
                }),
                status => Err(D::Error::custom(format!(
                    "expected \"ok\" or \"error\", got {:?}",
                    status,
                ))),
            };

            #[derive(Deserialize)]
            struct Repr<T> {
                status: String,
                messages: Vec<String>,
                result: T,
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Problem {
    /// > ```text
    /// > "url": {
    /// >   "type": "string",
    /// >   "format": "uri"
    /// > },
    /// > ```
    pub(crate) url: Url,

    /// > ```text
    /// > "name": {
    /// >   "type": "string",
    /// >   "description": "the title of the problem without alphabets, i.e. \"Xor Sum\" is used instead of \"D - Xor Sum\"; because in many contest sites, the alphabets are attributes belonging to the relation between problems and contests, rather than only the problem",
    /// >   "examples": [
    /// >     "Xor Sum",
    /// >     "K-th Beautiful String"
    /// >   ]
    /// > },
    /// > ```
    pub(crate) name: Option<String>,

    /// > ```text
    /// > "context": {
    /// >   "type": "object",
    /// >   "properties": {
    /// >     "contest": {
    /// >       "type": "object",
    /// >       "properties": {
    /// >         "url": {
    /// >           "type": "string",
    /// >           "format": "uri"
    /// >         },
    /// >         "name": {
    /// >           "type": "string"
    /// >         }
    /// >       }
    /// >     },
    /// >     "alphabet": {
    /// >       "type": "string"
    /// >     }
    /// >   }
    /// > },
    /// > ```
    pub(crate) context: ProblemContext,

    /// > ```text
    /// > "timeLimit": {
    /// >   "type": "integer",
    /// >   "description": "in milliseconds (msec)"
    /// > },
    /// > ```
    pub(crate) time_limit: Option<u64>,

    /// > ```text
    /// > "tests": {
    /// >   "type": "array",
    /// >   "items": {
    /// >     "type": "object",
    /// >     "properties": {
    /// >       "name": {
    /// >         "type": "string"
    /// >       },
    /// >       "input": {
    /// >         "type": "string"
    /// >       },
    /// >       "output": {
    /// >         "type": "string"
    /// >       }
    /// >     },
    /// >     "required": [
    /// >       "input",
    /// >       "output"
    /// >     ]
    /// >   },
    /// >   "examples": [
    /// >     [
    /// >       {
    /// >         "input": "35\n",
    /// >         "output": "57\n"
    /// >       },
    /// >       {
    /// >         "input": "57\n",
    /// >         "output": "319\n"
    /// >       }
    /// >     ]
    /// >   ]
    /// > },
    /// > ```
    pub(crate) tests: Vec<ProblemTest>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProblemContext {
    pub(crate) contest: Option<ProblemContextContest>,
    pub(crate) alphabet: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProblemContextContest {
    pub(crate) url: Option<Url>,
    pub(crate) name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProblemTest {
    pub(crate) name: Option<String>,
    pub(crate) input: String,
    pub(crate) output: String,
}
