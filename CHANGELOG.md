# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

### Chore

- Add some LanguageTool context.
- Correct some typos in the various docs ([@kianmeng](https://github.com/kianmeng)). [#4](https://github.com/jeertmans/languagetool-rust/pull/4)
- Add lint test for Markdown files ([@kianmeng](https://github.com/kianmeng)). [#5](https://github.com/jeertmans/languagetool-rust/pull/5)
- Add grammar check for text-like files. [#6](https://github.com/jeertmans/languagetool-rust/pull/6)

## [1.0.0] - 2022-07-24

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

## [0.0.18] - 2022-06-22

### 🆕 Features

-  [**cli/lib**] Allow for max. number of suggestions </br> └─ This feature is very likely to change in the future
-  [**lib**] Allow to easily convert strings into `Replacement`s

### 🖋️ Styling

-  [**lib**] Rustfmt and unused import

### 🗃️ Miscellaneous Tasks

-  [**readme**] Fix typo in readme
-  [**tests**] Added test for errors
-  [**docs**] 100% of src/lib is now documented

## [0.0.17] - 2022-06-21

### 🆕 Features

-  [**lib**] Default value to "auto" for language in Default
-  [**lib**] Clean `DataAnnotation` constructors </br> └─ Provides simplified constructors for the `DataAnnotation` structure for the different possible cases.
-  [**lib**] Deriving `PartialEq` for `Data` and `DataAnnotation`
-  [**lib**] Implement `FromIterator` for `Data` & tests </br> └─ This allows for an easy construction of requests with markup data.
-  [**lib**] Implement `From<ServerCli>` for `ServerClient`

### 🖋️ Styling

-  [**fmt**] Rustfmt
-  [**lib**] Rustfmt

### 🗃️ Miscellaneous Tasks

-  [**clean**] Change `to_string` to `to_owned`
-  [**lib**] Documentation some add. functions
-  [**version**] Update version & changelog

## [0.0.16] - 2022-06-08

### 🆕 Features

-  [**bin**] Allow to read text from stdin </br> └─ This changes the default behavior where `--text` or `--data` was required. Now, if none are provided, ltrs will read from stdin.

### 🖋️ Styling

-  [**fmt**] Rust fmt
-  [**lib**] Clippy suggestion

### 🗃️ Miscellaneous Tasks

-  [**fmt**] Rust fmt
-  [**diet**] Ran cargo diet to save space
-  [**cliff**] Enhance changelog appearance </br> └─ This adds emojis as well a more content to the changelog messages
-  [**ci**] Updated GitHub action to avoid integration error </br> └─ See [this issue](https://github.com/actions-rs/clippy-check/issues/2) for example
-  [**version**] Update version & changelog

## [0.0.15] - 2022-06-01

### 🗃️ Miscellaneous Tasks

-  [**deps**] Reduce necessary dependencies </br> └─ Reduce the amount of features loaded from `tokio` and only require this feature to be activated for bin and tests.
-  [**version**] Update version & changelog

### 🩹 Bug Fixes

-  [**lib**] Avoid panicking on string slices </br> └─ This fixes the problem that occurred when the string was not made of purely utf-8 chars. E.g.: `ltrs check --text "Un essai de texte en français à controler"`

## [0.0.14] - 2022-05-31

### 🗃️ Miscellaneous Tasks

-  [**readme**] Fix file link in readme
-  [**version**] Update version & changelog

## [0.0.13] - 2022-05-31

### 🆕 Features

-  [**cli**] Annotate text from check response

### 🗃️ Miscellaneous Tasks

-  [**readme**] Document new feature
-  [**readme**] Fix typo
-  [**future**] Add some todos
-  [**cli**] Derive display order
-  [**version**] Update version & changelog

## [0.0.12] - 2022-05-31

### 🆕 Features

-  [**src**] Create new "unstable" feature
-  [**lib**] Make response fields public and add "unstable" attribute

### 🗃️ Miscellaneous Tasks

-  [**check**] Renaming `with_data` to `with_data_str`
-  [**readme**] Refactor readme based on clap's readme
-  [**version**] Update version & changelog

### 🩹 Bug Fixes

-  [**readme**] Fix changelog link in readme

## [0.0.11] - 2022-05-24

### 🗃️ Miscellaneous Tasks

-  [**doc**] Refactor readme and include it in lib.rs' doc
-  [**naming**] Renamed server variables to client
-  [**version**] Update version & changelog

### 🩹 Bug Fixes

-  [**doc**] Automatic links in docstrings

## [0.0.10] - 2022-05-23

### 🆕 Features

-  [**lib**] Add (basic) support for add. languages

### 🗃️ Miscellaneous Tasks

-  [**clean**] Remove useless comments in code
-  [**doc**] Document config file struct
-  [**version**] Update version & changelog

### 🩹 Bug Fixes

-  [**lib**] Now correctly writes arrays in config file
-  [**test**] Fixing doctest

## [0.0.9] - 2022-05-20

### 🗃️ Miscellaneous Tasks

-  [**doc**] Auto-doc for "feature"-only commands
-  [**version**] Update version & changelog

## [0.0.9-beta] - 2022-05-20

### 🗃️ Miscellaneous Tasks

-  [**version**] Update version & changelog

### 🩹 Bug Fixes

-  [**CI**] Now publish with cli feature

## [0.0.9-alpha] - 2022-05-20

### 🗃️ Miscellaneous Tasks

-  [**tests**] Add some doctests
-  [**features**] Remove default features
-  [**version**] Update version & changelog

## [0.0.8] - 2022-05-20

### 🆕 Features

-  [**lib**] Derive serialize for words' responses
-  [**lib**] New errors system </br> └─ Errors and results are now unified under a common type that can be found inside `lib/error.rs`

### 🗃️ Miscellaneous Tasks

-  [**lib**] Refactor lib. exposed functions
-  [**doc**] Update description
-  [**deps**] Remove unused crate and add thiserror crate
-  [**version**] Update version & changelog

### 🩹 Bug Fixes

-  [**cli**] Text takes now 't' as short flag
-  [**lib**] IDs and categories are String

## [0.0.7] - 2022-05-18

### 🆕 Features

-  [**errors**] Response errors are now used </br> └─ But not the body of the error, only the url
-  [**cli**] Changed language to `auto` </br> └─ language flag was required, now it defaults to `auto`

### 🗃️ Miscellaneous Tasks

-  [**errors**] Now exits as usage error for two commands
-  [**lint**] Rustfmt
-  [**typo**] Fixed typo in docstring
-  [**docs**] Split docs for `-h` and `--help`
-  [**readme**] Update todos & usage
-  [**version**] Update version & changelog

## [0.0.6] - 2022-05-18

### 🆕 Features

-  [**cli**] Pretty print json output

### 🗃️ Miscellaneous Tasks

-  [**doc**] Improve docs
-  [**refactor**] Server mod and tests
-  [**clippy**] Removed useless ref.
-  [**defaults**] Change default hostname api </br> └─ Now uses online server from LanguageTool
-  [**version**] Update version & changelog

## [0.0.5] - 2022-05-17

### 🆕 Features

-  [**cli**] Add ping command

### 🗃️ Miscellaneous Tasks

-  [**version**] Update version & changelog

## [0.0.4] - 2022-05-17

### 🆕 Features

-  [**data**] Now correctly uses `data` field + pub fields </br> └─ `data` field is not correctly serialized for the `check` request

### 🗃️ Miscellaneous Tasks

-  [**readme**] Update todos
-  [**version**] Update version & changelog

## [0.0.3] - 2022-05-17

### 🆕 Features

-  [**cli**] Create a nice cli

### 🗃️ Miscellaneous Tasks

-  [**readme**] Add new todos
-  [**version**] Update version & changelog
-  [**version**] Update version in Cargo.lock

## [0.0.2] - 2022-05-16

### 🆕 Features

-  [**features**] Now Clap is a feature required for bin

### 🗃️ Miscellaneous Tasks

-  [**version**] Update version & changelog

### 🩹 Bug Fixes

-  [**src/lib/api**] Removed empty file

## [0.0.1] - 2022-05-16

### 🆕 Features

-  [**src**] First unstable code version

### 🗃️ Miscellaneous Tasks

-  [**ci/lint/doc**] Basic GitHub setup
-  [**changelog**] First changelog

<!-- generated by git-cliff -->
