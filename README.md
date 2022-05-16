# LanguageTool-Rust

[![Crates.io](https://img.shields.io/crates/v/languagetool-rust)](https://crates.io/crates/languagetool-rust)
[![docs.rs](https://img.shields.io/docsrs/languagetool-rust)](https://docs.rs/languagetool-rust)

Rust API to connect with LanguageTool servers.

#### Executable

If you wish to use the executable, you can install it with Cargo:
```
cargo install languagetool-rust
```

##### Usage

```
ltrs --help
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
- [ ] Construct a good cli
- [ ] Handle all possible responses from LT
- [ ] Document installation procedure
- [ ] Document functions
- [ ] ...

## Contributing

Contributions are more than welcome!
