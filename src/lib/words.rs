//! Structures for `words` requests and responses.

use crate::{
    check::serialize_option_vec_string,
    error::{Error, Result},
};
#[cfg(feature = "cli")]
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Parse `v` if valid word.
///
/// A valid word is any string slice that does not contain any whitespace
///
/// # Examples
///
/// ```
/// # use languagetool_rust::words::parse_word;
/// assert!(parse_word("word").is_ok());
///
/// assert!(parse_word("some words").is_err());
/// ```
pub fn parse_word(v: &str) -> Result<String> {
    if !v.contains(' ') {
        return Ok(v.to_string());
    }
    Err(Error::InvalidValue(
        "The value should be a word that does not contain any whitespace".to_string(),
    ))
}

/// Login arguments required by the API.
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LoginArgs {
    /// Your username as used to log in at languagetool.org.
    #[cfg_attr(
        feature = "cli",
        clap(short = 'u', long, required = true, env = "LANGUAGETOOL_USERNAME")
    )]
    pub username: String,
    /// [Your API key](https://languagetool.org/editor/settings/api).
    #[cfg_attr(
        feature = "cli",
        clap(short = 'k', long, required = true, env = "LANGUAGETOOL_API_KEY")
    )]
    pub api_key: String,
}

/// LanguageTool GET words request.
///
/// List words in the user's personal dictionaries.
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WordsRequest {
    /// Offset of where to start in the list of words.
    #[cfg_attr(feature = "cli", clap(long, default_value = "0"))]
    offset: isize,
    /// Maximum number of words to return.
    #[cfg_attr(feature = "cli", clap(long, default_value = "10"))]
    pub limit: isize,
    /// Login arguments.
    #[cfg_attr(feature = "cli", clap(flatten))]
    #[serde(flatten)]
    pub login: LoginArgs,
    /// Comma-separated list of dictionaries to include words from; uses special
    /// default dictionary if this is unset.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub dicts: Option<Vec<String>>,
}

/// Copy of [`WordsRequest`], but used to CLI only.
///
/// This is a temporary solution, until [#3165](https://github.com/clap-rs/clap/issues/3165) is
/// closed.
#[cfg(feature = "cli")]
#[derive(Args, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WordsRequestArgs {
    /// Offset of where to start in the list of words.
    #[cfg_attr(feature = "cli", clap(long, default_value = "0"))]
    offset: isize,
    /// Maximum number of words to return.
    #[cfg_attr(feature = "cli", clap(long, default_value = "10"))]
    pub limit: isize,
    /// Login arguments.
    #[cfg_attr(feature = "cli", clap(flatten))]
    #[serde(flatten)]
    pub login: Option<LoginArgs>,
    /// Comma-separated list of dictionaries to include words from; uses special
    /// default dictionary if this is unset.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub dicts: Option<Vec<String>>,
}

#[cfg(feature = "cli")]
impl From<WordsRequestArgs> for WordsRequest {
    #[inline]
    fn from(args: WordsRequestArgs) -> Self {
        Self {
            offset: args.offset,
            limit: args.limit,
            login: args.login.unwrap(),
            dicts: args.dicts,
        }
    }
}

/// LanguageTool POST words add request.
///
/// Add a word to one of the user's personal dictionaries. Please note that this
/// feature is considered to be used for personal dictionaries which must not
/// contain more than 500 words. If this is an issue for you, please contact us.
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WordsAddRequest {
    /// The word to be added. Must not be a phrase, i.e., cannot contain white
    /// space. The word is added to a global dictionary that applies to all
    /// languages.
    #[cfg_attr(feature = "cli", clap(required = true, value_parser = parse_word))]
    pub word: String,
    /// Login arguments.
    #[cfg_attr(feature = "cli", clap(flatten))]
    #[serde(flatten)]
    pub login: LoginArgs,
    /// Name of the dictionary to add the word to; non-existent dictionaries are
    /// created after calling this; if unset, adds to special default
    /// dictionary.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dict: Option<String>,
}

/// LanguageTool POST words delete request.
///
/// Remove a word from one of the user's personal dictionaries.
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WordsDeleteRequest {
    /// The word to be removed.
    #[cfg_attr(feature = "cli", clap(required = true, value_parser = parse_word))]
    pub word: String,
    /// Login arguments.
    #[cfg_attr(feature = "cli", clap(flatten))]
    #[serde(flatten)]
    pub login: LoginArgs,
    /// Name of the dictionary to add the word to; non-existent dictionaries are
    /// created after calling this; if unset, adds to special default
    /// dictionary.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dict: Option<String>,
}

/// Words' optional subcommand.
#[cfg(feature = "cli")]
#[derive(Clone, Debug, Subcommand)]
pub enum WordsSubcommand {
    /// Add a word to some user's list.
    Add(WordsAddRequest),
    /// Remove a word from some user's list.
    Delete(WordsDeleteRequest),
}

/// Retrieve some user's words list.
#[cfg(feature = "cli")]
#[derive(Debug, Parser)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(subcommand_negates_reqs = true)]
pub struct WordsCommand {
    /// Actual GET request.
    #[command(flatten)]
    pub request: WordsRequestArgs,
    /// Optional subcommand.
    #[command(subcommand)]
    pub subcommand: Option<WordsSubcommand>,
}

/// LanguageTool GET words response.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WordsResponse {
    /// List of words.
    pub words: Vec<String>,
}

/// LanguageTool POST word add response.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WordsAddResponse {
    /// `true` if word was correctly added.
    pub added: bool,
}

/// LanguageTool POST word delete response.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WordsDeleteResponse {
    /// `true` if word was correctly removed.
    pub deleted: bool,
}
