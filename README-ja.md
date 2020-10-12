# cargo-compete

[![CI](https://github.com/qryxip/cargo-compete/workflows/CI/badge.svg)](https://github.com/qryxip/cargo-compete/actions?workflow=CI)
[![codecov](https://codecov.io/gh/qryxip/cargo-compete/branch/master/graph/badge.svg)](https://codecov.io/gh/qryxip/cargo-compete/branch/master)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Crates.io](https://img.shields.io/crates/v/cargo-compete.svg)](https://crates.io/crates/cargo-compete)
[![Crates.io](https://img.shields.io/crates/l/cargo-compete.svg)](https://crates.io/crates/cargo-compete)
[![Join the chat at https://gitter.im/cargo-compete/community](https://badges.gitter.im/cargo-compete/community.svg)](https://gitter.im/cargo-compete/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

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

[バイナリでの提供](https://github.com/qryxip/cargo-compete/releases)もしています。

## 使い方

### `cargo compete init`

他のコマンドのためにいくつかのファイルを生成します。最初に実行してください。

- [`compete.toml`](#設定)

    他のコマンドに必要です。cargo-atcoderのように自動で生成しません。

- [`rust-toolchain`](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file)

    `cargo`と`rustc`のバージョンを指定するテキストファイルまたはTOMLファイルです。
    AtCoder用なら`1.42.0`と書けば、`rust-toolchain`を置いたディレクトリ下で`~/.cargo/bin/cargo(.exe)`を起動したときに1.42.0のものが呼ばれるようになります。

- [`.cargo/config.toml`](https://doc.rust-lang.org/cargo/reference/config.html)

    `build/target-dir`を設定し、`target`ディレクトリを共有するようにします。

- `.template-cargo-lock.toml`

    [`cargo compete new`](#cargo-compete-new)に使う`Cargo.lock`のテンプレートです。
    質問に「AtCoderでクレートを使用するがバイナリ提出はしない」と回答した場合のみ生成されます。
    生成された場合、`compete.toml`の`new.template.lockfile`にこのファイルへのパスが追加されます。

![Screenshot](https://user-images.githubusercontent.com/14125495/91646306-b7e65980-ea88-11ea-8f0c-f11080b914ed.png)

### `cargo compete migrate cargo-atcoder`

`cargo-atcoder`で作ったパッケージをそれぞれ`cargo-compete`用にマイグレートし、`compete.toml`等のファイルも追加します。

![Screenshot](https://user-images.githubusercontent.com/14125495/91646437-2a0b6e00-ea8a-11ea-8374-14a2564ed6d3.png)

### `cargo compete login`

サイトにログインします。

**パッケージを対象に取りません。** 引数で与えられた`platform`に対してログインします。

ただし`new`コマンド等ではログインが必要になった場合でも認証情報を聞いてログインし、続行するため事前にこのコマンドを実行しなくてもよいです。

### `cargo compete participate`

コンテストに参加登録します。

**パッケージを対象に取りません。** 引数で与えられた`platform`と`contest`に対して参加登録します。

同様に、`new`コマンド等で自動で参加登録するため事前にこのコマンドを実行しなくてもよいです。

### `cargo compete new`

テストケースを取得し、コンテストに応じたパッケージを作ります。

**[`compete.toml`](#設定)を起点とします。**
最初に[`cargo compete init`](#cargo-compete-init)で生成してください。

`--open`で問題のページをブラウザで開きます。
また`compete.toml`の`open`を設定することで、ソースコードとテストケースのYAMLをエディタで開くことができます。
`--open`を付け忘れた場合は生成されたパッケージに`cd`した後に[`cargo compete open`](#cargo-compete-open)で開いてください。

![Record](https://user-images.githubusercontent.com/14125495/91647287-1b29b900-ea94-11ea-9053-43e25c77706f.gif)

[`compete.toml`](#設定)の`new-workspace-member`が`"include"`の場合、他の既存のパッケージとビルドキャッシュを共有します。
クレートを使う場合も初回を除いて"warmup"は不要です。

### `cargo compete retrieve testcases` / `cargo compete download`

テストケースの再取得を行います。

**パッケージを対象に取ります。**
パッケージに`cd`して実行してください。

![Screenshot](https://user-images.githubusercontent.com/14125495/91647644-06e7bb00-ea98-11ea-8bc6-cd57714e4c84.png)

プラットフォームが使っているテストケースを公開している場合、`--full`を指定することでそちらをダウンロードすることができます。

AtCoderの場合、[テストケースはDropboxにアップロードされている](https://atcoder.jp/posts/20)のでそちらからダウンロードします。ただし[Dropbox API](https://www.dropbox.com/developers/documentation/http/overview)を使用するため

- `files.metadata.read`
- `sharing.read`

の2つのパーミッションが有効なアクセストークンが必要です。
何らかの方法でアクセストークンを取得し、以下の形式のJSONファイルを<code>[{data local directory}](https://docs.rs/dirs/3/dirs/fn.data_local_dir.html)/cargo-compete/tokens/dropbox.json</code>に保存してください。
(この辺はなんとかしたいと考えてます)

```json
{
  "access_token": "<access token>"
}
```

![Record](https://user-images.githubusercontent.com/14125495/91647905-c722d280-ea9b-11ea-88e8-e8c81b3ce555.gif)

### `cargo compete retrieve submission-summaries`

自分の提出の一覧を取得し、JSONで出力します。

**パッケージを対象に取ります。**
パッケージに`cd`して実行してください。

![Record](https://user-images.githubusercontent.com/14125495/91647691-765daa80-ea98-11ea-8378-b8631f8f3752.gif)

例えばAtCoderであれば(AtCoderしか実装してませんが)`| jq -r '.summaries[0].detail`とすることで「最新の提出の詳細ページのURL」が得られます。

```console
$ # 最新の提出の詳細ページをブラウザで開く (Linuxの場合)
$ xdg-open "$(cargo compete r ss | jq -r '.summaries[0].detail')"
```

### `cargo compete open`

`new`の`--open`と同様に問題のページをブラウザで、コードとテストファイルをエディタで開きます。

**パッケージを対象に取ります。**
パッケージに`cd`して実行してください。

### `cargo compete test`

テストを行います。

**パッケージを対象に取ります。**
パッケージに`cd`して実行してください。

`compete.toml`と対象パッケージの`[package.metadata]`からどのテストケースを使うかを決定します。

`submit`時にも提出するコードをテストするため、提出前にこのコマンドを実行しておく必要はありません。

### `cargo compete submit`

提出を行います。

**パッケージを対象に取ります。**
パッケージに`cd`して実行してください。

対象パッケージの`[package.metadata]`から提出先のサイトと問題を決定します。

![Record](https://user-images.githubusercontent.com/14125495/91647583-511c6c80-ea97-11ea-941c-884070a3182a.gif)

[`compete.toml`](#設定)の`submit.transpile`を設定することで、[cargo-equip](https://github.com/qryxip/cargo-equip)等のコード変換ツールを使って提出するコードを変換できます。

```toml
[submit.transpile]
kind = "command"
args = ["cargo", "equip", "--oneline", "mods", "--rustfmt", "--check", "--bin", {% raw %}"{{ bin_name }}"{% endraw %}]
#language_id = ""
```

## 設定

設定は各ワークスペース下にある`compete.toml`にあります。
バイナリ提出関連の設定もこちらです。

```toml
# Path to the test file (Liquid template)
#
# Variables:
#
# - `manifest_dir`: Package directory
# - `contest`:      Contest ID (e.g. "abc100")
# - `problem`:      Problem index (e.g. "A", "B")
#
# Additional filters:
#
# - `kebabcase`: Convert to kebab case (by using the `heck` crate)
test-suite = "{{ manifest_dir }}/testcases/{{ problem | kebabcase }}.yml"
#test-suite = "./testcases/{{ contest }}/{{ problem | kebabcase }}.yml"

# Open files with the command (`jq` command that outputs `string[] | string[][]`)
#
# VSCode:
#open = '[["code", "-a", .manifest_dir], ["code"] + (.paths | map([.src, .test_suite]) | flatten)]'
# Emacs:
#open = '["emacsclient", "-n"] + (.paths | map([.src, .test_suite]) | flatten)'

[new]
platform = "atcoder"
path = "./{{ package_name }}"

[new.template]
lockfile = "./template-cargo-lock.toml"

[new.template.dependencies]
kind = "inline"
content = '''
#proconio = { version = "=0.3.6", features = ["derive"] }
'''

[new.template.src]
kind = "inline"
content = '''
fn main() {
    todo!();
}
'''

#[submit.transpile]
#kind = "command"
#args = ["cargo", "equip", "--oneline", "mods", "--rustfmt", "--check", "--bin", "{{ bin_name }}"]
#language_id = ""

#[submit.via-binary]
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
authors = ["Ryo Yamashita <qryxip@gmail.com>"]
edition = "2018"

[package.metadata.cargo-compete]
config = "../compete.toml"

[package.metadata.cargo-compete.bin]
a = { name = "practice-a", problem = { platform = "atcoder", contest = "practice", index = "A", url = "https://atcoder.jp/contests/practice/tasks/practice_1" } }
b = { name = "practice-b", problem = { platform = "atcoder", contest = "practice", index = "B", url = "https://atcoder.jp/contests/practice/tasks/practice_2" } }

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
```

## cargo-atcoderとの対応

### `cargo atcoder new`

[`cargo compete new`](#cargo-compete-new)でパッケージを作成します。

[`compete.toml`](#設定)を起点とします。
[`cargo compete init`](#cargo-compete-init)か[`cargo compete migrate cargo-atcoder`](#cargo-compete-migrate-cargo-atcoder)で作成してください。

なお、開始前のコンテストには使えません。
`target`ディレクトリを共有する限り"warmup"が不要なためです。
ブラウザとエディタを開くのも`--open`で自動で行えます。

### `cargo atcoder submit`

[`cargo compete submit`](#cargo-compete-submit)でコード、または「エンコード」したコードを提出します。

他のコマンドと同様に、ワークスペース下に[`compete.toml`](#設定)がある必要があります。

「バイナリ提出」を行う場合の設定は[`compete.toml`](#設定)にあります。

### `cargo atcoder test`

[`cargo compete test`](#cargo-compete-test)でテストを実行します。

cargo-atcoderと同様にパッケージを対象に取ります。

一部のテストのみを実行する場合は、`<case-num>...`の代わりに`--testcases <NAME>...`で`"sample1"`等の「名前」で絞ります。

### `cargo atcoder login`

[`cargo compete login`](#cargo-comepte-login)でログインします。

### `cargo atcoder status`

[`cargo compete watch submission-summaries`](#cargo-compete-watch-submission-summaries)で提出一覧をwatchします。

注意として、cargo-competeの方はブラウザ上の表示に近い挙動をします。
実行時点で「ジャッジ待ち」/「ジャッジ中」のものが無い場合、直近20件を表示だけして終了します。

### `cargo atcoder result`

今のところありません。 [`cargo compete watch submission-summaries`](#cargo-compete-watch-submission-summaries)の出力を`| jq -r ".summaries[$nth].detail"`して得たURLをブラウザで開いてください。

### `cargo atcoder clear-session`

今のところありません。 [data local directory](https://docs.rs/dirs/3/dirs/fn.data_local_dir.html)下の`cargo-compete`を削除してください。

### `cargo atcoder info`

今のところありません。 ログインしているかを確認する場合、[practice contest](https://atcoder.jp/contests/practice)のテストケースをダウンロードしてください。 practice contestの場合問題の閲覧にログインが必要です。

### `cargo atcoder warmup`

今のところありません。上で述べた通り、`target`ディレクトリを共有する場合初回を除きwarmupは不要です。

### `cargo atcoder gen-binary`

今のところありません。
[`cargo compete submit`](#cargo-compete-submit)で作られるコードはファイルシステムに置かれません。
このリポジトリの[`resources/exec-base64-encoded-binary.rs.liquid`](https://github.com/qryxip/cargo-compete/blob/master/resources/exec-base64-encoded-binary.rs.liquid)に、`source_code`と`base64`のパラメータを与えたものが提出されます。

## ライセンス

[MIT](https://opensource.org/licenses/MIT) or [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)のデュアルライセンスです。
