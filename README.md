# LanguageTool-Rust

> **Rust bindings to connect with LanguageTool server API.**

*LanguageTool is an open source grammar style checker. It can correct 20+ languages and is free to use, more on that on [languagetool.org](https://languagetool.org/). There is a public API (with a free tier), but you can also host your own server locally. LanguageTool-Rust helps you communicate with those servers very easily via Rust code!*

[![Crates.io](https://img.shields.io/crates/v/languagetool-rust)](https://crates.io/crates/languagetool-rust)
[![docs.rs](https://img.shields.io/docsrs/languagetool-rust)](https://docs.rs/languagetool-rust)

1. [About](#about)
2. [CLI Reference](#cli-reference)
3. [API Reference](#api-reference)
    - [Feature Flags](#feature-flags)
4. [CHANGELOG](CHANGELOG.md)
5. [Related Projects](#related-projects)
6. [Contributing](#contributing)
    - [Future features](#future-features)

## About

LanguageTool-Rust (LTRS) is both an executable and a Rust library that aims to provide correct and safe bindings for the LanguageTool API.

*Disclaimer: the current work relies on an approximation of the LanguageTool API. We try to avoid breaking changes as much as possible, but we still highly depend on the future evolutions of LanguageTool.*

## CLI Reference

![Screenshot from CLI](https://raw.githubusercontent.com/jeertmans/languagetool-rust/main/img/screenshot.png)

The command line interface of LTRS allows to very quickly use any LanguageTool server to check for grammar and style errors. You can install the latest version with `cargo`:

```bash
> cargo install languagetool-rust --all-features
```

The reference for the CLI can be accessed via `ltrs --help`.

By default, LTRS uses LanguageTool public API.

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
> ltrs check --text "Some phrase with a smal mistake"
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
        "text": "Some phrase with a smal mistake"
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

## API Reference

If you would like to integrate LTRS within a Rust application or crate, then we recommend reading [documentation](https://docs.rs/languagetool-rust).

To use LanguageTool-Rust in your Rust project, add to your `Cargo.toml`:

```toml
[dependencies]
languagetool_rust = "version"
```

### Example

```rust
use languagetool_rust::{check::CheckRequest, server::ServerClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ServerClient::default();

    let req = CheckRequest::default()
        .with_text("Some phrase with a smal mistake");

    println!(
        "{}",
        serde_json::to_string_pretty(&client.check(&req).await?)?
    );
    Ok(())
}
```

### Feature Flags

#### Default Features

- **cli**: Adds command-line related methods for multiple structures. This is feature is required to install the LTRS CLI.

#### Optional Features

- **annotate**: Adds method(s) to annotate results from check request. If **cli** feature is also enabled, the CLI will by default print an annotated output.
- **unstable**: Adds more fields to JSON responses that are not present in the [Model | Example Value](https://languagetool.org/http-api/swagger-ui/#!/default/) but might be present in some cases. All added fields are optional, hence the `Option` around them.

## Related Projects

Here are listed some projects that use LTRS.

- *W.I.P.*

*Do you use LTRS in your project? Contact me so I can add it to the list!*

## Contributing

Contributions are more than welcome!

### Future features

- [x] Use Cargo features to enable Clap and others only in bin.rs
- [x] Construct a good cli
- [x] Handle all possible responses from LT
- [ ] Document installation procedure
- [x] Document functions
- [ ] Test commands that need API keys
- [x] Build test for lib
- [x] Build automated testing with LT server (GitHub action?)
- [x] Parse "data" as input to check command
- [x] Parse parameters from env value with clap/env feature
- [x] Enhance annotated text
- [ ] ...
