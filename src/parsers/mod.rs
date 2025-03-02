//! Utilities for parsing the contents of different file types into a format
//! representation that can be parsed by the LanguageTool API.

#![cfg(feature = "html")]
pub mod html;

#[cfg(feature = "markdown")]
pub mod markdown;

#[cfg(feature = "typst")]
pub mod typst;
