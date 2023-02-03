# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/jeertmans/languagetool-rust/compare/v1.3.0...HEAD)

### Chore

- Created founding link. [#19](https://github.com/jeertmans/languagetool-rust/pull/19)
- Added two related projets. [#21](https://github.com/jeertmans/languagetool-rust/pull/21)
- Added release dates. [#31](https://github.com/jeertmans/languagetool-rust/pull/31)
- Added `#[must_use]` flag to most structures, to please clippy pendantic. [#29](https://github.com/jeertmans/languagetool-rust/pull/29)
- Changed conditional compilation flags to directly point to dependency, e.g., `"clap"` instead of `"cli"`. [#28](https://github.com/jeertmans/languagetool-rust/pull/28)
- Use `cargo-nextest` instead of `cargo test` for faster CI testing. [#32](https://github.com/jeertmans/languagetool-rust/pull/32)
- Improve CI testing. [#41](https://github.com/jeertmans/languagetool-rust/pull/41)

### Added

- Added `cli-complete` feature to generate completion files. [#23](https://github.com/jeertmans/languagetool-rust/pull/23)
- Added message when reading from STDIN. [#25](https://github.com/jeertmans/languagetool-rust/pull/25), [#26](https://github.com/jeertmans/languagetool-rust/pull/26)
- Added (regex) validator for language code. [#27](https://github.com/jeertmans/languagetool-rust/pull/27)
- Added cli requirements for `username`/`api_key` pair. [#16](https://github.com/jeertmans/languagetool-rust/pull/16), [#30](https://github.com/jeertmans/languagetool-rust/pull/30)

### Fixed

- Stopped serializing useless fields. [#17](https://github.com/jeertmans/languagetool-rust/pull/17)

## [1.3.0](https://github.com/jeertmans/languagetool-rust/compare/v1.2.0...v1.3.0) - 2022-08-25

### Chore

- Fixed features flag in `CI.yml` action. [#12](https://github.com/jeertmans/languagetool-rust/pull/12)

### Added

- Added basic Docker support. [#12](https://github.com/jeertmans/languagetool-rust/pull/12)

### Fixed

- Fixed typo in message when no error was found. [#12](https://github.com/jeertmans/languagetool-rust/pull/12)

## [1.2.0](https://github.com/jeertmans/languagetool-rust/compare/v1.1.1...v1.2.0) - 2022-08-10

### Chore

- Use vendored TLS for release. [#14](https://github.com/jeertmans/languagetool-rust/pull/14)
- Fixed PR links in CHANGELOG. [#15](https://github.com/jeertmans/languagetool-rust/pull/15)

### Added

- Add 3 new features `native-tls`, `native-tls-vendored` and `full`. [#14](https://github.com/jeertmans/languagetool-rust/pull/14)

## [1.1.1](https://github.com/jeertmans/languagetool-rust/compare/v1.0.1...v1.1.1) - 2022-08-09

### Chore

- Add GitHub action to automate release process. [#12](https://github.com/jeertmans/languagetool-rust/pull/12)

## [1.1.0](https://github.com/jeertmans/languagetool-rust/compare/v1.0.0...v1.1.0) - 2022-08-08

### Chore

- Add some LanguageTool context.
- Correct some typos in the various docs ([@kianmeng](https://github.com/kianmeng)). [#4](https://github.com/jeertmans/languagetool-rust/pull/4)
- Add lint test for Markdown files ([@kianmeng](https://github.com/kianmeng)). [#5](https://github.com/jeertmans/languagetool-rust/pull/5)
- Add grammar check for text-like files. [#6](https://github.com/jeertmans/languagetool-rust/pull/6)
- Add compare links to release tags in CHANGELOG. [#9](https://github.com/jeertmans/languagetool-rust/pull/9)
- Add action to check if can publish. [#11](https://github.com/jeertmans/languagetool-rust/pull/11)

### Added

- Add `get_text` method for `CheckRequest`. [#8](https://github.com/jeertmans/languagetool-rust/pull/8)
- Create `CheckResponseWithContext` that enables keeping information about checked text. Hence, mutable and immutable iterators have been created to provide more tools to the end-user. [#10](https://github.com/jeertmans/languagetool-rust/pull/10)
- Add `--more-context` option flag to CLI. This enables to add more information in the JSON output. [#10](https://github.com/jeertmans/languagetool-rust/pull/10)
- Derive `Eq` for all structs that derive `PartialEq` and with fields that derive `Eq`. [#10](https://github.com/jeertmans/languagetool-rust/pull/10)
- Add `from_env` and `from_env_or_default` methods for `ServerCli` and `ServerClient`. [#10](https://github.com/jeertmans/languagetool-rust/pull/10)

### Fixed

- Fixed line number in annotated responses. [#8](https://github.com/jeertmans/languagetool-rust/pull/8)
- Fixed missing bench path in `Cargo.toml`. [#11](https://github.com/jeertmans/languagetool-rust/pull/11)

## [1.0.0](https://github.com/jeertmans/languagetool-rust/compare/v0.0.18...v1.0.0) - 2022-07-24

### Chore

- Add GitHub actions for MSRV.
- Enhance GitHub actions' set by adding more checks.
- Add `pre-commit.ci` linting.
- Add GitHub action with dockerized LanguageTool server for more advanced tests.
- Remove `cliff`'s changelog generation and opt for a manual changelog maintenance.
- Create "changelog-bot" that requires pull-requests to contain diffs in CHANGELOG.md.

### Added

- Add loading `hostname` and `port` from env. variables if `feature = "cli"` is set.

### Changed

- Bump MSRV to 1.57.
- Changed some `&str` parameters to `String` when `to_string()` was called inside the function.
- Add `[non_exhaustive]` flag to all structures that rely on the LT server API.

### Fixed

- Lowered dependency versions and using non-strict versions.
- Fix compilation error in `src/lib/error.rs` when `feature = "cli"` was not set.
- Remove unused print in `src/lib/server.rs`.

> **_NOTE:_** Pre v1.0.0, the changelog was generated using the `cliff` tool that is based on commits.

## [0.0.18](https://github.com/jeertmans/languagetool-rust/compare/v0.0.17...v0.0.18) - 2022-06-22

### 🆕 Features

- [**cli/lib**] Allow for max. number of suggestions </br> └─ This feature is very likely to change in the future
- [**lib**] Allow to easily convert strings into `Replacement`s

### 🖋️ Styling

- [**lib**] Rustfmt and unused import

### 🗃️ Miscellaneous Tasks

- [**readme**] Fix typo in readme
- [**tests**] Added test for errors
- [**docs**] 100% of src/lib is now documented

## [0.0.17](https://github.com/jeertmans/languagetool-rust/compare/v0.0.16...v0.0.17) - 2022-06-21

### 🆕 Features

- [**lib**] Default value to "auto" for language in Default
- [**lib**] Clean `DataAnnotation` constructors </br> └─ Provides simplified constructors for the `DataAnnotation` structure for the different possible cases.
- [**lib**] Deriving `PartialEq` for `Data` and `DataAnnotation`
- [**lib**] Implement `FromIterator` for `Data` & tests </br> └─ This allows for an easy construction of requests with markup data.
- [**lib**] Implement `From<ServerCli>` for `ServerClient`

### 🖋️ Styling

- [**fmt**] Rustfmt
- [**lib**] Rustfmt

### 🗃️ Miscellaneous Tasks

- [**clean**] Change `to_string` to `to_owned`
- [**lib**] Documentation some add. functions
- [**version**] Update version & changelog

## [0.0.16](https://github.com/jeertmans/languagetool-rust/compare/v0.0.15...v0.0.16) - 2022-06-08

### 🆕 Features

- [**bin**] Allow to read text from stdin </br> └─ This changes the default behavior where `--text` or `--data` was required. Now, if none are provided, ltrs will read from stdin.

### 🖋️ Styling

- [**fmt**] Rust fmt
- [**lib**] Clippy suggestion

### 🗃️ Miscellaneous Tasks

- [**fmt**] Rust fmt
- [**diet**] Ran cargo diet to save space
- [**cliff**] Enhance changelog appearance </br> └─ This adds emojis as well a more content to the changelog messages
- [**ci**] Updated GitHub action to avoid integration error </br> └─ See [this issue](https://github.com/actions-rs/clippy-check/issues/2) for example
- [**version**] Update version & changelog

## [0.0.15](https://github.com/jeertmans/languagetool-rust/compare/v0.0.14...v0.0.15) - 2022-06-01

### 🗃️ Miscellaneous Tasks

- [**deps**] Reduce necessary dependencies </br> └─ Reduce the amount of features loaded from `tokio` and only require this feature to be activated for bin and tests.
- [**version**] Update version & changelog

### 🩹 Bug Fixes

- [**lib**] Avoid panicking on string slices </br> └─ This fixes the problem that occurred when the string was not made of purely utf-8 chars. E.g.: `ltrs check --text "Un essai de texte en français à controler"`

## [0.0.14](https://github.com/jeertmans/languagetool-rust/compare/v0.0.13...v0.0.14) - 2022-05-31

### 🗃️ Miscellaneous Tasks

- [**readme**] Fix file link in readme
- [**version**] Update version & changelog

## [0.0.13](https://github.com/jeertmans/languagetool-rust/compare/v0.0.12...v0.0.13) - 2022-05-31

### 🆕 Features

- [**cli**] Annotate text from check response

### 🗃️ Miscellaneous Tasks

- [**readme**] Document new feature
- [**readme**] Fix typo
- [**future**] Add some todos
- [**cli**] Derive display order
- [**version**] Update version & changelog

## [0.0.12](https://github.com/jeertmans/languagetool-rust/compare/v0.0.11...v0.0.12) - 2022-05-31

### 🆕 Features

- [**src**] Create new "unstable" feature
- [**lib**] Make response fields public and add "unstable" attribute

### 🗃️ Miscellaneous Tasks

- [**check**] Renaming `with_data` to `with_data_str`
- [**readme**] Refactor readme based on clap's readme
- [**version**] Update version & changelog

### 🩹 Bug Fixes

- [**readme**] Fix changelog link in readme

## [0.0.11](https://github.com/jeertmans/languagetool-rust/compare/v0.0.10...v0.0.11) - 2022-05-24

### 🗃️ Miscellaneous Tasks

- [**doc**] Refactor readme and include it in lib.rs' doc
- [**naming**] Renamed server variables to client
- [**version**] Update version & changelog

### 🩹 Bug Fixes

- [**doc**] Automatic links in docstrings

## [0.0.10](https://github.com/jeertmans/languagetool-rust/compare/v0.0.9...v0.0.10) - 2022-05-23

### 🆕 Features

- [**lib**] Add (basic) support for add. languages

### 🗃️ Miscellaneous Tasks

- [**clean**] Remove useless comments in code
- [**doc**] Document config file struct
- [**version**] Update version & changelog

### 🩹 Bug Fixes

- [**lib**] Now correctly writes arrays in config file
- [**test**] Fixing doctest

## [0.0.9](https://github.com/jeertmans/languagetool-rust/compare/v0.0.9-beta...v0.0.9) - 2022-05-20

### 🗃️ Miscellaneous Tasks

- [**doc**] Auto-doc for "feature"-only commands
- [**version**] Update version & changelog

## [0.0.9-beta](https://github.com/jeertmans/languagetool-rust/compare/v0.0.9-alpha...v0.0.9-beta) - 2022-05-20

### 🗃️ Miscellaneous Tasks

- [**version**] Update version & changelog

### 🩹 Bug Fixes

- [**CI**] Now publish with cli feature

## [0.0.9-alpha](https://github.com/jeertmans/languagetool-rust/compare/v0.0.8...v0.0.9-alpha) - 2022-05-20

### 🗃️ Miscellaneous Tasks

- [**tests**] Add some doctests
- [**features**] Remove default features
- [**version**] Update version & changelog

## [0.0.8](https://github.com/jeertmans/languagetool-rust/compare/v0.0.7...v0.0.8) - 2022-05-20

### 🆕 Features

- [**lib**] Derive serialize for words' responses
- [**lib**] New errors system </br> └─ Errors and results are now unified under a common type that can be found inside `lib/error.rs`

### 🗃️ Miscellaneous Tasks

- [**lib**] Refactor lib. exposed functions
- [**doc**] Update description
- [**deps**] Remove unused crate and add thiserror crate
- [**version**] Update version & changelog

### 🩹 Bug Fixes

- [**cli**] Text takes now 't' as short flag
- [**lib**] IDs and categories are String

## [0.0.7](https://github.com/jeertmans/languagetool-rust/compare/v0.0.6...v0.0.7) - 2022-05-18

### 🆕 Features

- [**errors**] Response errors are now used </br> └─ But not the body of the error, only the url
- [**cli**] Changed language to `auto` </br> └─ language flag was required, now it defaults to `auto`

### 🗃️ Miscellaneous Tasks

- [**errors**] Now exits as usage error for two commands
- [**lint**] Rustfmt
- [**typo**] Fixed typo in docstring
- [**docs**] Split docs for `-h` and `--help`
- [**readme**] Update todos & usage
- [**version**] Update version & changelog

## [0.0.6](https://github.com/jeertmans/languagetool-rust/compare/v0.0.5...v0.0.6) - 2022-05-18

### 🆕 Features

- [**cli**] Pretty print json output

### 🗃️ Miscellaneous Tasks

- [**doc**] Improve docs
- [**refactor**] Server mod and tests
- [**clippy**] Removed useless ref.
- [**defaults**] Change default hostname api </br> └─ Now uses online server from LanguageTool
- [**version**] Update version & changelog

## [0.0.5](https://github.com/jeertmans/languagetool-rust/compare/v0.0.4...v0.0.5) - 2022-05-17

### 🆕 Features

- [**cli**] Add ping command

### 🗃️ Miscellaneous Tasks

- [**version**] Update version & changelog

## [0.0.4](https://github.com/jeertmans/languagetool-rust/compare/v0.0.3...v0.0.4) - 2022-05-17

### 🆕 Features

- [**data**] Now correctly uses `data` field + pub fields </br> └─ `data` field is not correctly serialized for the `check` request

### 🗃️ Miscellaneous Tasks

- [**readme**] Update todos
- [**version**] Update version & changelog

## [0.0.3](https://github.com/jeertmans/languagetool-rust/compare/v0.0.2...v0.0.3) - 2022-05-17

### 🆕 Features

- [**cli**] Create a nice cli

### 🗃️ Miscellaneous Tasks

- [**readme**] Add new todos
- [**version**] Update version & changelog
- [**version**] Update version in Cargo.lock

## [0.0.2](https://github.com/jeertmans/languagetool-rust/compare/v0.0.1...v0.0.2) - 2022-05-16

### 🆕 Features

- [**features**] Now Clap is a feature required for bin

### 🗃️ Miscellaneous Tasks

- [**version**] Update version & changelog

### 🩹 Bug Fixes

- [**src/lib/api**] Removed empty file

## [0.0.1](https://github.com/jeertmans/languagetool-rust/tree/v0.0.1) - 2022-05-16

### 🆕 Features

- [**src**] First unstable code version

### 🗃️ Miscellaneous Tasks

- [**ci/lint/doc**] Basic GitHub setup
- [**changelog**] First changelog

<!-- generated by git-cliff -->
