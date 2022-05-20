# LanguageTool-Rust

[![Crates.io](https://img.shields.io/crates/v/languagetool-rust)](https://crates.io/crates/languagetool-rust)
[![docs.rs](https://img.shields.io/docsrs/languagetool-rust)](https://docs.rs/languagetool-rust)

Rust bindings to connect with LanguageTool server API.

#### Executable

If you wish to use the executable, you can install it with Cargo:
```
cargo install languagetool-rust
```

##### Usage

```
ltrs ping # to check if the server is alive
ltrs languages # to list all languages
ltrs check --text "Some phrase with a smal mistake"

ltrs --help # for more details
```

#### Library

You can use LanguageTool-Rust in your Rust project by adding to your `Cargo.toml`:
```toml
languagetool_rust = "version"
```

##### Documentation

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
