//! Structures for `languages` requests and responses.

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Language information
pub struct Language {
    /// Language name, e.g., `"Ukrainian"`.
    pub name: String,
    /// Language (short) code, e.g., `"uk"`.
    pub code: String,
    /// Language long code, e.g., `"uk-UA"`.
    pub long_code: String,
}

/// LanguageTool GET languages response.
///
/// List of all supported languages.
pub type LanguagesResponse = Vec<Language>;
