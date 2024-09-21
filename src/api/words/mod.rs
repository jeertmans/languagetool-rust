//! Structures for `words` requests and responses.

use crate::error::{Error, Result};

use super::check::serialize_option_vec_string;
#[cfg(feature = "cli")]
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

pub mod add;
pub mod delete;

/// Parse `v` if valid word.
///
/// A valid word is any string slice that does not contain any whitespace
///
/// # Examples
///
/// ```
/// # use languagetool_rust::api::words::parse_word;
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Hash)]
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[non_exhaustive]
pub struct Request {
    /// Offset of where to start in the list of words.
    ///
    /// Defaults to 0.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<isize>,
    /// Maximum number of words to return.
    ///
    /// Defaults to 10.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<isize>,
    /// Login arguments.
    #[cfg_attr(feature = "cli", clap(flatten))]
    #[serde(flatten)]
    pub login: LoginArgs,
    /// Comma-separated list of dictionaries to include words from; uses special
    /// default dictionary if this is unset.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dicts: Option<Vec<String>>,
}

/// Copy of [`Request`], but used to CLI only.
///
/// This is a temporary solution, until [#3165](https://github.com/clap-rs/clap/issues/3165) is
/// closed.
#[cfg(feature = "cli")]
#[derive(Args, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RequestArgs {
    /// Offset of where to start in the list of words.
    #[cfg_attr(feature = "cli", clap(long, default_value = "0"))]
    pub offset: isize,
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
impl From<RequestArgs> for Request {
    #[inline]
    fn from(args: RequestArgs) -> Self {
        Self {
            offset: Some(args.offset),
            limit: Some(args.limit),
            login: args.login.unwrap(),
            dicts: args.dicts,
        }
    }
}

/// Words' optional subcommand.
#[cfg(feature = "cli")]
#[derive(Clone, Debug, Subcommand)]
pub enum WordsSubcommand {
    /// Add a word to some user's list.
    Add(add::Request),
    /// Remove a word from some user's list.
    Delete(delete::Request),
}

/// Retrieve some user's words list.
#[cfg(feature = "cli")]
#[derive(Debug, Parser)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(subcommand_negates_reqs = true)]
pub struct WordsCommand {
    /// Actual GET request.
    #[command(flatten)]
    pub request: RequestArgs,
    /// Optional subcommand.
    #[command(subcommand)]
    pub subcommand: Option<WordsSubcommand>,
}

/// LanguageTool GET words response.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Response {
    /// List of words.
    pub words: Vec<String>,
}
