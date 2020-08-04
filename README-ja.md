# cargo-compete

[![CI](https://github.com/qryxip/cargo-compete/workflows/CI/badge.svg)](https://github.com/qryxip/cargo-compete/actions?workflow=CI)
[![codecov](https://codecov.io/gh/qryxip/cargo-compete/branch/master/graph/badge.svg)](https://codecov.io/gh/qryxip/cargo-compete/branch/master)
[![Crates.io](https://img.shields.io/crates/v/cargo-compete.svg)](https://crates.io/crates/cargo-compete)
[![Crates.io](https://img.shields.io/crates/l/cargo-compete.svg)](https://crates.io/crates/cargo-compete)

[English](https://github.com/qryxip/cargo-compete/blob/master/README.md)

競技プログラミングのためのCargoコマンドです。

## 機能

- コンテストへの参加登録
- サンプルケースを取得し、YAMLで保存
- コードのテスト
- 提出

|            | 参加登録           | テストケース (サンプル) | テストケース (全部) | 提出               | 提出を見る      | 提出一覧をwatchする |
| :--------: | :----------------: | :---------------------: | :-----------------: | :----------------: | :-------------: | :-----------------: |
| AtCoder    | :heavy_check_mark: | :heavy_check_mark:      | :heavy_check_mark:  | :heavy_check_mark: | :x:             | :grey_question:     |
| Codeforces | :x:                | :heavy_check_mark:      | N/A                 | :heavy_check_mark: | :x:             | :x:                 |
| yukicoder  | N/A                | :heavy_check_mark:      | :heavy_check_mark:  | :heavy_check_mark: | :x:             | :x:                 |

## インストール

### Crates.io

```console
$ cargo install cargo-compete
```

### `master`

```console
$ cargo install --git https://github.com/qryxip/cargo-compete
```

### GitHub Releases

[バイナリでの提供]((https://github.com/qryxip/cargo-compete/releases))もしています。

## 使い方

### `cargo compete init`

Gitリポジトリ下に、各サイトに対する[ワークスペース](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)を作ります。

![Screenshot](https://user-images.githubusercontent.com/14125495/89305770-04b55b00-d6aa-11ea-9a08-d1a4f0631d06.png)

### `cargo compete login`

サイトにログインします。

**ワークスペースやパッケージは対象に取りません。** 引数で与えられた`platform`に対してログインします。

ただし`new`コマンド等ではログインが必要になった場合でも認証情報を聞いてログインし、続行するため事前にこのコマンドを実行しなくてもよいです。

### `cargo compete participate`

コンテストに参加登録します。

**ワークスペースやパッケージは対象に取りません。** 引数で与えられた`platform`と`contest`に対して参加登録します。

同様に、`new`コマンド等で自動で参加登録するため事前にこのコマンドを実行しなくてもよいです。

### `cargo compete new`

テストケースを取得し、コンテストに応じたパッケージを作ります。

**ワークスペースを対象に取ります。**

![Screenshot](https://user-images.githubusercontent.com/14125495/89306652-206d3100-d6ab-11ea-8d33-8bf3e3419bb8.png)

`--open`で問題のページをブラウザで開きます。また`compete.toml`の`open`を設定することで、ソースコードとテストケースのYAMLをエディタで開くことができます。

![Screenshot](https://user-images.githubusercontent.com/14125495/89118593-b05f7f00-d4e1-11ea-9644-32c3560bda29.png)

### `cargo compete download`

テストケースの再取得を行います。

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**

![Screenshot](https://user-images.githubusercontent.com/14125495/89116606-04ae3300-d4d1-11ea-9306-0c3fed6a2797.png)

### `cargo compete open`

`new`の`--open`と同様に問題のページをブラウザで、コードとテストファイルをエディタで開きます。

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**

### `cargo compete test`

テストを行います。 ただし`submit`時にも提出するコードをテストします。

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**
`compete.toml`と対象パッケージの`[package.metadata]`からどのテストケースを使うかを決定します。

### `cargo compete submit`

提出を行います。

![Screenshot](https://user-images.githubusercontent.com/14125495/89117413-8786bc00-d4d8-11ea-92b3-ce71151c3d45.gif)

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**
対象パッケージの`[package.metadata]`から提出先のサイトと問題を決定します。

## 設定

設定は各ワークスペース下にある`compete.toml`にあります。
バイナリ提出関連の設定もこちらです。

```toml
# How to manage new workspace members ("include", "focus")
new-workspace-member = "include"
# Path to the test file (Liquid template)
test-suite = "./testcases/{{ contest }}/{{ problem | kebabcase }}.yml"
# Open files with the command (`jq` command)
#
# VSCode:
#open = '["code"] + (.paths | map([.src, .test_suite]) | flatten) + ["-a", .manifest_dir]'
# Emacs:
#open = '["emacsclient", "-n"] + (.paths | map([.src, .test_suite]) | flatten)'

[template]
manifest = "./cargo-compete-template/Cargo.toml"
src = "./cargo-compete-template/src/main.rs"

[platform]
kind = "atcoder"

#[platform.via-binary]
#target = "x86_64-unknown-linux-musl"
##cross = "cross"
#strip = "strip"
##upx = "upx"
```

各`bin` targetに紐付くサイト上の問題は、パッケージの`Cargo.toml`の`[package.metadata]`に記述されます。

```toml
[package]
name = "practice"
version = "0.1.0"
edition = "2018"
publish = false

[package.metadata.cargo-compete.bin]
a = { name = "practice-a", problem = { platform = "atcoder", contest = "practice", index = "A", url = "https://atcoder.jp/contests/practice/tasks/practice_1" } }
b = { name = "practice-b", problem = { platform = "atcoder", contest = "practice", index = "B", url = "https://atcoder.jp/contests/practice/tasks/practice_2" } }

[[bin]]
name = "practice-a"
path = "src/bin/a.rs"

[[bin]]
name = "practice-b"
path = "src/bin/b.rs"
```

## ライセンス

[MIT](https://opensource.org/licenses/MIT) or [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)のデュアルライセンスです。
