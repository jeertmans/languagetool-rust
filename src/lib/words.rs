use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct WordsResponse {
    words: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct WordsAddResponse {
    added: bool,
}

#[derive(Debug, Deserialize)]
pub struct WordsDeleteResponse {
    deleted: bool,
}
