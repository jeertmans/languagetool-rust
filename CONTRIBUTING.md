# Contributing to `languagetool-rust`

Thanks for your interest in contributing to `languagetool-rust`! This project aims to provide both (1) a fast, idiomatic Rust client for [LanguageTool](https://languagetool.org/), supporting both HTTP and local servers, and (2) a convenient command-line interface to check your files for grammar mistakes.

We welcome contributions of all kinds: bug fixes, documentation improvements, feature additions, or performance enhancements.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Guide](#development-guide)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Requests](#pull-requests)

---

## Getting Started

1. **Fork the repository** and clone it locally:

   ```bash
   git clone https://github.com/your-username/languagetool-rust.git
   cd languagetool-rust
   ```

2. [**Install Rust (if you haven't already)**](https://www.rust-lang.org/learn/get-started) as well as `rustfmt` and `clippy`.

   This project also requires the *nightly* channel for formatting the code. You can install it with:

   ```bash
   rustup toolchain install nightly
   ```

3. **Build the project:**

   ```bash
   cargo build
   ```

4. **Run the CLI to ensure everything works:**

   ```bash
   cargo run -- check --text "This text contans one mistake."  # codespell:ignore contans
   ```

## Development Guide

This project is organized in two parts:

- The API library, in `src/api/`, with the bindings to connect to the LanguageTool API;
- The command-line interface (CLI), in `src/cli/`, to provide an easy-to-use tool for checking your files.

Tests are located either in the Rust modules as unit tests, or in the `tests/` folder for integration tests.

## Testing

To avoid spamming the free LanguageTool API, running the tests required you to specify the LanguageTool server URL and PORT, via environment variables `LANGUAGETOOL_HOSTNAME` and `LANGUAGETOOL_PORT`. We also recommend that you [run a local server](https://dev.languagetool.org/http-server) on your machine.

If you have [Docker](https://www.docker.com/) installed on your machine, we provide you with a `docker-compose.yml` file that allows you to set up a local server for testing (or actual grammar checking) with `docker compose up`.

Then, you can run the test suite with:

```bash
cargo test
```

> [!IMPORTANT]
> Please write tests for any new features or bug fixes you introduce.

## Documentation

Writing good documentation is as important as writing good code.

Make sure that any addition or modification to the code results in an appropriate change in the documentation. We encourage contributors to take inspiration from existing documentation.

## Pull Requests

Before submitting your code, please run the following commands to clean and lint your code:

```bash
cargo +nightly fmt
cargo clippy
```

If any issue is raised by Clippy, please try to address it, or ask us for help / guidance if needed.

Once your contribution is ready, you can create a pull request, and we will take time to review it!

---

Thanks for making `languagetool-rust` better!
