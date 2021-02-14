# Changelog

## [Unreleased]

### Fixed

- Added a workaround for large process input/output. ([qryxip/snowchains#121](https://github.com/qryxip/snowchains/pull/121))

- Fixed a problem where string values in YAMLs are unnecessarily quoted. ([qryxip/snowchains#121](https://github.com/qryxip/snowchains/pull/121))

    This problem was caused by [a change](https://github.com/dtolnay/serde-yaml/commit/ef990758a19d4d845cf19a8943e7d905909cafd8) in `serde-yaml v0.8.16`, which was released in February 2, 2021.

## [0.8.0] - 2021-01-24Z

### Added

- Added `add` command. ([#114](https://github.com/qryxip/cargo-compete/pull/114))

    ```toml
    # for Library-Checker
    [add]
    url = "https://judge.yosupo.jp/problem/{{ args[0] }}"
    #is-contest = ["false"] # optional
    bin-name = '{{ args[0] }}'
    #bin-alias = '{{ args[0] }}' # optional
    #bin-src-path = './src/bin/{{ bin_alias }}.rs' # optional
    ```

    ```console
    ❯ cargo compete a --full many_aplusb
        Running `/home/ryo/tools/python/3.8.6/oj/bin/oj-api get-problem 'https://judge.yosupo.jp/problem/many_aplusb' --system` in /home/ryo/src/competitive/library-checker
    ```

    ```console
    ︙
    ```

    ```console
        Added `many_aplusb` (bin) for https://judge.yosupo.jp/problem/many_aplusb
        Saved 7 test cases to /home/ryo/src/competitive/library-checker/testcases/{many_aplusb.yml, many_aplusb/}
    ```

    ```toml
    # for yukicoder
    [add]
    url = '{% case args[0] %}{% when "contest" %}https://yukicoder.me/contests/{{ args[1] }}{% when "problem" %}https://yukicoder.me/problems/no/{{ args[1] }}{% endcase %}'
    is-contest = ["bash", "-c", '[[ $(cut -d / -f 4) == "contests" ]]'] # optional
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

### Changed

- Made `package.metadata.cargo-compete.config` optional. ([#112](https://github.com/qryxip/cargo-compete/pull/112))

    ```diff
    [package.metadata.cargo-compete]
    -config = "../compete.toml"
    ```

- Added new formats for `package.metadata.cargo-compete.bin`. ([#113](https://github.com/qryxip/cargo-compete/pull/113))

    ```toml
    [package.metadata.cargo-compete.bin]
    practice-a = { alias = "a", problem = "https://atcoder.jp/contests/practice/tasks/practice_1" }
    practice-b = { alias = "b", problem = "https://atcoder.jp/contests/practice/tasks/practice_2" }
    ```

    ```toml
    [package.metadata.cargo-compete.bin]
    practice-a = { problem = "https://atcoder.jp/contests/practice/tasks/practice_1" }
    practice-b = { problem = "https://atcoder.jp/contests/practice/tasks/practice_2" }
    ```

    The old format is still valid.

### Fixed

- Fixed a problem about hyphen-separated contest IDs. ([#114](https://github.com/qryxip/cargo-compete/pull/114))

## [0.7.1] - 2021-01-21Z

### Changed

- Improved around Dropbox. ([qryxip/snowchains#116](https://github.com/qryxip/snowchains/pull/116))

### Fixed

- Fixed URL parsing for Codeforces. ([qryxip/snowchains#119](https://github.com/qryxip/snowchains/pull/119))
- The `init` command creates target paths. ([#105](https://github.com/qryxip/cargo-compete/pull/105))
- Fixed a problem where generated package names are set to `"contest"` for Codeforces. ([#108](https://github.com/qryxip/cargo-compete/pull/108) by [@tamuhey](https://github.com/tamuhey))

## [0.7.0] - 2021-01-16Z

### Added

- Added `bin_name` and `bin_alias` variables for `test-suite`. ([#96](https://github.com/qryxip/cargo-compete/pull/96))

- Enabled `download`ing/`submit`ting with [online-judge-tools](https://github.com/online-judge-tools/oj). ([#98](https://github.com/qryxip/cargo-compete/pull/98))

    `oj-api` in `$PATH` will be used if the domain of a problem URL is unknown.
    To use this new function, you need to install online-judge-tools first.

    ```toml
    [package.metadata.cargo-compete.bin]
    aplusb = { name = "aplusb", problem = "https://judge.yosupo.jp/problem/aplusb" }
    ```

### Changed

- Simplified `package.metadata.cargo-compete.bin.*.problem`. ([#96](https://github.com/qryxip/cargo-compete/pull/96))

    - Removed `package.metadata.cargo-compete.bin.*.problem.{platform, contest, index}`.
    - URLs are now required.
    - The `problem = { url = ".." }` format is still supported.
    - `contest` in `test-suite` for yukicoder will be always `null`.

    ```diff
     [package.metadata.cargo-compete.bin]
    -a = { name = "arc110-a", problem = { platform = "atcoder", contest = "arc110", index = "A", url = "https://atcoder.jp/contests/arc110/tasks/arc110_a" } }
    -b = { name = "arc110-b", problem = { platform = "atcoder", contest = "arc110", index = "B", url = "https://atcoder.jp/contests/arc110/tasks/arc110_b" } }
    -c = { name = "arc110-c", problem = { platform = "atcoder", contest = "arc110", index = "C", url = "https://atcoder.jp/contests/arc110/tasks/arc110_c" } }
    -d = { name = "arc110-d", problem = { platform = "atcoder", contest = "arc110", index = "D", url = "https://atcoder.jp/contests/arc110/tasks/arc110_d" } }
    -e = { name = "arc110-e", problem = { platform = "atcoder", contest = "arc110", index = "E", url = "https://atcoder.jp/contests/arc110/tasks/arc110_e" } }
    -f = { name = "arc110-f", problem = { platform = "atcoder", contest = "arc110", index = "F", url = "https://atcoder.jp/contests/arc110/tasks/arc110_f" } }
    +a = { name = "arc110-a", problem = "https://atcoder.jp/contests/arc110/tasks/arc110_a" }
    +b = { name = "arc110-b", problem = "https://atcoder.jp/contests/arc110/tasks/arc110_b" }
    +c = { name = "arc110-c", problem = "https://atcoder.jp/contests/arc110/tasks/arc110_c" }
    +d = { name = "arc110-d", problem = "https://atcoder.jp/contests/arc110/tasks/arc110_d" }
    +e = { name = "arc110-e", problem = "https://atcoder.jp/contests/arc110/tasks/arc110_e" }
    +f = { name = "arc110-f", problem = "https://atcoder.jp/contests/arc110/tasks/arc110_f" }
    ```

### Deprecated

- Deprecated `problem` variable for `test-suite`. ([#96](https://github.com/qryxip/cargo-compete/pull/96))

    Use `bin_alias` instead.

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
    ```

### Removed

- Removed the `migrate v04` command. ([#100](https://github.com/qryxip/cargo-compete/pull/100))

- Removed the `submit.via-binary` configration. ([#100](https://github.com/qryxip/cargo-compete/pull/100))

    Use [cargo-executable-payload](https://github.com/qryxip/cargo-executable-payload) instead.

    ```toml
    [submit.transpile]
    kind = "command"
    args = ["cargo", "executable-payload", "--bin", "{{ bin_name }}"]
    ```

## [0.6.5] - 2020-12-05Z

### Added

- Introduced [`serde_ignored` crate](https://docs.rs/serde_ignored).

    As Cargo does, cargo-compete warns for unused keys in `compete.toml` and in `package.metadata`.

    ```console
    ❯ cargo compete o
    warning: unused key in compete.toml: oepn
    ```

- [`test`, `submit`] Added `test.profile` option.

    Now you can test your programs with release mode by default.

    ```toml
    [new.template]
    profile = '''
    [release]
    debug-assertions = true
    overflow-checks = true
    '''

    # ...

    [test]
    # Profile for `cargo build`. ("dev" | "release")
    #
    # Defaults to `"dev"`.
    profile = "release"
    ```

    [Profiles - The Cargo Book](https://doc.rust-lang.org/cargo/reference/profiles.html)

## [0.6.4] - 2020-11-24Z

### Added

- [`new`] Added `new.template.profile`.

    ```toml
    [new.template]
    lockfile = "./template-cargo-lock.toml"
    # `profile` for `Cargo.toml`.
    #
    # By setting this, you can run tests with `opt-level=3` while enabling `debug-assertions` and `overflow-checks`.
    profile = '''
    [dev]
    opt-level = 3
    '''
    ```

    [Profiles - The Cargo Book](https://doc.rust-lang.org/cargo/reference/profiles.html)

### Fixed

- Problem indexes for yukicoder contests will be alphabets. ([qryxip/snowchains/#102](https://github.com/qryxip/snowchains/pull/102))

    Previously, "problem no"s were set.

- Stopped asking username and password when you have already logged in AtCoder. ([qryxip/snowchains/#106](https://github.com/qryxip/snowchains/pull/106))

## [0.6.3] - 2020-10-12Z

### Added

- [`open`] Now supports `stirng[][]` output for the `open` configuration.

    ```toml
    # Open files with the command (`jq` command that outputs `string[] | string[][]`)
    open = '[["code", "-a", .manifest_dir], ["code"] + (.paths | map([.src, .test_suite]) | flatten)]'
    ```

## [0.6.2] - 2020-10-09Z

### Fixed

- Updated the dependencies.
    - Now it supports "小数誤差許容問題" in yukicoder. ([qryxip/snowchains#96](https://github.com/qryxip/snowchains/pull/96))

## [0.6.1] - 2020-09-20Z

### Fixed

- [`submit`] `submit.via-binary` will be correctly recognized. Previously, `cargo-compete` tried to read `submit.via-bianry`.

## [0.6.0] - 2020-09-06Z

### Added

- [`new`] Added `contest` variable to `new.path`.

### Changed

- [`new`] Package names will be `"contest{}"` to adapt to [rust-lang/cargo#7959](https://github.com/rust-lang/cargo/pull/7959).

    Modify `new.path` as following.

    ```diff
    -path = "./{{ package_name }}"
    +path = "./{{ contest }}"
    ```

### Fixed

- [`new`] `new.path` will work correctly.
- [`new`] `new` command for yukicoder will work properly.

## [0.5.1] - 2020-09-04Z

### Added

- [`submit`] Added `submit.transpile` field.

    You will be able to convert Rust code before submitting.

    ```toml
    [submit.transpile]
    kind = "command"
    args = ["cargo", "equip", "--oneline", "mods", "--rustfmt", "--check", "--bin", "{{ bin_name }}"]
    #language_id = ""
    ```

## [0.5.0] - 2020-08-31Z

### Added

- [`migrate`] Added `migrate v04` command. ([#58](https://github.com/qryxip/cargo-compete/pull/58))
- [`new`, `open`] Added `git_workdir` variable for `open` in `compete.toml`. ([#56](https://github.com/qryxip/cargo-compete/pull/56))

### Changed

- `cargo-compete` no longer manage workspaces. Instead, each package will just share the same `target` directory. Run `cargo compete migrate v04` to migrate packages. ([#58](https://github.com/qryxip/cargo-compete/pull/58))
- Changed the format of `compet.toml`. ([#58](https://github.com/qryxip/cargo-compete/pull/58))

## [0.4.7] - 2020-08-25Z

### Added

- [`retrieve testcases`] Enabled downloading all of the test cases on Dropbox. ([qryxip/snowchains#89](https://github.com/qryxip/snowchains/pull/89))

## [0.4.6] - 2020-08-23Z

### Fixed

- [`submit`] Added workaround for the problem that <kbd>C-c</kbd> does not work. ([#52](https://github.com/qryxip/cargo-compete/pull/52))
- [`sumibt`, `watch`] Fixed a cosmetic problem. ([qryxip/snowchains#86](https://github.com/qryxip/snowchains/pull/86))

## [0.4.5] - 2020-08-20Z

### Added

- [`test`, `submit`] Enabled specifying a `bin` with `src_path` instead of index for `package.metadata.cargo-compete.bin`. ([#49](https://github.com/qryxip/cargo-compete/pull/49))

    ```console
    $ cargo compete s a
    ```

    ```console
    $ cargo compete s --src ./contest/src/bin/a.rs
    ```

### Fixed

- [`new`, `download`] Against AtCoder, retrieving sample cases proceeds when encountered scraping errors. ([qryxip/snowchains_core#80](https://github.com/qryxip/snowchains/pull/80))

## [0.4.4] - 2020-08-19Z

### Changed

- Improved error messages. ([#47](https://github.com/qryxip/cargo-compete/pull/47))

### Added

- Now each command can be run against a virtual manifest when the workspace has exactly one member. ([#47](https://github.com/qryxip/cargo-compete/pull/47))

## [0.4.3] - 2020-08-18Z

### Fixed

- Support [ABC003](https://atcoder.jp/contests/abc003) and [ABC007](https://atcoder.jp/contests/abc007) to [ABC010](https://atcoder.jp/contests/abc010). ([qryxip/snowchains_core#76](https://github.com/qryxip/snowchains/pull/76))

## [0.4.2] - 2020-08-17Z

### Added

- [`new`] Now empty virtual workspaces are supported.

## [0.4.1] - 2020-08-12Z

### Changed

- [`init`] Now `cargo compete init` creates a `.gitignore` when it is missing.
- [`new`, `open`] CWD for `jq` is set to the workspace root.

## [0.4.0] - 2020-08-09Z

### Changed

- [`compete.toml`] Changed the behavior of `exclude`.
- [rename] Renamed `migrate packages` command to `migrate cargo-atcoder`.

### Added

- [`compete.toml`] `skip` variant to `new-workspace-members`.

### Fixed

- [`migrate cargo-atcoder`] Now `cargo compete migrate` also generates a template package as `cargo compete init` does.

## [0.3.2] - 2020-08-08Z

### Added

- Added `migrate packages` command.

## [0.3.1] - 2020-08-07Z

### Changed

- On `submit` command, now it snows the result before `watch`ing the submission. ([#19](https://github.com/qryxip/cargo-compete/pull/19))

### Fixed

- Fixed the parser for AtCoder submissions. ([qryxip/snowchains#71](https://github.com/qryxip/snowchains/pull/71))

## [0.3.0] - 2020-08-06Z

### Changed

#### `compete.toml`

- Changed the format. ([#17](https://github.com/qryxip/cargo-compete/pull/17))

### Added

#### New commands

- Added `retrieve submission summaries` command. ([#16](https://github.com/qryxip/cargo-compete/pull/16))

#### `compete.toml`

- Added `exclude` value to `new-workspace-member`. ([#14](https://github.com/qryxip/cargo-compete/pull/14))
- Added `manifest_dir` variable for `test-suite`. ([#15](https://github.com/qryxip/cargo-compete/pull/15))

## [0.2.2] - 2020-08-05Z

### Added

- Added `--testcases <NAME>...` option to `test` command and `submit` command. ([#12](https://github.com/qryxip/cargo-compete/pull/12))

## [0.2.1] - 2020-08-04Z

### Fixed

- Previously `cargo compete test` was trying to test cross-compiled program even on Windows and macOS. Now it just execute `"$CARGO" build` without `--target`. ([#10](https://github.com/qryxip/cargo-compete/pull/10))

## [0.2.0] - 2020-08-04Z

### Changed

- Splitted the `download` command into `new`, `open`, and `download`. ([#3](https://github.com/qryxip/cargo-compete/pull/3))
- Renamed `workspace-metadata.toml` to `compete.toml`. ([#7](https://github.com/qryxip/cargo-compete/pull/7))
    - Now `test-suite`s are [Liquid](https://shopify.github.io/liquid/) templates.  ([#4](https://github.com/qryxip/cargo-compete/pull/4))
    - Now `open`s are [jq](https://github.com/stedolan/jq) commands. ([#5](https://github.com/qryxip/cargo-compete/pull/5))
    - Renamed `template.code` to `template.src`.  ([#8](https://github.com/qryxip/cargo-compete/pull/8))
    - Now `template.manifest` instead of `template.dependencies` is used for each `Cargo.toml`.  ([#8](https://github.com/qryxip/cargo-compete/pull/8))

### Fixed

- Fixed `package.repository` of this package. ([#2](https://github.com/qryxip/cargo-compete/pull/3))
