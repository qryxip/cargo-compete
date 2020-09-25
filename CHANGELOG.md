# Changelog

## [Unreleased]

### Fixed

- Supports "小数誤差許容問題" in yukicoder. ([qryxip/snowchains#96](https://github.com/qryxip/snowchains/pull/96))

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
