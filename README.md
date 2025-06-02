# LanguageTool-Rust

> **Rust bindings to connect with LanguageTool server API.**

*LanguageTool is an open source grammar style checker.
It can correct 30+ languages and is free to use, more on that on
[languagetool.org](https://languagetool.org/).
There is a public API (with a free tier),
but you can also host your own server locally.
LanguageTool-Rust helps you communicate with those servers very easily via Rust code!*

[![Crates.io](https://img.shields.io/crates/v/languagetool-rust)](https://crates.io/crates/languagetool-rust)
[![docs.rs](https://img.shields.io/docsrs/languagetool-rust)](https://docs.rs/languagetool-rust)
[![codecov](https://codecov.io/gh/jeertmans/languagetool-rust/branch/main/graph/badge.svg?token=ZDZ8YBQTPH)](https://codecov.io/gh/jeertmans/languagetool-rust)

1. [About](#about)
2. [CLI Reference](#cli-reference)
    - [Docker](#docker)
3. [API Reference](#api-reference)
    - [Feature Flags](#feature-flags)
4. [CHANGELOG](https://github.com/jeertmans/languagetool-rust/blob/main/CHANGELOG.md)
5. [Related Projects](#related-projects)
6. [Contributing](#contributing)

## About

LanguageTool-Rust (LTRS) is both an **executable and a Rust library**
that strives to provide correct and safe bindings for the LanguageTool API.

*Disclaimer: the current work relies on an approximation of the LanguageTool API.
We try to avoid breaking changes as much as possible,
but we still highly depend on the future evolutions of LanguageTool.*

## Installation

You can install the latest version with `cargo`.

```bash
cargo install languagetool-rust --features full
```

### AUR

If you are on Arch Linux, you call also install with your
[AUR helper](https://wiki.archlinux.org/title/AUR_helpers):

```bash
paru -S languagetool-rust
```

## CLI Reference

![Screenshot from CLI](https://raw.githubusercontent.com/jeertmans/languagetool-rust/main/img/screenshot.svg)

The command line interface of LTRS allows to very quickly use any LanguageTool server
to check for grammar and style errors.

The reference for the CLI can be accessed via `ltrs --help`.

By default, LTRS uses the LanguageTool public API.

### Example

```bash
> ltrs ping # to check if the server is alive
PONG! Delay: 110 ms
> ltrs languages # to list all languages
[
  {
    "name": "Arabic",
    "code": "ar",
    "longCode": "ar"
  },
  {
    "name": "Asturian",
    "code": "ast",
    "longCode": "ast-ES"
  },
  # ...
]
> ltrs check --text "Some phrase with a smal mistake"  # codespell:ignore smal
{
  "language": {
    "code": "en-US",
    "detectedLanguage": {
      "code": "en-US",
      "confidence": 0.99,
      "name": "English (US)",
      "source": "ngram"
    },
    "name": "English (US)"
  },
  "matches": [
    {
      "context": {
        "length": 4,
        "offset": 19,
        "text": "Some phrase with a smal mistake"  # codespell:ignore smal
      },
      "contextForSureMatch": 0,
      "ignoreForIncompleteSentence": false,
      "length": 4,
      "message": "Possible spelling mistake found.",
      "offset": 19,
      "replacements": [
        {
          "value": "small"
        },
        {
          "value": "seal"
        },
        # ...
      }
      # ...
    ]
  # ...
}
> ltrs --help # for more details
```

### Docker

Since LanguageTool's installation might not be straightforward,
we provide a basic Docker integration that allows to `pull`, `start`, and `stop`
LanguageTool Docker containers in a few lines:

```bash
ltrs docker pull # only once
ltrs docker start # start the LT server
ltrs --hostname http://localhost -p 8010 check -t "Some tex"
# Other commands...
ltrs docker stop # stop the LT server
```

> *Note:* Docker is a tool that facilitates running applications without worrying
  about local dependencies, platform-related issues, and so on.
  Installation guidelines can be found [here](https://www.docker.com/get-started/).
  On Linux platforms, you might need to circumvent the *sudo privilege issue* by doing
  [this](https://docs.docker.com/engine/install/linux-postinstall/).

## API Reference

If you would like to integrate LTRS within a Rust application or crate,
then we recommend reading the [documentation](https://docs.rs/languagetool-rust).

To use LanguageTool-Rust in your Rust project, add to your `Cargo.toml`:

```toml
[dependencies]
languagetool-rust = "^2.1"
```

### Example

```rust
use languagetool_rust::api::{check, server::ServerClient};
use std::borrow::Cow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ServerClient::from_env_or_default();

    let req = check::Request::default()
        .with_text("Some phrase with a smal mistake");  // # codespell:ignore smal

    println!(
        "{}",
        serde_json::to_string_pretty(&client.check(&req).await?)?
    );
    Ok(())
}
```

### Feature Flags

Below are listed the various feature flags you can enable when compiling LTRS.

#### Default Features

- **cli**: Adds command-line related methods for multiple structures.
  This feature is required to install the LTRS CLI,
  and enables the following features: **annotate**, **color**, **html**, **markdown**, **multithreaded**, **typst**.
- **native-tls**: Enables TLS functionality provided by `native-tls`.

#### Optional Features

- **annotate**: Adds method(s) to annotate results from check request.
- **cli-complete**: Adds commands to generate completion files for various shells.
  This feature also activates the **cli** feature. Enter `ltrs completions --help` to get help with installing completion files.
- **color**: Enables color outputting in the terminal. If **cli** feature is also enabled, the `--color=<WHEN>` option will be available.
- **full**: Enables all features that are mutually compatible
  (i.e., `cli-complete`, `docker`, and `undoc`).
- **html**: Enables HTML parser utilities.
- **markdown**: Enables Markdown parser utilities (and also HTML).
- **multithreaded**: Enables multithreaded requests.
- **native-tls-vendored**: Enables the `vendored` feature of `native-tls`. This or `native-tls` should be activated if you are planning to use HTTPS servers.
- **typst**: Enables Typst parser utilities.
- **undoc**: Adds more fields to JSON responses that are not present
  in the [Model | Example Value](https://languagetool.org/http-api/swagger-ui/#!/default/)
  but might be present in some cases. All added fields are stored in a hashmap as
  JSON values.

## Related Projects

Here are listed some projects that use LTRS.

- [`null-ls`](https://github.com/jose-elias-alvarez/null-ls.nvim):
  Neovim plugin with LTRS builtin ([see PR](https://github.com/jose-elias-alvarez/null-ls.nvim/pull/997))
- [`languagetool-code-comments`](https://github.com/dustinblackman/languagetool-code-comments):
  uses LTRS to check for grammar errors within code comments

*Do you use LTRS in your project? Contact me so I can add it to the list!*

## Contributing

Contributions are more than welcome!

First, check out the [CONTRIBUTING][CHANGELOG](https://github.com/jeertmans/languagetool-rust/blob/main/CONTRIBUTING.md)
guide for detailed information about how to contribute to this project.

Should you have any question, feel free to reach out via:
[Issues](https://github.com/jeertmans/languagetool-rust/issues) for bugs,
[Pull requests](https://github.com/jeertmans/languagetool-rust/pulls) for code contributions
or [Discussions](https://github.com/jeertmans/languagetool-rust/discussions) for anything else.
