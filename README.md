# CosmWasm Plus Utils (Updated for Secret Network v1)

This repo contains some packages from [cw-plus](https://github.com/CosmWasm/cw-plus) that have been updated to work with the Secret Network version of Cosmwasm v1.

[![CircleCI](https://circleci.com/gh/CosmWasm/cw-plus/tree/main.svg?style=shield)](https://circleci.com/gh/CosmWasm/cw-plus/tree/main)

| Utilities       | Crates.io                                                                                                                        | Docs                                                                                  | Coverage                                                                                                                                  |
|-----------------|----------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------|
| cw-multi-test   | [![cw-multi-test on crates.io](https://img.shields.io/crates/v/cw-multi-test.svg)](https://crates.io/crates/cw-multi-test)       | [![Docs](https://docs.rs/cw-multi-test/badge.svg)](https://docs.rs/cw-multi-test)     | [![codecov](https://codecov.io/gh/CosmWasm/cw-plus/branch/main/graph/badge.svg?token=IYY72ZVS3X)](https://codecov.io/gh/CosmWasm/cw-plus) |                                                                                                                                          |
| cw-storage-plus | [![cw-storage-plus on crates.io](https://img.shields.io/crates/v/cw-storage-plus.svg)](https://crates.io/crates/cw-storage-plus) | [![Docs](https://docs.rs/cw-storage-plus/badge.svg)](https://docs.rs/cw-storage-plus) | [![codecov](https://codecov.io/gh/CosmWasm/cw-plus/branch/main/graph/badge.svg?token=IYY72ZVS3X)](https://codecov.io/gh/CosmWasm/cw-plus) |
| cw-utils        | [![cw-utils on crates.io](https://img.shields.io/crates/v/cw-utils.svg)](https://crates.io/crates/cw-utils)                      | [![Docs](https://docs.rs/cw-utils/badge.svg)](https://docs.rs/cw-utils)               | [![codecov](https://codecov.io/gh/CosmWasm/cw-plus/branch/main/graph/badge.svg?token=IYY72ZVS3X)](https://codecov.io/gh/CosmWasm/cw-plus) |

If you don't know what CosmWasm is, please check out
[our homepage](https://cosmwasm.com) and
[our documentation](https://docs.cosmwasm.com) to get more background.
We are running a [public testnet](https://github.com/CosmWasm/testnets/blob/master/sandynet-1/README.md)
you can use to test out any contracts.

**Warning** None of these contracts have been audited and no liability is
assumed for the use of this code. They are provided to turbo-start
your projects.

**Note** All code in pre-1.0 packages is in "draft" form, meaning it may
undergo minor changes and additions until 1.0. For example between 0.1 and
0.2 we adjusted the `Expiration` type to make the JSON representation
cleaner (before: `expires: {at_height: {height: 12345}}` after
`expires: {at_height: 12345}`)

## Compiling

To compile all the contracts, run the following in the repo root:

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.6
```

This will compile all packages in the `contracts` directory and output the
stripped and optimized wasm code under the `artifacts` directory as output,
along with a `checksums.txt` file.

If you hit any issues there and want to debug, you can try to run the
following in each contract dir:
`RUSTFLAGS="-C link-arg=-s" cargo build --release --target=wasm32-unknown-unknown --locked`

## Quality Control

One of the basic metrics of assurance over code quality is how much is covered by
unit tests. There are several tools available for Rust to do such analysis and
we will describe one below. This should be used as a baseline metric to give some
confidence in the code.

Beyond code coverage metrics, just having a robust PR review process with a few
more trained eyes looking for bugs is very helpful in detecting paths the original
coder was not aware of. This is more subjective, but looking at the relevant PRs
and depth of discussion can give an idea how much review was present.

After that, fuzzing it (ideally with an intelligent fuzzer that understands the domain)
can be valuable. And beyond that formal verification can provide even more assurance
(but is very time consuming and expensive).

### Code Coverage

I recommend the use of [tarpaulin](https://github.com/xd009642/tarpaulin): `cargo install cargo-tarpaulin`

To get some nice interactive charts, you can go to the root directory and run:

`cargo tarpaulin -o html`
and then `xdg-open tarpaulin-report.html` (or just `open` on MacOS).

Once you find a package that you want to improve, you can do the following to just
analyze this package, which gives much faster turn-around:

`cargo tarpaulin -o html --packages cw3-fixed-multisig`

Note that it will produce a code coverage report for the entire project, but only the coverage in that
package is the real value. If does give quick feedback for you if you unit test writing was successful.

## Generating changelog

To generate a changelog we decided to use [github-changelog-generator](https://github.com/github-changelog-generator/github-changelog-generator).

To install tool you need Ruby's `gem` package manager.

    $ gem --user install github_changelog_generator

And put `$HOME/.gem/ruby/*/bin/` into your PATH.

Generating changelog file first time:

    $ github_changelog_generator -u CosmWasm -p cw-plus

Appending next releases could be done adding `--base` flag:

    $ github_changelog_generator -u CosmWasm -p cw-plus --base CHANGELOG.md

If you hit GitHub's 50 requests/hour limit, please follow [this](https://github.com/github-changelog-generator/github-changelog-generator#github-token)
guide to create a token key which you can pass using `--token` flag.

There's also a convenience `scripts/update_changelog.sh`, which can take a
--since-tag parameter (to avoid processing the entire history). It can also
auto-detect the latest version tag for you, with --latest-tag.

## Licenses

This repo contains two license, [Apache 2.0](./LICENSE-APACHE) and
[AGPL 3.0](./LICENSE-AGPL.md). All crates in this repo may be licensed
as one or the other. Please check the `NOTICE` in each crate or the
relevant `Cargo.toml` file for clarity.

All *specifications* will always be Apache-2.0. All contracts that are
meant to be *building blocks* will also be Apache-2.0. This is along
the lines of Open Zepellin or other public references.

Contracts that are "ready to deploy" may be licensed under AGPL 3.0 to
encourage anyone using them to contribute back any improvements they
make. This is common practice for actual projects running on Ethereum,
like Uniswap or Maker DAO.
