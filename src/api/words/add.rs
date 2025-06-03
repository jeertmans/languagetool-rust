//! Structures for `words` requests and responses related to adding.

use super::*;

/// LanguageTool POST words add request.
///
/// Add a word to one of the user's personal dictionaries. Please note that
/// this feature is considered to be used for personal dictionaries
/// which must not contain more than 500 words. If this is an issue for
/// you, please contact us.
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[non_exhaustive]
pub struct Request {
    /// The word to be added. Must not be a phrase, i.e., cannot contain
    /// white space. The word is added to a global dictionary that
    /// applies to all languages.
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

/// LanguageTool POST word add response.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Response {
    /// `true` if word was correctly added.
    pub added: bool,
}
