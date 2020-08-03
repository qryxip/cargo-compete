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

```console
$ cargo install cargo-compete
```

または

```console
$ cargo install --git https://github.com/qryxip/cargo-compete
```

[GitHub Releases](https://github.com/qryxip/cargo-compete/releases)でバイナリを提供しています。

## 使い方

### `cargo compete init`

Gitリポジトリ下に、各サイトに対する[ワークスペース](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)を作ります。

AtCoderを選択に入れる場合、

1. クレートを使用しない
2. クレートを使用する
3. バイナリ提出を行なう

の三択を聞き、それに応じた設定を生成します。

![Screenshot](https://user-images.githubusercontent.com/14125495/89116339-16daa200-d4ce-11ea-9c5d-0a67aa958ce3.png)

### `cargo compete login`

`download`コマンド等ではログインが必要であれば認証情報を要求し、ログインしますが`cargo compete login`で明示的にログインできます。

**ワークスペースやパッケージは対象に取りません。** 引数で与えられた`platform`に対してログインします。

### `cargo compete participate`

コンテストに参加登録します。 同様に、`download`コマンド等で参加登録は行なわれますがこのコマンドで明示的に参加登録ができます。

**ワークスペースやパッケージは対象に取りません。** 引数で与えられた`platform`と`contest`に対して参加登録します。

### `cargo compete download`

**インターフェイスがよろしくないのでv0.2.0で`new`, `open`, `download`の3つに分割する予定です。** ([#3](https://github.com/qryxip/cargo-compete/pull/3))

テストケースの取得を行います。

**workspace rootかworkspace memberのどちらかを対象に取ります。**

- workspace rootを対象にした場合、新たにパッケージを作成します。

    ![Screenshot](https://user-images.githubusercontent.com/14125495/89116540-2d81f880-d4d0-11ea-8d6d-14e077cbfa3d.png)

- 既存のworkspace memberを対象にした場合、テストケースを再取得するだけです。

    ![Screenshot](https://user-images.githubusercontent.com/14125495/89116606-04ae3300-d4d1-11ea-9306-0c3fed6a2797.png)

`--open`で問題のページを開きます。また`workspace-metadata.toml`の`open`を設定することで、ソースコードとテストケースのYAMLをVSCodeまたはEmacsで開くことができます。

![Screenshot](https://user-images.githubusercontent.com/14125495/89118593-b05f7f00-d4e1-11ea-9644-32c3560bda29.png)

### `cargo compete test`

テストを行ないます。 ただし`submit`時にも提出するコードをテストします。

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。`workspace-metadata.toml`と対象パッケージの`[package.metadata]`からどのテストケースを使うかを決定します。**

### `cargo compete submit`

提出を行ないます。

![Screenshot](https://user-images.githubusercontent.com/14125495/89117413-8786bc00-d4d8-11ea-92b3-ce71151c3d45.gif)

**`test`と同様にパッケージを対象に取ります。パッケージ内に`cd`して実行してください。対象パッケージの`[package.metadata]`から提出先のサイトと問題を決定します。**

## 設定

設定は

- 各ワークスペース下にある`workspace-metadata.toml`
- 各パッケージの`Cargo.toml`に書かれた`[package.metadata.cargo-compete.bin]`

にあります。 バイナリ提出関連の設定もこちらです。

```toml
[cargo-compete]
new-workspace-member = "include" # "include", "focus"
test-suite = "./testcases/{contest}/{problem | kebab-case}.yml"
#open = "vscode" # "vscode", "emacsclient"

[cargo-compete.template]
code = "./cargo-compete-template/src/main.rs"

[cargo-compete.template.dependencies]
proconio = { version = "=0.3.6", features = ["derive"] }

[cargo-compete.platform]
kind = "atcoder"

[cargo-compete.platform.via-binary]
target = "x86_64-unknown-linux-musl"
#cross = "cross"
strip = "strip"
#upx = "upx"
```

```toml
[package]
name = "practice"
version = "0.1.0"
edition = "2018"
publish = false

[package.metadata.cargo-compete.bin]
a = { name = "practice-a", problem = { platform = "atcoder", contest = "practice", index = "A" } }
b = { name = "practice-b", problem = { platform = "atcoder", contest = "practice", index = "B" } }

[[bin]]
name = "practice-a"
path = "src/bin/a.rs"

[[bin]]
name = "practice-b"
path = "src/bin/b.rs"
```

## ライセンス

[MIT](https://opensource.org/licenses/MIT) or [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)のデュアルライセンスです。
