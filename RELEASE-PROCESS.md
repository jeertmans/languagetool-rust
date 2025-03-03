# Release process

If you don't have write access to **LanguageTool-Rust**' crates, you can still
perform steps 1-3, and ask a maintainer with accesses to perform step 4

This project uses `cargo-release` to bump the version number and update the change log with more ease.

Note that, by default, every command runs in *dry mode*, and you need to append `--execute`
to actually perform the action.

Here are the the following steps to install `cargo-release`:
```bash
cargo install cargo-release
```
Here are the following steps to release a new version:

1. create a branch `release-x.y.z` from the main branch;
2. run `cargo release <version|LEVEL>`;
3. create a pull request;
4. and, once your branch was merged to `main` tag it `vx.y.z` and push it (we prefer to create tags through GitHub releases)

And voilà!
