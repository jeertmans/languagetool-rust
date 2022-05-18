//! Structures for `languages` requests and responses.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    name: String,
    code: String,
    long_code: String,
}

pub type LanguagesResponse = Vec<Language>;
