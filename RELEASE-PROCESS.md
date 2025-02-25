# Release process

First, make sure you are logged-in https://crates.io with: `cargo login`.
If you don't have write access to **LanguageTool-Rust**' crates, you can still
perform steps 1-4, and ask a maintainer with accesses to perform step 5.

This project uses `cargo-release` to publish all packages with more ease.
Note that, by default, every command runs in *dry mode*, and you need to append `--execute`
to actually perform the action.

Here are the the following steps to install `cargo-release`:
```bash
cargo install cargo-release
```
Here are the following steps to release a new version:

1. create a branch `release-x.y.z` from the `main` branch;
2. run and commit `cargo release version <LEVEL>`;
3. run and commit `cargo release replace`;
4. push your branch and create a pull request;
5. and, once your branch was merged to `main`, run the following:
   ```bash
   cargo release publish --package languagetool-rust
   ```

And voilà!
