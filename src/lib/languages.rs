//! Structures for `languages` requests and responses.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    name: String,
    code: String,
    long_code: String,
}

pub type LanguagesResponse = Vec<Language>;
