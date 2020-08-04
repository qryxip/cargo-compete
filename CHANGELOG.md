# Changelog

## [Unreleased]

### Changed

- Splitted the `download` command into `new`, `open`, and `download`. ([#3](https://github.com/qryxip/cargo-compete/pull/3))
- Changed the format of `workspace-metadata.toml`.
    - Now `cargo-compete.test-suite`s are [Liquid](https://shopify.github.io/liquid/) templates.  ([#4](https://github.com/qryxip/cargo-compete/pull/4))
    - Now `cargo-compete.open`s are [jq](https://github.com/stedolan/jq) commands. ([#5](https://github.com/qryxip/cargo-compete/pull/5))

### Fixed

- Fixed `package.repository` of this package. ([#2](https://github.com/qryxip/cargo-compete/pull/3))
