//#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown, clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../../README.md")]
//!
//! # Note
//!
//! Most structures in this library are marked with
//! ```ignore
//! #[non_exhaustive]
//! ```
//! to indicate that they are likely to change in the future.
//!
//! This is a consequence of using an external API (i.e., the LanguageTool API)
//! that cannot be controlled and (possible) breaking changes are to be expected.

pub mod check;
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "docker")]
pub mod docker;
pub mod error;
pub mod languages;
pub mod server;
pub mod words;

pub use crate::check::{CheckRequest, CheckResponse};
#[cfg(feature = "docker")]
pub use crate::docker::Docker;
pub use crate::languages::LanguagesResponse;
pub use crate::server::ServerClient;
pub use crate::words::{
    WordsAddRequest, WordsAddResponse, WordsDeleteRequest, WordsDeleteResponse, WordsRequest,
    WordsResponse,
};
