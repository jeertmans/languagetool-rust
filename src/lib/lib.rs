#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../../README.md")]

pub mod check;
pub mod error;
pub mod languages;
pub mod server;
pub mod words;

pub use crate::check::{CheckRequest, CheckResponse};
pub use crate::languages::LanguagesResponse;
pub use crate::server::ServerClient;
pub use crate::words::{
    WordsAddRequest, WordsAddResponse, WordsDeleteRequest, WordsDeleteResponse, WordsRequest,
    WordsResponse,
};
