# cargo-compete

[![CI](https://github.com/qryxip/cargo-compete/workflows/CI/badge.svg)](https://github.com/qryxip/cargo-compete/actions?workflow=CI)
[![codecov](https://codecov.io/gh/qryxip/cargo-compete/branch/master/graph/badge.svg)](https://codecov.io/gh/qryxip/cargo-compete/branch/master)
[![dependency status](https://deps.rs/repo/github/qryxip/cargo-compete/status.svg)](https://deps.rs/repo/github/qryxip/cargo-compete)
[![Crates.io](https://img.shields.io/crates/v/cargo-compete.svg)](https://crates.io/crates/cargo-compete)
[![Crates.io](https://img.shields.io/crates/l/cargo-compete.svg)](https://crates.io/crates/cargo-compete)
[![Join the chat at https://gitter.im/cargo-compete/community](https://badges.gitter.im/cargo-compete/community.svg)](https://gitter.im/cargo-compete/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

[日本語](https://github.com/qryxip/cargo-compete/blob/master/README-ja.md)

A Cargo subcommand for competitive programming.

Supports AtCoder, Codeforces, and yukicoder.
Other websites are available via [online-judge-tools/api-client](https://github.com/online-judge-tools/api-client).

## Features

- Log in a website,
- (Automatically) register in a contest,
- Retrieves sample/system test cases, and save them as YAML files,
- Test your code for the YAML files,
- Submit your code,
- Watch your submissions. (available for only AtCoder)

|                | Registeration          | Sample Test Cases             | System Test Cases             | Submiting                     | Watching Submissions    | Submission Details |
| :------------: | :--------------------: | :---------------------------: | :---------------------------: | :---------------------------: | :---------------------: | :----------------: |
| AtCoder        | :heavy_check_mark:     | :heavy_check_mark:            | :heavy_check_mark:            | :heavy_check_mark:            | :grey_question:         | :x:                |
| Codeforces     | :x:                    | :heavy_check_mark:            | N/A                           | :heavy_check_mark:            | :x:                     | :x:                |
| yukicoder      | N/A                    | :heavy_check_mark:            | :heavy_check_mark:            | :heavy_check_mark:            | :x:                     | :x:                |
| Other websites | :x:                    | Depends on online-judge-tools | Depends on online-judge-tools | Depends on online-judge-tools | :x:                     | :x:                |

## Installation

### From Crates.io

```console
$ cargo install cargo-compete
```

If the build fails, adding `--locked` may help.

### From `master` branch

```console
$ cargo install --git https://github.com/qryxip/cargo-compete
```

### From GitHub Releases

We [provide the binaries in GitHub Releases](https://github.com/qryxip/cargo-compete/releases).

## Usages

### `cargo compete init`

Generates some files for other commands.

Run this command first.
It generates the following files.

- [`compete.toml`](#configuration)

    Required for other commands.

- [`.cargo/config.toml`](https://doc.rust-lang.org/cargo/reference/config.html)

    Sets `build/target-dir` to share the [`target` directory](https://doc.rust-lang.org/cargo/guide/build-cache.html).

- `template-cargo-lock.toml`

    A template of `Cargo.lock` for [`cargo compete new`](#cargo-compete-new).
    Generated only if you answer `2 Yes` to `Do you use crates on AtCoder?` question.
    If this file is generated, file path to it is added to `new.template.lockfile` in `compete.toml`.

![Screenshot](https://user-images.githubusercontent.com/14125495/91646306-b7e65980-ea88-11ea-8f0c-f11080b914ed.png)

### `cargo compete migrate cargo-atcoder`

See [the section in the Japanese readme](https://github.com/qryxip/cargo-compete/blob/master/README-ja.md#cargo-compete-migrate-cargo-atcoder).

### `cargo compete login`

Logges in a website.

**This is not a command for a package.**

You don't have to run this command beforehand, because cargo-compete asks credentials if necessary.

### `cargo compete participate`

Registeres in a contest.

**This is not a command for a package.**

You don't have to run this command beforehand, because cargo-compete registers in the contest if necessary.

### `cargo compete new`

Retrieves test cases and creates a package for the contest.

**Requires [`compete.toml`](#configuration).**
Generate it with [`cargo compete init`](#cargo-compete-init) first.

You can opens the pages in your browser with the `--open` option.
And you can also open the source files and the test cases in your browser by testing `open` in `compete.toml`.
If you forget to add `--open`, `cd` to the generated package and run [`cargo compete open`](#cargo-compete-open).

![Record](https://user-images.githubusercontent.com/14125495/91647287-1b29b900-ea94-11ea-9053-43e25c77706f.gif)

### `cargo compete add`

Generates [`bin` targets]((https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries)) and retrieves the test cases for them.

**Requires [`compete.toml`](#configuration).**
Generate it with [`cargo compete init`](#cargo-compete-init) first.

To use this function, configure `add` in the [`compete.toml`](#configuration) like this.

```toml
# for yukicoder
[add]
url = '{% case args[0] %}{% when "contest" %}https://yukicoder.me/contests/{{ args[1] }}{% when "problem" %}https://yukicoder.me/problems/no/{{ args[1] }}{% endcase %}'
is-contest = ["bash", "-c", '[[ $(cut -d / -f 4) == "contests" ]]'] # optional
#target-kind = "bin" # ["bin", "example"]. default to "bin"
bin-name = '{% assign segments = url | split: "/" %}{{ segments[5] }}'
#bin-alias = '{% assign segments = url | split: "/" %}{{ segments[5] }}' # optional
#bin-src-path = './src/bin/{{ bin_alias }}.rs' # optional
```

```console
❯ cargo compete a contest 296
    Added `1358` (bin) for https://yukicoder.me/problems/no/1358
    Added `1359` (bin) for https://yukicoder.me/problems/no/1359
    Added `1360` (bin) for https://yukicoder.me/problems/no/1360
    Added `1361` (bin) for https://yukicoder.me/problems/no/1361
    Added `1362` (bin) for https://yukicoder.me/problems/no/1362
    Added `1363` (bin) for https://yukicoder.me/problems/no/1363
    Added `1364` (bin) for https://yukicoder.me/problems/no/1364
    Added `1365` (bin) for https://yukicoder.me/problems/no/1365
    Saved 1 test case to /home/ryo/src/competitive/yukicoder/testcases/1358.yml
    Saved 3 test cases to /home/ryo/src/competitive/yukicoder/testcases/1359.yml
    Saved 3 test cases to /home/ryo/src/competitive/yukicoder/testcases/1360.yml
    Saved 3 test cases to /home/ryo/src/competitive/yukicoder/testcases/1361.yml
    Saved 3 test cases to /home/ryo/src/competitive/yukicoder/testcases/1362.yml
    Saved 1 test case to /home/ryo/src/competitive/yukicoder/testcases/1363.yml
    Saved 3 test cases to /home/ryo/src/competitive/yukicoder/testcases/1364.yml
    Saved 3 test cases to /home/ryo/src/competitive/yukicoder/testcases/1365.yml
❯ cargo compete a problem 9001
    Added `9001` (bin) for https://yukicoder.me/problems/no/9001
    Saved 1 test case to /home/ryo/src/competitive/yukicoder/testcases/9001.yml
```

### `cargo compete retrieve testcases` / `cargo compete download`

Retrieves test cases for an existing package.

**This is a command for a package.**
`cd` to the package generated with [`cargo compete new`](#cargo-compete-new).

![Screenshot](https://user-images.githubusercontent.com/14125495/113158161-82039080-9276-11eb-89df-58613b276ba4.png)

With `--open` option, you can download system test cases instead of sample ones.

For AtCoder, we have to use [Dropbox API](https://www.dropbox.com/developers/documentation/http/overview).
Generate an access token with these two permissions in some way,

- `files.metadata.read`
- `sharing.read`

and save a JSON file in the following format to <code>[{local data directory}](https://docs.rs/dirs-next/2.0.0/dirs_next/fn.data_local_dir.html)/cargo-compete/tokens/dropbox.json</code>.
(I'm thinking of better way)

```json
{
  "access_token": "<access token>"
}
```

[![asciicast](https://asciinema.org/a/409353.svg)](https://asciinema.org/a/409353?autoplay=1)

### `cargo compete retrieve submission-summaries`

Retrieves your submissions, and outputs as JSON.

**This is a command for a package.**
`cd` to the package generated with [`cargo compete new`](#cargo-compete-new).

[![asciicast](https://asciinema.org/a/403724.svg)](https://asciinema.org/a/403724?autoplay=1)

For example, you can get "the URL for the latest submission" by adding `| jq -r '.summaries[0].detail`.

```console
$ # for Linux
$ xdg-open "$(cargo compete r ss | jq -r '.summaries[0].detail')"
```

### `cargo compete open`

Opens pages in your browser, and opens source and test cases in your editor.

**This is a command for a package.**
`cd` to the package generated with [`cargo compete new`](#cargo-compete-new).

### `cargo compete test`

Runs tests.

**This is a command for a package.**
`cd` to the package generated with [`cargo compete new`](#cargo-compete-new).

You don't have to run this command beforehand, because the tests are run in [the `submit` command](#cargo-compete-submit).

### `cargo compete submit`

Submits your code.

**This is a command for a package.**
`cd` to the package generated with [`cargo compete new`](#cargo-compete-new).

[![asciicast](https://asciinema.org/a/403449.svg)](https://asciinema.org/a/403449?autoplay=1)

You can convert code with a tool such as [cargo-equip](https://github.com/qryxip/cargo-equip) and [cargo-executable-payload](https://github.com/qryxip/cargo-executable-payload) by setting `submit` in the [`compete.toml`](#configuration).

```toml
[submit]
kind = "command"
args = ["cargo", "+1.70.0", "equip", "--exclude-atcoder-202301-crates", "--remove", "docs", "--minify", "libs", "--bin", "{{ bin_name }}"]
language_id = "5054"
```

```toml
[submit]
kind = "command"
args = ["cargo", "executable-payload", "--bin", "{{ bin_name }}"]
language_id = "5054"
```

## Configuration

Here is an example for `compete.toml`.

```toml
# Path to the test file (Liquid template)
#
# Variables:
#
# - `manifest_dir`: Package directory
# - `contest`:      Contest ID (e.g. "abc100")
# - `bin_name`:     Name of a `bin` target (e.g. "abc100-a")
# - `bin_alias`:    "Alias" for a `bin` target defined in `pacakge.metadata.cargo-compete` (e.g. "a")
# - `problem`:      Alias for `bin_alias` (deprecated)
#
# Additional filters:
#
# - `kebabcase`: Convert to kebab case (by using the `heck` crate)
test-suite = "{{ manifest_dir }}/testcases/{{ bin_alias }}.yml"

# Open files with the command (`jq` command that outputs `string[] | string[][]`)
#
# VSCode:
#open = '[["code", "-a", .manifest_dir], ["code"] + (.paths | map([.src, .test_suite]) | flatten)]'
# Emacs:
#open = '["emacsclient", "-n"] + (.paths | map([.src, .test_suite]) | flatten)'

[template]
src = '''
fn main() {
    todo!();
}
'''

[template.new]
# `edition` for `Cargo.toml`.
edition = "2018"
# `profile` for `Cargo.toml`.
#
# By setting this, you can run tests with `opt-level=3` while enabling `debug-assertions` and `overflow-checks`.
#profile = '''
#[dev]
#opt-level = 3
#'''
dependencies = '''
num = "=0.2.1"
num-bigint = "=0.2.6"
num-complex = "=0.2.4"
num-integer = "=0.1.42"
num-iter = "=0.1.40"
num-rational = "=0.2.4"
num-traits = "=0.2.11"
num-derive = "=0.3.0"
ndarray = "=0.13.0"
nalgebra = "=0.20.0"
alga = "=0.9.3"
libm = "=0.2.1"
rand = { version = "=0.7.3", features = ["small_rng"] }
getrandom = "=0.1.14"
rand_chacha = "=0.2.2"
rand_core = "=0.5.1"
rand_hc = "=0.2.0"
rand_pcg = "=0.2.1"
rand_distr = "=0.2.2"
petgraph = "=0.5.0"
indexmap = "=1.3.2"
regex = "=1.3.6"
lazy_static = "=1.4.0"
ordered-float = "=1.0.2"
ascii = "=1.0.0"
permutohedron = "=0.2.4"
superslice = "=1.0.0"
itertools = "=0.9.0"
itertools-num = "=0.1.3"
maplit = "=1.0.2"
either = "=1.5.3"
im-rc = "=14.3.0"
fixedbitset = "=0.2.0"
bitset-fixed = "=0.1.0"
proconio = { version = "=0.3.6", features = ["derive"] }
text_io = "=0.1.8"
whiteread = "=0.5.0"
rustc-hash = "=1.1.0"
smallvec = "=1.2.0"
'''
dev-dependencies = '''
#atcoder-202004-lock = { git = "https://github.com/qryxip/atcoder-202004-lock" }
'''

[template.new.copy-files]
"./template-cargo-lock.toml" = "Cargo.lock"

[new]
kind = "cargo-compete"
# Platform
#
# - atcoder
# - codeforces
# - yukicoder
platform = "atcoder"
# Path (Liquid template)
#
# Variables:
#
# - `contest`:      Contest ID. **May be nil**
# - `package_name`: Package name
path = "./{{ contest }}"

#[new]
#kind = "oj-api"
#url = "https://atcoder.jp/contests/{{ id }}"
#path = "./{{ contest }}"

# for Library-Checker
#[add]
#url = "https://judge.yosupo.jp/problem/{{ args[0] }}"
##is-contest = ["false"] # optional
##target-kind = "bin" # ["bin", "example"]. default to "bin"
#bin-name = '{{ args[0] }}'
##bin-alias = '{{ args[0] }}' # optional
##bin-src-path = './src/bin/{{ bin_alias }}.rs' # optional

# for yukicoder
#[add]
#url = '{% case args[0] %}{% when "contest" %}https://yukicoder.me/contests/{{ args[1] }}{% when "problem" %}https://yukicoder.me/problems/no/{{ args[1] }}{% endcase %}'
#is-contest = ["bash", "-c", '[[ $(cut -d / -f 4) == "contests" ]]'] # optional
##target-kind = "bin" # ["bin", "example"]. default to "bin"
#bin-name = '{% assign segments = url | split: "/" %}{{ segments[5] }}'
##bin-alias = '{% assign segments = url | split: "/" %}{{ segments[5] }}' # optional
##bin-src-path = './src/bin/{{ bin_alias }}.rs' # optional

[test]
# Toolchain for the test. (optional)
toolchain = "1.42.0"
# Profile for `cargo build`. ("dev" | "release")
#
# Defaults to `"dev"`.
#profile = "dev"

[submit]
kind = "file"
path = "{{ src_path }}"
language_id = "5054"
#[submit]
#kind = "command"
#args = ["cargo", "+1.70.0", "equip", "--exclude-atcoder-202301-crates", "--remove", "docs", "--minify", "libs", "--bin", "{{ bin_name }}"]
#language_id = "5054"
```

And here is an example for `package.metadata` in `Cargo.toml`.

```toml
[package]
name = "practice"
version = "0.1.0"
authors = ["Ryo Yamashita <qryxip@gmail.com>"]
edition = "2018"

[package.metadata.cargo-compete.bin]
practice-a = { alias = "a", problem = "https://atcoder.jp/contests/practice/tasks/practice_1" }
practice-b = { alias = "b", problem = "https://atcoder.jp/contests/practice/tasks/practice_2" }

#[package.metadata.cargo-compete.example]

[[bin]]
name = "practice-a"
path = "src/bin/a.rs"

[[bin]]
name = "practice-b"
path = "src/bin/b.rs"

[dependencies]
num = "=0.2.1"
num-bigint = "=0.2.6"
num-complex = "=0.2.4"
num-integer = "=0.1.42"
num-iter = "=0.1.40"
num-rational = "=0.2.4"
num-traits = "=0.2.11"
num-derive = "=0.3.0"
ndarray = "=0.13.0"
nalgebra = "=0.20.0"
alga = "=0.9.3"
libm = "=0.2.1"
rand = { version = "=0.7.3", features = ["small_rng"] }
getrandom = "=0.1.14"
rand_chacha = "=0.2.2"
rand_core = "=0.5.1"
rand_hc = "=0.2.0"
rand_pcg = "=0.2.1"
rand_distr = "=0.2.2"
petgraph = "=0.5.0"
indexmap = "=1.3.2"
regex = "=1.3.6"
lazy_static = "=1.4.0"
ordered-float = "=1.0.2"
ascii = "=1.0.0"
permutohedron = "=0.2.4"
superslice = "=1.0.0"
itertools = "=0.9.0"
itertools-num = "=0.1.3"
maplit = "=1.0.2"
either = "=1.5.3"
im-rc = "=14.3.0"
fixedbitset = "=0.2.0"
bitset-fixed = "=0.1.0"
proconio = { version = "=0.3.6", features = ["derive"] }
text_io = "=0.1.8"
whiteread = "=0.5.0"
rustc-hash = "=1.1.0"
smallvec = "=1.2.0"

[dev-dependencies]
```

## Test suite

Test cases are saved as YAML files.

```yaml
# https://atcoder.jp/contests/practice/tasks/practice_1
---
type: Batch
timelimit: 2s
match: Lines

cases:
  - name: sample1
    in: |
      1
      2 3
      test
    out: |
      6 test
  - name: sample2
    in: |
      72
      128 256
      myonmyon
    out: |
      456 myonmyon

extend:
  - type: Text
    path: "./a"
    in: /in/*.txt
    out: /out/*.txt
```

```yaml
# https://atcoder.jp/contests/ddcc2019-final/tasks/ddcc2019_final_a
---
type: Batch
timelimit: 2s
match:
  Float:
    relative_error: 1e-8
    absolute_error: 1e-8

cases:
  - name: sample1
    in: |
      5
      -->--
    out: |
      3.83333333333333
  - name: sample2
    in: |
      7
      -------
    out: |
      6.5
  - name: sample3
    in: |
      10
      -->>>-->--
    out: |
      6.78333333333333

extend:
  - type: Text
    path: "./a"
    in: /in/*.txt
    out: /out/*.txt
```

```yaml
# https://judge.yosupo.jp/problem/sqrt_mod
---
type: Batch
timelimit: 10s
match:
  Checker:
    cmd: ~/.cache/online-judge-tools/library-checker-problems/math/sqrt_mod/checker "$INPUT" "$ACTUAL_OUTPUT" "$EXPECTED_OUTPUT"
    shell: Bash

cases: []

extend:
  - type: SystemTestCases
```

The format is `TestSuite` in the following schemas.

### `TestSuite`

An [internally tagged ADT](https://serde.rs/enum-representations.html#internally-tagged).

- [`TestSuite::Batch`](#testsuitebatch)
- [`TestSuite::Interactive`](#testsuiteinteractive)
- [`TestSuite::Unsubmittable`](#testsuiteunsubmittable)

### `TestSuite::Batch`

A test suite for a normal problem.

<table>
  <thead>
    <tr>
      <th align="left">Field</th>
      <th align="left">Type</th>
      <th align="left">Default</th>
      <th align="left">Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="left"><code>timelimit</code></td>
      <td align="left"><code><a href="#duration">Duration</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Time limit</td>
    </tr>
    <tr>
      <td align="left"><code>match</code></td>
      <td align="left"><code><a href="#match">Match</a></code></td>
      <td align="left"></td>
      <td align="left">Judging method</td>
    </tr>
    <tr>
      <td align="left"><code>cases</code></td>
      <td align="left"><code><a href="#case">Case</a>[]</code></td>
      <td align="left"><code>[]</code></td>
      <td align="left">Sets of input and output</td>
    </tr>
    <tr>
      <td align="left"><code>extend</code></td>
      <td align="left"><code><a href="#extend">Extend</a>[]</code></td>
      <td align="left"><code>[]</code></td>
      <td align="left">Additional sets of input and output</td>
    </tr>
  </tbody>
</table>

### `Duration`

A string that can parsed with [`humantime::format_duration`](https://docs.rs/humantime/2/humantime/fn.format_duration.html).

### `Match`

An [untagged ADT](https://serde.rs/enum-representations.html#untagged).

- [`Match::Exact`](#matchexact--exact)
- [`Match::SplitWhitespace`](#matchsplitwhitespace--splitwhitespace)
- [`Match::Lines`](#matchlines--lines)
- [`Match::Float`](#matchfloat)
- [`Match::Checker`](#matchchecker)

### `Match::Exact` = `"Exact"`

Compares whole strings.

### `Match::SplitWhiteSpace` = `"SplitWhitespace"`

Compares [words splitted by whitespace](https://doc.rust-lang.org/stable/std/primitive.str.html#method.split_whitespace).

### `Match::Lines` = `"Lines"`

Compares [lines](https://doc.rust-lang.org/stable/std/primitive.str.html#method.lines).

### `Match::Float`

Compares words [splitted by whitespace](https://doc.rust-lang.org/stable/std/primitive.str.html#method.split_whitespace).

`absolute_error` and `relative_error` are applied for pairs of words that can parsed as floating point numbers.

<table>
  <thead>
    <tr>
      <th align="left">Field</th>
      <th align="left">Type</th>
      <th align="left">Default</th>
      <th align="left">Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="left"><code>relative_error</code></td>
      <td align="left"><code><a href="#positivefinitefloat64">PositiveFiniteFloat64</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Relative error</td>
    </tr>
    <tr>
      <td align="left"><code>absolute_error</code></td>
      <td align="left"><code><a href="#positivefinitefloat64">PositiveFiniteFloat64</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Absolute error</td>
    </tr>
  </tbody>
</table>

### `PositiveFiniteFloat64`

A 64-bit floating point number that is positive and is not `inf`.

### `Match::Checker`

Checks with a shell script.

The following environment variables are given for the script.

- `INPUT`
- `ACTUAL_OUTPUT`
- `EXPECTED_OUTPUT` (only if the <code>[Case](#case).out</code> is present)

<table>
  <thead>
    <tr>
      <th align="left">Field</th>
      <th align="left">Type</th>
      <th align="left">Default</th>
      <th align="left">Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="left"><code>cmd</code></td>
      <td align="left"><a href="https://yaml.org/spec/1.2/spec.html#tag/repository/str" rel="nofollow"><code>str</code></a></td>
      <td align="left"></td>
      <td align="left">Command</td>
    </tr>
    <tr>
      <td align="left"><code>shell</code></td>
      <td align="left"><a href="#shell"><code>Shell</code></a></td>
      <td align="left"></td>
      <td align="left">Shell</td>
    </tr>
  </tbody>
</table>

### `Shell`

An [untagged ADT](https://serde.rs/enum-representations.html#untagged).

- [`Shell::Bash`](#shellbash--bash)

### `Shell::Bash` = `"Bash"`

Bash.

### `Case`

<table>
  <thead>
    <tr>
      <th align="left">Field</th>
      <th align="left">Type</th>
      <th align="left">Default</th>
      <th align="left">Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="left"><code>name</code></td>
      <td align="left"><a href="https://yaml.org/spec/1.2/spec.html#tag/repository/str" rel="nofollow"><code>str</code></a></td>
      <td align="left"><code>""</code></td>
      <td align="left">Name</td>
    </tr>
    <tr>
      <td align="left"><code>in</code></td>
      <td align="left"><a href="https://yaml.org/spec/1.2/spec.html#tag/repository/str" rel="nofollow"><code>str</code></a></td>
      <td align="left"></td>
      <td align="left">Input</td>
    </tr>
    <tr>
      <td align="left"><code>out</code></td>
      <td align="left"><code><a href="https://yaml.org/spec/1.2/spec.html#tag/repository/str" rel="nofollow">str</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Output</td>
    </tr>
    <tr>
      <td align="left"><code>timelimit</code></td>
      <td align="left"><code><a href="#duration">Duration</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Overrides <code>timelimit</code></td>
    </tr>
    <tr>
      <td align="left"><code>match</code></td>
      <td align="left"><code><a href="#match">Match</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Overrides <code>match</code></td>
    </tr>
  </tbody>
</table>

### `Extend`

An [internally tagged ADT](https://serde.rs/enum-representations.html#internally-tagged).

- [`Extend::Text`](#extendtext)
- [`Extend::SystemTestCases`](#extendsystemtestcases)

### `Extend::Text`

<table>
  <thead>
    <tr>
      <th align="left">Field</th>
      <th align="left">Type</th>
      <th align="left">Default</th>
      <th align="left">Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="left"><code>path</code></td>
      <td align="left"><a href="https://yaml.org/spec/1.2/spec.html#tag/repository/str" rel="nofollow"><code>str</code></a></td>
      <td align="left"></td>
      <td align="left">Directory</td>
    </tr>
    <tr>
      <td align="left"><code>in</code></td>
      <td align="left"><a href="#glob"><code>Glob</code></a></td>
      <td align="left"></td>
      <td align="left">Text files for input</td>
    </tr>
    <tr>
      <td align="left"><code>out</code></td>
      <td align="left"><a href="#glob"><code>Glob</code></a></td>
      <td align="left"></td>
      <td align="left">Text files for output</td>
    </tr>
    <tr>
      <td align="left"><code>timelimit</code></td>
      <td align="left"><code><a href="#duration">Duration</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Overrides <code>timelimit</code></td>
    </tr>
    <tr>
      <td align="left"><code>match</code></td>
      <td align="left"><code><a href="#match">Match</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Overrides <code>match</code></td>
    </tr>
  </tbody>
</table>

### `Glob`

A glob.

### `Extend::SystemTestCases`

System test cases.

System test cases are stored under <code>[{ cache directory }](https://docs.rs/dirs-next/2/dirs_next/fn.cache_dir.html)/cargo-compete/system-test-cases</code>.
They are automatically downloaded if missing when `test`ing code.

<table>
  <thead>
    <tr>
      <th align="left">Field</th>
      <th align="left">Type</th>
      <th align="left">Default</th>
      <th align="left">Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="left"><code>problem</code></td>
      <td align="left"><code><a href="#url">Url</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left">~</td>
      <td align="left">URL of the problem</td>
    </tr>
  </tbody>
</table>

### `Url`

A URL.

### `TestSuite::Interactive`

A test suite for an interactive problem.

<table>
  <thead>
    <tr>
      <th align="left">Field</th>
      <th align="left">Type</th>
      <th align="left">Default</th>
      <th align="left">Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="left"><code>timelimit</code></td>
      <td align="left"><code><a href="#duration">Duration</a> | <a href="https://yaml.org/spec/1.2/spec.html#tag/repository/null" rel="nofollow">null</a></code></td>
      <td align="left"><code>~</code></td>
      <td align="left">Time limit</td>
    </tr>
  </tbody>
</table>

### `TestSuite::Unsubmittable`

A dummy test suite for dummy problems such as ones in [APG4b](https://atcoder.jp/contests/APG4b).


<table>
  <thead>
    <tr>
      <th align="left">Field</th>
      <th align="left">Type</th>
      <th align="left">Default</th>
      <th align="left">Description</th>
    </tr>
  </thead>
</table>

## Cookies and tokens

The cookies and tokens are saved under <code>[{ local data directory }](https://docs.rs/dirs-next/2/dirs_next/fn.data_local_dir.html)/cargo-compete</code>.

```console
.
├── cookies.jsonl
└── tokens
    ├── codeforces.json
    ├── dropbox.json
    └── yukicoder.json
```

## Environment variables

cargo-compete reads these environment variables if they exist, and use them.

- `$DROPBOX_ACCESS_TOKEN`
- `$YUKICODER_API_KEY`
- `$CODEFORCES_API_KEY`
- `$CODEFORCES_API_SECRET`

## With [online-judge-tools](https://github.com/online-judge-tools)

For unsupported websites, `oj-api(.exe)` in the `$PATH` is used when `download`ing and `submit`ting.

```toml
[package]
name = "library-checker"
version = "0.0.0"
edition = "2018"
publish = false

[package.metadata.cargo-compete.bin]
aplusb = { problem = "https://judge.yosupo.jp/problem/aplusb" }
```

![Video](https://user-images.githubusercontent.com/14125495/104786174-9257b380-57cf-11eb-8d67-ba893ba34f22.mp4)

## Compared with cargo-atcoder

See [the section in the Japanese readme](https://github.com/qryxip/cargo-compete/blob/master/README-ja.md#cargo-atcoder%E3%81%A8%E3%81%AE%E5%AF%BE%E5%BF%9C).

## License

Dual-licensed under [MIT](https://opensource.org/licenses/MIT) or [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0).
