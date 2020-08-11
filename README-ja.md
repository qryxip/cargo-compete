# cargo-compete

[![CI](https://github.com/qryxip/cargo-compete/workflows/CI/badge.svg)](https://github.com/qryxip/cargo-compete/actions?workflow=CI)
[![codecov](https://codecov.io/gh/qryxip/cargo-compete/branch/master/graph/badge.svg)](https://codecov.io/gh/qryxip/cargo-compete/branch/master)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Crates.io](https://img.shields.io/crates/v/cargo-compete.svg)](https://crates.io/crates/cargo-compete)
[![Crates.io](https://img.shields.io/crates/l/cargo-compete.svg)](https://crates.io/crates/cargo-compete)

[English](https://github.com/qryxip/cargo-compete/blob/master/README.md)

競技プログラミングのためのCargoコマンドです。

## 機能

- コンテストへの参加登録
- サンプルケースを取得し、YAMLで保存
- コードのテスト
- 提出

|            | 参加登録           | テストケース (サンプル) | テストケース (全部) | 提出               | 提出一覧をwatchする | 提出の詳細を見る |
| :--------: | :----------------: | :---------------------: | :-----------------: | :----------------: | :-----------------: | :--------------: |
| AtCoder    | :heavy_check_mark: | :heavy_check_mark:      | :heavy_check_mark:  | :heavy_check_mark: | :grey_question:     | :x:              |
| Codeforces | :x:                | :heavy_check_mark:      | N/A                 | :heavy_check_mark: | :x:                 | :x:              |
| yukicoder  | N/A                | :heavy_check_mark:      | :heavy_check_mark:  | :heavy_check_mark: | :x:                 | :x:              |

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

### `cargo compete migrate cargo-atcoder`

`cargo-atcoder`で作ったパッケージを、ワークスペースにまとめて`cargo-compete`用にマイグレートします。

![Screenshot](https://user-images.githubusercontent.com/14125495/89726038-1489c200-da51-11ea-93ae-d317a13f04e9.png)

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

![Screenshot](https://user-images.githubusercontent.com/14125495/89712134-d5fcf480-d9c9-11ea-9124-bcd0caabb545.png)

`--open`で問題のページをブラウザで開きます。また`compete.toml`の`open`を設定することで、ソースコードとテストケースのYAMLをエディタで開くことができます。

![Screenshot](https://user-images.githubusercontent.com/14125495/89712184-25432500-d9ca-11ea-92fe-f699b6e0cfab.png)

`--open`を付け忘れた場合は[`cargo compete open`](#cargo-compete-open)で開いてください。

### `cargo compete retrieve testcases` / `cargo compete download`

テストケースの再取得を行います。

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**

![Screenshot](https://user-images.githubusercontent.com/14125495/89116606-04ae3300-d4d1-11ea-9306-0c3fed6a2797.png)

### `cargo compete retrieve submission-summaries`

自分の提出の一覧を取得し、JSONで出力します。

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**

![Record](https://user-images.githubusercontent.com/14125495/89495297-f7f04e80-d7f2-11ea-9973-88763993e70a.gif)

例えばAtCoderであれば(AtCoderしか実装してませんが)`| jq -r '.summaries[0].detail`とすることで「最新の提出の詳細ページのURL」が得られます。

```console
$ # 最新の提出の詳細ページをブラウザで開く (Linuxの場合)
$ xdg-open "$(cargo compete r ss | jq -r '.summaries[0].detail')"
```

### `cargo compete open`

`new`の`--open`と同様に問題のページをブラウザで、コードとテストファイルをエディタで開きます。

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**

### `cargo compete test`

テストを行います。

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**
`compete.toml`と対象パッケージの`[package.metadata]`からどのテストケースを使うかを決定します。

`submit`時にも提出するコードをテストするため、提出前にこのコマンドを実行しておく必要はありません。

### `cargo compete submit`

提出を行います。

![Record](https://user-images.githubusercontent.com/14125495/89711920-40149a00-d9c8-11ea-86ec-154f662405ed.gif)

**パッケージを対象に取ります。パッケージ内に`cd`して実行してください。**
対象パッケージの`[package.metadata]`から提出先のサイトと問題を決定します。

## 設定

設定は各ワークスペース下にある`compete.toml`にあります。
バイナリ提出関連の設定もこちらです。

```toml
# How to manage new workspace members ("include" | "exclude" | "focus")
#
# - `skip`:    Does not modify `[workspace]`
# - `include`: Adds the package to `workspace.members`
# - `exclude`: Adds the package to `workspace.exclude` and create a symlink to the `compete.toml`
# - `focus`:   Adds the package to `workspace.members` and remove the other from both of `workspace.{members, exclude}`
new-workspace-member = "include"

# Path to the test file (Liquid template)
#
# Variables:
#
# - `manifest_dir`: Package directory
# - `contest`:      Contest ID (e.g. "abc100")
# - `problem`:      Problem index (e.g. "a", "b")
#
# Additional filters:
#
# - `kebabcase`: Convert to kebab case (by using the `heck` crate)
test-suite = "./testcases/{{ contest }}/{{ problem | kebabcase }}.yml"
#test-suite = "{{ manifest_dir }}/testcases/{{ problem | kebabcase }}.yml"

# Open files with the command (`jq` command)
#
# VSCode:
#open = '["code"] + (.paths | map([.src, .test_suite]) | flatten) + ["-a", .manifest_dir]'
# Emacs:
#open = '["emacsclient", "-n"] + (.paths | map([.src, .test_suite]) | flatten)'

[template]
platform = "atcoder"
manifest = "./cargo-compete-template/Cargo.toml"
src = "./cargo-compete-template/src/main.rs"

[submit-via-binary]
target = "x86_64-unknown-linux-musl"
#cross = "cross"
strip = "strip"
#upx = "upx"
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

## cargo-atcoderとの対応

### `cargo atcoder new`

[`cargo compete new`](#cargo-compete-new)でパッケージを作成します。

[`compete.toml`](#設定)があるワークスペースから実行する必要があります。
[`cargo compete init`](#cargo-compete-init)でワークスペースを作成するか、[`cargo compete migrate cargo-atcoder`](#cargo-compete-migrate-cargo-atcoder)でパッケージ達をマイグレートしてください。

[`compete.toml`](#設定)の`new-workspace-member`が`"include"`または`"focus"`の場合、他の既存のパッケージとビルドキャッシュを共有します。
クレートを使う場合も初回を除いて"warmup"は不要です。

`"exclude"`の場合独立したワークスペースが作られます。
こちらは`cargo atcoder new`の挙動に近いです。
ただし`cargo compete submit`等のコマンドのために`compete.toml`のシンボリックリンクが作られます。
Windows上では一般ユーザーでシンボリックリンクを作れるようにしてください。

なお、開始前のコンテストには使えません。
ビルドキャッシュを共有する限り"warmup"が不要なためです。
ブラウザとエディタを開くのも`--open`で自動で行えます。

### `cargo atcoder submit`

[`cargo compete submit`](#cargo-compete-submit)でコード、または「エンコード」したコードを提出します。

他のコマンドと同様に、ワークスペース下に[`compete.toml`](#設定)がある必要があります。

「バイナリ提出」を行う場合の設定は[`compete.toml`](#設定)にあります。

### `cargo atcoder test`

[`cargo compete test`](#cargo-compete-test)でテストを実行します。

他のコマンドと同様に、ワークスペース下に[`compete.toml`](#設定)がある必要があります。

一部のテストのみを実行する場合は、`<case-num>...`の代わりに`--testcases <NAME>...`で`"sample1"`等の「名前」で絞ります。

### `cargo atcoder login`

[`cargo compete login`](#cargo-comepte-login)でログインします。

### `cargo atcoder status`

`cargo compete watch submissions`で提出一覧をwatchします。

注意として、cargo-competeの方はブラウザ上の表示に近い挙動をします。
実行時点で「ジャッジ待ち」/「ジャッジ中」のものが無い場合、直近20件を表示だけして終了します。

### `cargo atcoder result`

今のところありません。 [`cargo compete retrieve submissions`](#cargo-compete-retrieve-submissions)の出力を`| jq -r ".summaries[$nth].detail"`して得たURLをブラウザで開いてください。

### `cargo atcoder clear-session`

今のところありません。 [local data directory](https://docs.rs/dirs/3/dirs/fn.data_local_dir.html)下の`cargo-compete`を削除してください。

### `cargo atcoder info`

今のところありません。 ログインしているかを確認する場合、[practice contest](https://atcoder.jp/contests/practice)のテストケースをダウンロードしてください。 practice contestの場合問題の閲覧にログインが必要です。

### `cargo atcoder warmup`

今のところありません。上で述べた通り、`target`ディレクトリを共有する場合初回を除きwarmupは不要です。

### `cargo atcoder gen-binary`

今のところありません。
`cargo compete submit`で作られるコードはファイルシステムに置かれません。
このリポジトリの[`resources/exec-base64-encoded-binary.rs.liquid`](https://github.com/qryxip/cargo-compete/blob/master/resources/exec-base64-encoded-binary.rs.liquid)に、`source_code`と`base64`のパラメータを与えたものが提出されます。

## ライセンス

[MIT](https://opensource.org/licenses/MIT) or [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)のデュアルライセンスです。
