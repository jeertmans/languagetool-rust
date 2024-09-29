//! Structures for `words` requests and responses related to deleting.

use super::*;

/// LanguageTool POST words delete request.
///
/// Remove a word from one of the user's personal dictionaries.
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[non_exhaustive]
pub struct Request {
    /// The word to be removed.
    #[cfg_attr(feature = "cli", clap(required = true, value_parser = parse_word))]
    pub word: String,
    /// Login arguments.
    #[cfg_attr(feature = "cli", clap(flatten))]
    #[serde(flatten)]
    pub login: LoginArgs,
    /// Name of the dictionary to add the word to; non-existent dictionaries
    /// are created after calling this; if unset, adds to special
    /// default dictionary.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dict: Option<String>,
}

/// LanguageTool POST word delete response.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Response {
    /// `true` if word was correctly removed.
    pub deleted: bool,
}
