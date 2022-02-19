# Changelog

## [0.10.4] - 2022-02-19Z

### Fixed

- Fixed the problem where test files are mixed up for recent ABC. ([#186](https://github.com/qryxip/cargo-compete/pull/187) by [@aoriso](https://github.com/aoriso))

## [0.10.3] - 2022-01-29Z

### Fixed

- (also applied to the previous versions unless `--locked`) Accepts "Ex" problems of ABC. ([qryxip/snowchains#147](https://github.com/qryxip/snowchains/pull/147))
- (also applied to the previous versions unless `--locked`) Allows `" \n"` as output text. ([qryxip/snowchains#149](https://github.com/qryxip/snowchains/pull/149) by [@nebocco](https://github.com/nebocco))

## [0.10.2] - 2021-12-15Z

### Changed

- (also applied to the previous versions unless `--locked`) Updated `serde-yaml` crate to v0.8.23.

    ```diff
     match:
       Float:
    -    relative_error: 0.000001
    -    absolute_error: 0.000001
    +    relative_error: 1e-6
    +    absolute_error: 1e-6
    ```

    [dtolnay/serde-yaml@`6b83603`](https://github.com/dtolnay/serde-yaml/commit/6b836037b58ebb359e7c485fc6002b1e8214bd6c)

### Fixed

- Updated Rust edition, Rust version, and the language ID for Codeforces. ([#181](https://github.com/qryxip/cargo-compete/issues/181) by [@nebocco](https://github.com/nebocco))

## [0.10.1] - 2021-12-09Z

### Fixed

- Fixed a problem where [`timeLimit`](https://github.com/online-judge-tools/api-client#format) is not read when using `oj-api`. ([#178](https://github.com/qryxip/cargo-compete/pull/178) by [@SGThr7](https://github.com/SGThr7))

## [0.10.0] - 2021-11-06Z

### Added

- Added `template.new.edition` field. ([#175](https://github.com/qryxip/cargo-compete/issues/175))

    ```toml
    # `edition` for `Cargo.toml`.
    edition = "2018"
    ```

- Added `test.toolchain` field. ([#173](https://github.com/qryxip/cargo-compete/issues/173))

    ```toml
    # Toolchain for the test. (optional)
    toolchain = "1.42.0"
    ```

### Changed

- `cargo compete init` no longer generate `rust-toolchain`s. ([#173](https://github.com/qryxip/cargo-compete/issues/173))

### Fixed

- Updated the Rust versions for `cargo compete init`. ([#175](https://github.com/qryxip/cargo-compete/issues/175))

- Inserts a newline between `bin` and `dependencies`. ([#175](https://github.com/qryxip/cargo-compete/issues/175))

    ```diff
     [[bin]]
     name = "practice-b"
     path = "src/bin/b.rs"
    +
     [dependencies]
    ```

## [0.9.1] - 2021-09-21Z

### Fixed

- When using `oj-api`, prioritize `alphabet`s from `get-contest` over ones from `get-problem`. ([#166](https://github.com/qryxip/cargo-compete/pull/166) by [@bouzuya](https://github.com/bouzuya))

    Now you can use cargo-compete for [AtCoder Problems](https://kenkoooo.com/atcoder/#/contest/recent).

## [0.9.0] - 2021-03-31Z

### Added

- Added `template.new.dev-dependencies`. ([#152](https://github.com/qryxip/cargo-compete/pull/152))

    ```toml
    profile = '''
    [dev]
    opt-level = 3
    '''
    [template.new]
    dependencies = '''
    proconio = "0.3.7"
    '''
    dev-dependencies = '''
    atcoder-202004-lock = { git = "https://github.com/qryxip/atcoder-202004-lock" }
    '''
    ```

- Enabled running for `example` targets. ([#157](https://github.com/qryxip/cargo-compete/pull/157))

    ```toml
    [package.metadata.cargo-compete.example]
    atcoder-abc188-a = { problem = "https://atcoder.jp/contests/abc188/tasks/abc188_a" }
    ```

- Added `add.target-kind` configuration. ([#157](https://github.com/qryxip/cargo-compete/pull/157))

### Changed

- Modified the template for the `init` command. ([#156](https://github.com/qryxip/cargo-compete/pull/156))

    ```diff
     #[submit.transpile]
     #kind = "command"
    -#args = ["cargo", "equip", "--resolve-cfgs", "--remove", "docs", "--minify", "libs", "--rustfmt", "--check", "--bin", "{{ bin_name }}"]
    +#args = ["cargo", "equip", "--exclude-atcoder-crates", "--resolve-cfgs", "--remove", "docs", "--minify", "libs", "--rustfmt", "--check", "--bin", "{{ bin_name }}"]
     ##language_id = ""
    ```

- Updated `rust-toolchain`s for Codeforces and yukicoder. ([#153](https://github.com/qryxip/cargo-compete/pull/153))

- Now `download` command requires `--overwrite` flag to overwrite existing test files. ([#158](https://github.com/qryxip/cargo-compete/pull/158))

    ```console
    ❯ cargo compete d
    error: `/home/ryo/src/local/competitive/atcoder/arc115/testcases/a.yml` already exists. run with `--overwrite` to overwrite
    ❯ cargo compete d --overwrite
           Saved 2 test cases to /home/ryo/src/local/competitive/atcoder/arc115/testcases/{a.yml, a/}
           Saved 2 test cases to /home/ryo/src/local/competitive/atcoder/arc115/testcases/{b.yml, b/}
           Saved 1 test case to /home/ryo/src/local/competitive/atcoder/arc115/testcases/{c.yml, c/}
           Saved 2 test cases to /home/ryo/src/local/competitive/atcoder/arc115/testcases/{d.yml, d/}
           Saved 2 test cases to /home/ryo/src/local/competitive/atcoder/arc115/testcases/{e.yml, e/}
           Saved 5 test cases to /home/ryo/src/local/competitive/atcoder/arc115/testcases/{f.yml, f/}
    ```

## [0.8.8] - 2021-03-10Z

### Added

- [Added `SplitWhitespace` variant to `Match`](https://github.com/qryxip/cargo-compete#matchsplitwhitespace--splitwhitespace). ([qryxip/snowchains#136](https://github.com/qryxip/snowchains/pull/136))

- cargo-compete now warns when the expected output and the actual one are not matched and whitespace-separated words do. ([qryxip/snowchains#137](https://github.com/qryxip/snowchains/pull/137))

    ```console
    2/2 ("sample2") Wrong Answer (0 ms)
    stdin:
    3 3
    3 3 3
    expected:
    EMPTY
    actual:

    note:
    whitespace-separated words matched. try setting `match` to `SplitWhitespace`
    error: 1/2 tests failed
    ```

## [0.8.7] - 2021-02-26Z

### Changed

- Made `Extend::SystemTestCases.problem` optional. ([#qryxip/snowchains#133](https://github.com/qryxip/snowchains/pull/133), [#141](https://github.com/qryxip/cargo-compete/pull/141))

    ```diff
     extend:
       - type: SystemTestCases
    -    problem: https://atcoder.jp/contests/agc001/tasks/agc001_a
    ```

### Fixed

- Fixed around <kbd>Ctrl-c</kbd>. ([#qryxip/snowchains#135](https://github.com/qryxip/snowchains/pull/135))

## [0.8.6] - 2021-02-25Z

### Added

- Added [`SystemTestCases` variant](https://github.com/qryxip/cargo-compete#extendsystemtestcases) to `extend` in test suite files. ([qryxip/snowchains#131](https://github.com/qryxip/snowchains/pull/131), [#138](https://github.com/qryxip/cargo-compete/pull/138))

    System test cases are stored under <code>[{ cache directory }](https://docs.rs/dirs-next/2/dirs_next/fn.cache_dir.html)/cargo-compete/system-test-cases</code>.
    They are automatically downloaded if missing when `test`ing code.

    ```yaml
    extend:
      - type: SystemTestCases
        problem: "https://atcoder.jp/contests/abc191/tasks/abc191_a"
    ```

### Changed

- `cargo compete test` command without `--full` option will append `{ type: Text, ... }` to `extend` and will create empty `in` and `out` directories. ([#138](https://github.com/qryxip/cargo-compete/pull/138))

    ```console
    ❯ cargo compete n arc110 --problems a
        Created `arc110` package at /home/ryo/src/competitive/atcoder/./arc110
        Saved 2 test cases to /home/ryo/src/competitive/atcoder/./arc110/testcases/{a.yml, a/}
    ❯ tree ./arc110/testcases
    ./arc110/testcases
    ├── a
    │   ├── in
    │   └── out
    └── a.yml

    3 directories, 1 file
    ```

    ```yaml
    ---
    type: Batch
    timelimit: 2s
    match: Lines

    cases:
      - name: sample1
        in: |
          3
        out: |
          7
      - name: sample2
        in: |
          10
        out: |
          39916801

    extend:
      - type: Text
        path: "./a"
        in: /in/*.txt
        out: /out/*.txt
    ```

- `SystemTestCases` will be used for `--full` option. ([#138](https://github.com/qryxip/cargo-compete/pull/138))

### Fixed

- Fixed a problem where `cargo compete download` command saves nothing. ([#139](https://github.com/qryxip/cargo-compete/pull/139))

## [0.8.5] - 2021-02-23Z

### Fixed

- Fixed [a problem where processes are not killed for timeout](https://github.com/qryxip/cargo-compete/issues/135). ([qryxip/snowchains#129](https://github.com/qryxip/snowchains/pull/129))

## [0.8.4] - 2021-02-19Z

### Added

- Added `template` field to `compete.toml`.

    ```toml
    [template]
    src = '''
    fn main() {
        todo!();
    }
    '''

    [template.new]
    profile = '''
    [dev]
    opt-level = 3
    '''
    dependencies = '''
    proconio = { version = "=0.3.6", features = ["derive"] }
    # ...
    '''

    [template.new.copy-files]
    "./template-cargo-lock.toml" = "Cargo.lock"
    ```

- Made `new` and `new.template` optional.

### Deprecated

- Deprecated the `new.template` config.

    Use `template` instead.

### Fixed

- Fixed [a problem where `Cargo.lock` was not copied when running `new` command](https://github.com/qryxip/cargo-compete/issues/131).

## [0.8.3] - 2021-02-18Z

### Added

- cargo-compete now reads these environment variables if they exist, and use them. ([#129](https://github.com/qryxip/cargo-compete/pull/129))

    - `$DROPBOX_ACCESS_TOKEN`
    - `$YUKICODER_API_KEY`
    - `$CODEFORCES_API_KEY`
    - `$CODEFORCES_API_SECRET`

### Fixed

- Added `#[serde(default)]` to `PartialBatchTestCase::out: Option<Arc<str>>`. ([qryxip/snowchains/#128](https://github.com/qryxip/snowchains/pull/128))

    Previously, explicit `out: ~` had been allowed but the field itself was required.

## [0.8.2] - 2021-02-15Z

### Added

- Added `Checker` variant to `Match`. ([qryxip/snowchains#124](https://github.com/qryxip/snowchains/pull/124))

    ```yaml
    match:
      Checker:
        cmd: cat "$ACTUAL_OUTPUT" | cargo run --bin check-a
        shell: Bash
    ```

    ```yaml
    match:
      Checker:
        cmd: ~/.cache/online-judge-tools/library-checker-problems/math/sqrt_mod/checker "$INPUT" "$ACTUAL_OUTPUT" "$EXPECTED_OUTPUT"
        shell: Bash
    ```

## [0.8.1] - 2021-02-14Z

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
