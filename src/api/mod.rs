//! Raw bindings to the LanguageTool API v1.1.2.
//!
//! The current bindings were generated using the
//! [HTTP API documentation](https://languagetool.org/http-api/).
//!
//! Unfortunately, the LanguageTool API is not as documented as we could
//! hope, and requests might return undocumented fields. Those are de-serialized
//! to the `undocumented` field.
pub mod check;
pub mod languages;
pub mod server;
pub mod words;
