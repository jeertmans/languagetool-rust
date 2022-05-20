//! Structures for `words` requests and responses.

#[cfg(feature = "cli")]
use clap::Parser;
use serde::{Deserialize, Serialize};

pub fn is_word(v: &str) -> Result<(), String> {
    if !v.contains(' ') {
        return Ok(());
    }
    Err(String::from(
        "The value should be a word that does not contain any whitespace",
    ))
}

#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginArgs {
    #[cfg_attr(feature = "cli", clap(short = 'u', long, required = true))]
    /// Your username as used to log in at languagetool.org.
    username: String,
    #[cfg_attr(feature = "cli", clap(short = 'k', long, required = true))]
    /// [Your API key](https://languagetool.org/editor/settings/api)
    api_key: String,
}

#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Debug, Default, Serialize)]
/// LanguageTool GET words request.
///
/// List words in the user's personal dictionaries.
pub struct WordsRequest {
    #[cfg_attr(feature = "cli", clap(long, default_value = "0"))]
    /// Offset of where to start in the list of words.
    offset: isize,
    #[cfg_attr(feature = "cli", clap(long, default_value = "10"))]
    /// Maximum number of words to return.
    limit: isize,
    #[cfg_attr(feature = "cli", clap(flatten))]
    login: LoginArgs,
    #[cfg_attr(feature = "cli", clap(long))]
    /// Comma-separated list of dictionaries to include words from; uses special default dictionary if this is unset
    dicts: Option<Vec<String>>,
}

#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Debug, Default, Serialize)]
/// LanguageTool POST words add request.
///
/// Add a word to one of the user's personal dictionaries. Please note that this feature is considered to be used for personal dictionaries which must not contain more than 500 words. If this is an issue for you, please contact us.
pub struct WordsAddRequest {
    #[cfg_attr(feature = "cli", clap(required = true, validator = is_word))]
    /// The word to be added. Must not be a phrase, i.e. cannot contain white space. The word is added to a global dictionary that applies to all languages.
    word: String,
    #[cfg_attr(feature = "cli", clap(flatten))]
    login: LoginArgs,
    #[cfg_attr(feature = "cli", clap(long))]
    /// Name of the dictionary to add the word to; non-existent dictionaries are created after
    /// calling this; if unset, adds to special default dictionary
    dict: Option<String>,
}

#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Debug, Default, Serialize)]
/// LanguageTool POST words delete request.
///
/// Remove a word from one of the user's personal dictionaries.
pub struct WordsDeleteRequest {
    #[cfg_attr(feature = "cli", clap(required = true, validator = is_word))]
    /// The word to be removed.
    word: String,
    #[cfg_attr(feature = "cli", clap(flatten))]
    login: LoginArgs,
    #[cfg_attr(feature = "cli", clap(long))]
    /// Name of the dictionary to add the word to; non-existent dictionaries are created after
    /// calling this; if unset, adds to special default dictionary
    dict: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WordsResponse {
    words: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WordsAddResponse {
    added: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WordsDeleteResponse {
    deleted: bool,
}
