# LanguageTool-Rust

[![Crates.io](https://img.shields.io/crates/v/languagetool-rust)](https://crates.io/crates/languagetool-rust)
[![docs.rs](https://img.shields.io/docsrs/languagetool-rust)](https://docs.rs/languagetool-rust)

Rust bindings to connect with LanguageTool server API.

## Usage

LanguageTool-Rust (LTRS) can be both used as an executable or a Rust library.

#### Executable

By default, LTRS uses LanguageTool public API.

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

#### Rust Library

```rust
use languagetool_rust::{check::CheckRequest, server::ServerClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ServerClient::default();

    let req = CheckRequest::default()
        .with_language("auto")
        .with_text("Some phrase with a smal mistake");

    println!(
        "{}",
        serde_json::to_string_pretty(&client.check(&req).await?)?
    );
    Ok(())
}
```

## Installation

If you wish to use the executable, you can install it with Cargo:
```bash
cargo install languagetool-rust
```

You can use LanguageTool-Rust in your Rust project by adding to your `Cargo.toml`:
```toml
languagetool_rust = "version"
```

## Documentation

Automatically generated documentation can found found [here](https://docs.rs/languagetool-rust). Many functions are missing docs, but it's on the TODO list!

### Disclaimers

This software is not production-ready. Many changes are expected before I consider it to be *usable*.

### TODO List

- [x] Use Cargo features to enable Clap and others only in bin.rs
- [x] Construct a good cli
- [x] Handle all possible responses from LT
- [ ] Document installation procedure
- [ ] Document functions
- [ ] Test commands that need API keys
- [ ] Build test for lib
- [ ] Build automated testing with LT server (GitHub action?)
- [x] Parse "data" as input to check command
- [ ] ...

## Contributing

Contributions are more than welcome!
