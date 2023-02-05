//! Structures for `check` requests and responses.

use super::error::{Error, Result};
#[cfg(feature = "cli")]
use clap::{Args, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Requests

/// Parse `v` is valid language code.
///
/// A valid language code is usually
/// - a two character string matching pattern
///   `[a-z]{2}
/// - a five character string matching pattern
///   `[a-z]{2}-[A-Z]{2}
/// - or some more complex ascii string (see below)
///
/// Language code is case insensitive.
///
/// Therefore, a valid language code must match the following:
///
/// - `[a-zA-Z]{2,3}(-[a-zA-Z]{2}(-[a-zA-Z]+)*)?`
///
/// or
///
/// - "auto"
///
/// > Note: a valid language code does not mean that it exists.
///
/// # Examples
///
/// ```
/// # use languagetool_rust::check::parse_language_code;
/// assert!(parse_language_code("en").is_ok());
///
/// assert!(parse_language_code("en-US").is_ok());
///
/// assert!(parse_language_code("en-us").is_ok());
///
/// assert!(parse_language_code("ca-ES-valencia").is_ok());
///
/// assert!(parse_language_code("abcd").is_err());
///
/// assert!(parse_language_code("en_US").is_err());
///
/// assert!(parse_language_code("fr-french").is_err());
///
/// assert!(parse_language_code("some random text").is_err());
/// ```
#[cfg(feature = "cli")]
pub fn parse_language_code(v: &str) -> Result<String> {
    #[inline]
    fn is_match(v: &str) -> bool {
        let mut splits = v.split('-');

        match splits.next() {
            Some(s)
                if (s.len() == 2 || s.len() == 3) && s.chars().all(|c| c.is_ascii_alphabetic()) => {
            }
            _ => return false,
        }

        match splits.next() {
            Some(s) if s.len() != 2 || s.chars().any(|c| !c.is_ascii_alphabetic()) => return false,
            Some(_) => (),
            None => return true,
        }
        for s in splits {
            if !s.chars().all(|c| c.is_ascii_alphabetic()) {
                return false;
            }
        }
        true
    }

    if v == "auto" || is_match(v) {
        Ok(v.to_string())
    } else {
        Err(Error::InvalidValue("The value should be `auto` or match regex pattern: ^[a-zA-Z]{2,3}(-[a-zA-Z]{2}(-[a-zA-Z]+)*)?$".to_string()))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
/// A portion of text to be checked.
pub struct DataAnnotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If set, the markup will be interpreted as this
    pub interpret_as: Option<String>,
    /// Text that should be treated as markup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markup: Option<String>,
    /// Text that should be treated as normal text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl Default for DataAnnotation {
    fn default() -> Self {
        Self {
            interpret_as: None,
            markup: None,
            text: Some(String::new()),
        }
    }
}

impl DataAnnotation {
    #[inline]
    /// Instantiate a new `DataAnnotation` with text only
    #[must_use]
    pub fn new_text(text: String) -> Self {
        Self {
            interpret_as: None,
            markup: None,
            text: Some(text),
        }
    }

    #[inline]
    /// Instantiate a new `DataAnnotation` with markup only
    #[must_use]
    pub fn new_markup(markup: String) -> Self {
        Self {
            interpret_as: None,
            markup: Some(markup),
            text: None,
        }
    }

    #[inline]
    /// Instantiate a new `DataAnnotation` with markup and its interpretation
    #[must_use]
    pub fn new_interpreted_markup(markup: String, interpret_as: String) -> Self {
        Self {
            interpret_as: Some(interpret_as),
            markup: Some(markup),
            text: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
/// Alternative text to be checked.
pub struct Data {
    /// Vector of markup text, see [DataAnnotation]
    pub annotation: Vec<DataAnnotation>,
}

impl<T: Into<DataAnnotation>> FromIterator<T> for Data {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let annotation = iter.into_iter().map(std::convert::Into::into).collect();
        Data { annotation }
    }
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = std::collections::HashMap::new();
        map.insert("annotation", &self.annotation);

        serializer.serialize_str(&serde_json::to_string(&map).unwrap())
    }
}

#[cfg(feature = "cli")]
impl std::str::FromStr for Data {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let v: Self = serde_json::from_str(s)?;
        Ok(v)
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
/// Possible levels for additional rules.
///
/// Currently, `Level::Picky` adds additional rules
/// with respect to `Level::Default`
pub enum Level {
    /// Default level
    Default,
    /// Picky level
    Picky,
}

impl Default for Level {
    fn default() -> Self {
        Level::Default
    }
}

impl Level {
    /// Return `true` if current level is the default one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use languagetool_rust::check::Level;
    ///
    /// let level: Level = Default::default();
    ///
    /// assert!(level.is_default());
    /// ```
    #[must_use]
    pub fn is_default(&self) -> bool {
        *self == Level::default()
    }
}

/// Split string into as few fragments as possible, where each fragment contains
/// (if possible) a maximum of `n` characters. Pattern str `Pat` is used for splitting.
///
/// # Examples
///
/// ```
/// # use languagetool_rust::check::split_len;
/// let s = "I have so many friends.
/// They are very funny.
/// I think I am very lucky to have them.
/// One day, I will write them a poem.
/// But, in the meantime, I write code.
/// ";
///
/// let split = split_len(&s, 40, "\n");
///
/// assert_eq!(split.join(""), s);
/// assert_eq!(split, vec![
///     "I have so many friends.\n",
///     "They are very funny.\n",
///     "I think I am very lucky to have them.\n",
///     "One day, I will write them a poem.\n",
///     "But, in the meantime, I write code.\n"]);
///
/// let split = split_len(&s, 80, "\n");
///
/// assert_eq!(split, vec![
///     "I have so many friends.\nThey are very funny.\n",
///     "I think I am very lucky to have them.\nOne day, I will write them a poem.\n",
///     "But, in the meantime, I write code.\n"]);
///
/// let s = "I have so many friends.
/// They are very funny.
/// I think I am very lucky to have them.
///
/// One day, I will write them a poem.
/// But, in the meantime, I write code.
/// ";
///
/// let split = split_len(&s, 80, "\n\n");
///
/// println!("{:?}", split);
///
/// assert_eq!(split, vec![
///     "I have so many friends.\nThey are very funny.\nI think I am very lucky to have them.\n\n",
///     "One day, I will write them a poem.\nBut, in the meantime, I write code.\n"]);
/// ```
#[must_use]
pub fn split_len<'source>(s: &'source str, n: usize, pat: &str) -> Vec<&'source str> {
    let mut vec: Vec<&'source str> = Vec::with_capacity(s.len() / n);
    let mut splits = s.split_inclusive(pat);

    let mut start = 0;
    let mut i = 0;

    if let Some(split) = splits.next() {
        vec.push(split);
    } else {
        return Vec::new();
    }

    for split in splits {
        let new_len = vec[i].len() + split.len();
        if new_len < n {
            vec[i] = &s[start..start + new_len];
        } else {
            vec.push(split);
            start += vec[i].len();
            i += 1;
        }
    }

    vec
}

#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// LanguageTool POST check request.
///
/// The main feature - check a text with LanguageTool for possible style and grammar issues.
///
/// The structure below tries to follow as closely as possible the JSON API described
/// [here](https://languagetool.org/http-api/swagger-ui/#!/default/post_check).
pub struct CheckRequest {
    #[cfg(all(feature = "cli", feature = "annotate"))]
    #[clap(short = 'r', long)]
    #[serde(skip_serializing)]
    /// If present, raw JSON output will be printed instead of annotated text.
    pub raw: bool,
    #[cfg(feature = "cli")]
    #[clap(short = 'm', long)]
    #[serde(skip_serializing)]
    /// If present, more context (i.e., line number and line offset) will be added to response.
    pub more_context: bool,
    #[cfg_attr(feature = "cli", clap(short = 't', long, conflicts_with = "data",))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The text to be checked. This or 'data' is required.
    pub text: Option<String>,
    #[cfg_attr(feature = "cli", clap(short = 'd', long, conflicts_with = "text"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The text to be checked, given as a JSON document that specifies what's text and what's markup. This or 'text' is required.
    ///
    /// Markup will be ignored when looking for errors. Example text:
    /// ```html
    /// A <b>test</b>
    /// ```
    /// JSON for the example text:
    /// ```json
    /// {"annotation":[
    ///  {"text": "A "},
    ///  {"markup": "<b>"},
    ///  {"text": "test"},
    ///  {"markup": "</b>"}
    /// ]}
    /// ```
    /// If you have markup that should be interpreted as whitespace, like `<p>` in HTML, you can have it interpreted like this:
    ///
    /// ```json
    /// {"markup": "<p>", "interpretAs": "\n\n"}
    /// ```
    /// The 'data' feature is not limited to HTML or XML, it can be used for any kind of markup. Entities will need to be expanded in this input.
    pub data: Option<Data>,
    #[cfg_attr(
        all(feature = "cli", feature = "cli", feature = "cli"),
        clap(
            short = 'l',
            long,
            default_value = "auto",
            value_parser = parse_language_code
        )
    )]
    #[cfg_attr(
        all(feature = "cli", not(all(feature = "cli", feature = "cli"))),
        clap(short = 'l', long, default_value = "auto",)
    )]
    /// A language code like `en-US`, `de-DE`, `fr`, or `auto` to guess the language automatically (see `preferredVariants` below).
    ///
    /// For languages with variants (English, German, Portuguese) spell checking will only be activated when you specify the variant, e.g. `en-GB` instead of just `en`.
    pub language: String,
    #[cfg_attr(feature = "cli", clap(short = 'u', long, requires = "api_key"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Set to get Premium API access: Your username/email as used to log in at languagetool.org.
    pub username: Option<String>,
    #[cfg_attr(feature = "cli", clap(short = 'k', long, requires = "username"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Set to get Premium API access: [your API key](https://languagetool.org/editor/settings/api)
    pub api_key: Option<String>,
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Comma-separated list of dictionaries to include words from; uses special default dictionary if this is unset
    pub dicts: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// A language code of the user's native language, enabling false friends checks for some language pairs.
    pub mother_tongue: Option<String>,
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Comma-separated list of preferred language variants.
    ///
    /// The language detector used with `language=auto` can detect e.g. English, but it cannot decide whether British English or American English is used. Thus this parameter can be used to specify the preferred variants like `en-GB` and `de-AT`. Only available with `language=auto`. You should set variants for at least German and English, as otherwise the spell checking will not work for those, as no spelling dictionary can be selected for just `en` or `de`.
    pub preferred_variants: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// IDs of rules to be enabled, comma-separated
    pub enabled_rules: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// IDs of rules to be disabled, comma-separated
    pub disabled_rules: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// IDs of categories to be enabled, comma-separated
    pub enabled_categories: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// IDs of categories to be disabled, comma-separated
    pub disabled_categories: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "is_false")]
    /// If true, only the rules and categories whose IDs are specified with `enabledRules` or `enabledCategories` are enabled.
    pub enabled_only: bool,
    #[cfg_attr(
        feature = "cli",
        clap(long, default_value = "default", ignore_case = true, value_enum)
    )]
    #[serde(skip_serializing_if = "Level::is_default")]
    /// If set to `picky`, additional rules will be activated, i.e. rules that you might only find useful when checking formal text.
    pub level: Level,
}

impl Default for CheckRequest {
    #[inline]
    fn default() -> CheckRequest {
        CheckRequest {
            #[cfg(all(feature = "cli", feature = "annotate"))]
            raw: Default::default(),
            #[cfg(feature = "cli")]
            more_context: Default::default(),
            text: Default::default(),
            data: Default::default(),
            language: "auto".to_string(),
            username: Default::default(),
            api_key: Default::default(),
            dicts: Default::default(),
            mother_tongue: Default::default(),
            preferred_variants: Default::default(),
            enabled_rules: Default::default(),
            disabled_rules: Default::default(),
            enabled_categories: Default::default(),
            disabled_categories: Default::default(),
            enabled_only: Default::default(),
            level: Default::default(),
        }
    }
}

#[inline]
fn is_false(b: &bool) -> bool {
    !(*b)
}

impl CheckRequest {
    /// Set the text to be checked and removed potential data field
    #[must_use]
    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self.data = None;
        self
    }

    /// Set the data to be checked and removed potential text field
    #[must_use]
    pub fn with_data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self.text = None;
        self
    }

    /// Set the data (obtained from string) to be checked and removed potential text field
    pub fn with_data_str(self, data: &str) -> serde_json::Result<Self> {
        Ok(self.with_data(serde_json::from_str(data)?))
    }

    /// Set the language of the text / data
    #[must_use]
    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    /// Return a copy of the text within the request.
    ///
    /// # Panics
    ///
    /// Panics if both self.text and self.data are None.
    /// Panics if any data annotation does not contain text or markup.
    #[must_use]
    pub fn get_text(&self) -> String {
        if let Some(ref text) = self.text {
            text.clone()
        } else if let Some(ref data) = self.data {
            let mut text = String::new();
            for da in data.annotation.iter() {
                if let Some(ref t) = da.text {
                    text.push_str(t.as_str());
                } else if let Some(ref t) = da.markup {
                    text.push_str(t.as_str());
                } else {
                    panic!("request contains some invalid data annotations(s): {da:?}");
                }
            }
            text
        } else {
            panic!(
                "impossible to retrieve text from request if both data and text fields are None"
            );
        }
    }
}

#[cfg(feature = "cli")]
/// Parse a string slice into a [`PathBuf`], and error if the file does not exist.
fn parse_filename(s: &str) -> Result<PathBuf> {
    let path_buf: PathBuf = s.parse().unwrap();

    if path_buf.is_file() {
        Ok(path_buf)
    } else {
        Err(Error::InvalidFilename(s.to_string()))
    }
}

#[cfg(feature = "cli")]
#[derive(Debug, Parser)]
pub struct CheckCommand {
    #[command(flatten)]
    pub request: CheckRequest,
    #[arg(conflicts_with_all(["text", "data"]), value_parser = parse_filename)]
    /// Optional filenames from which input is read.
    pub filenames: Vec<PathBuf>,
}

/// Responses

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
/// Detected language from check request.
pub struct DetectedLanguage {
    /// Language code, e.g., `"sk-SK"` for Slovak
    pub code: String,
    #[cfg(feature = "unstable")]
    /// Confidence level, from 0 to 1
    pub confidence: Option<f64>,
    /// Language name, e.g., `"Slovak"`
    pub name: String,
    #[cfg(feature = "unstable")]
    /// Source (file) for the language detection
    pub source: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Language information in check response.
pub struct LanguageResponse {
    /// Language code, e.g., `"sk-SK"` for Slovak
    pub code: String,
    /// Detected language from provided request
    pub detected_language: DetectedLanguage,
    /// Language name, e.g., `"Slovak"`
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
/// Match context in check response.
pub struct Context {
    /// Length of the match
    pub length: usize,
    /// Char index at which the match starts
    pub offset: usize,
    /// Contextual text around the match
    pub text: String,
}

#[cfg(feature = "cli")]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
/// More context, post-processed in check response.
pub struct MoreContext {
    /// Line number where match occured
    pub line_number: usize,
    /// Char index at which the match starts on the current line
    pub line_offset: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
/// Possible replacement for a given match in check response.
pub struct Replacement {
    /// Possible replacement value
    pub value: String,
}

impl From<String> for Replacement {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<&str> for Replacement {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
/// A rule category.
pub struct Category {
    /// Category id
    pub id: String,
    /// Category name
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
/// A possible url of a rule in a check response.
pub struct Url {
    /// Url value
    pub value: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// The rule that was not satisfied in a given match.
pub struct Rule {
    /// Rule category
    pub category: Category,
    /// Rule description
    pub description: String,
    /// Rule id
    pub id: String,
    #[cfg(feature = "unstable")]
    /// Indicate if the rule is from the premium API
    pub is_premium: Option<bool>,
    /// Issue type
    pub issue_type: String,
    #[cfg(feature = "unstable")]
    /// Rule source file
    pub source_file: Option<String>,
    /// Rule sub id
    pub sub_id: Option<String>,
    /// Rule list of urls
    pub urls: Option<Vec<Url>>,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Type of a given match.
pub struct Type {
    /// Type name
    pub type_name: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Grammatical error match.
pub struct Match {
    /// Match context
    pub context: Context,
    #[cfg(feature = "unstable")]
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for
    pub context_for_sure_match: isize,
    #[cfg(feature = "unstable")]
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for
    pub ignore_for_incomplete_sentence: bool,
    /// Match length
    pub length: usize,
    /// Error message
    pub message: String,
    #[cfg(feature = "cli")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// More context to match, post-processed using original text
    pub more_context: Option<MoreContext>,
    /// Char index at which the match start
    pub offset: usize,
    /// List of possible replacements (if applies)
    pub replacements: Vec<Replacement>,
    /// Match rule that was not satisfied
    pub rule: Rule,
    /// Sentence in which the error was found
    pub sentence: String,
    /// Short message about the error
    pub short_message: String,
    #[cfg(feature = "unstable")]
    #[serde(rename = "type")]
    /// Match type
    pub type_: Type,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// LanguageTool software details.
pub struct Software {
    /// LanguageTool API version
    pub api_version: usize,
    /// Some information about build date
    pub build_date: String,
    /// Name (should be `"LanguageTool"`)
    pub name: String,
    /// Tell whether the server uses premium API or not
    pub premium: bool,
    #[cfg(feature = "unstable")]
    /// Sentence that indicates if using premium API would find more errors
    pub premium_hint: Option<String>,
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for
    pub status: String,
    /// LanguageTool version
    pub version: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Warnings about check response.
pub struct Warnings {
    /// Indicate if results are incomplete
    pub incomplete_results: bool,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// LanguageTool POST check response.
pub struct CheckResponse {
    /// Language information
    pub language: LanguageResponse,
    /// List of error matches
    pub matches: Vec<Match>,
    #[cfg(feature = "unstable")]
    /// Ranges ([start, end]) of sentences
    pub sentence_ranges: Option<Vec<[usize; 2]>>,
    /// LanguageTool software information
    pub software: Software,
    #[cfg(feature = "unstable")]
    /// Possible warnings
    pub warnings: Option<Warnings>,
}

impl CheckResponse {
    /// Return an iterator over matches.
    pub fn iter_matches(&self) -> std::slice::Iter<'_, Match> {
        self.matches.iter()
    }

    /// Return an iterator over mutable matches.
    pub fn iter_matches_mut(&mut self) -> std::slice::IterMut<'_, Match> {
        self.matches.iter_mut()
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Check response with additional context.
///
/// This structure exists to keep a link between a check response
/// and the original text that was checked.
pub struct CheckResponseWithContext {
    /// Original text that was checked by LT
    pub text: String,
    /// Check response
    pub response: CheckResponse,
    /// Text's length
    pub text_length: usize,
}

impl CheckResponseWithContext {
    /// Bind a check response with its original text.
    #[must_use]
    pub fn new(text: String, response: CheckResponse) -> Self {
        let text_length = text.chars().count();
        Self {
            text,
            response,
            text_length,
        }
    }

    /// Return an iterator over matches.
    pub fn iter_matches(&self) -> std::slice::Iter<'_, Match> {
        self.response.iter_matches()
    }

    /// Return an iterator over mutable matches.
    pub fn iter_matches_mut(&mut self) -> std::slice::IterMut<'_, Match> {
        self.response.iter_matches_mut()
    }

    /// Return an iterator over matches and correspondig line number and line offset.
    #[must_use]
    pub fn iter_match_positions(&self) -> MatchPositions<'_, std::slice::Iter<'_, Match>> {
        self.into()
    }

    /// Append a check response to the current while
    /// adjusting the matches' offsets.
    ///
    /// This is especially useful when a text was split in multiple requests.
    #[must_use]
    pub fn append(mut self, mut other: Self) -> Self {
        let offset = self.text_length;
        for m in other.iter_matches_mut() {
            m.offset += offset;
        }

        #[cfg(feature = "unstable")]
        if let Some(ref mut sr_other) = other.response.sentence_ranges {
            match self.response.sentence_ranges {
                Some(ref mut sr_self) => {
                    sr_self.append(sr_other);
                }
                None => {
                    std::mem::swap(
                        &mut self.response.sentence_ranges,
                        &mut other.response.sentence_ranges,
                    );
                }
            }
        }

        self.response.matches.append(&mut other.response.matches);
        self.text.push_str(other.text.as_str());
        self.text_length += other.text_length;
        self
    }
}

#[cfg(feature = "cli")]
impl From<CheckResponseWithContext> for CheckResponse {
    #[allow(clippy::needless_borrow)]
    fn from(mut resp: CheckResponseWithContext) -> Self {
        let iter: MatchPositions<'_, std::slice::IterMut<'_, Match>> = (&mut resp).into();

        for (line_number, line_offset, m) in iter {
            m.more_context = Some(MoreContext {
                line_number,
                line_offset,
            });
        }
        resp.response
    }
}

/// Iterator over matches and their corresponding line number and line offset.
#[derive(Clone, Debug)]
pub struct MatchPositions<'source, T> {
    text_chars: std::str::Chars<'source>,
    matches: T,
    line_number: usize,
    line_offset: usize,
    offset: usize,
}

impl<'source> From<&'source CheckResponseWithContext>
    for MatchPositions<'source, std::slice::Iter<'source, Match>>
{
    fn from(response: &'source CheckResponseWithContext) -> Self {
        MatchPositions {
            text_chars: response.text.chars(),
            matches: response.iter_matches(),
            line_number: 1,
            line_offset: 0,
            offset: 0,
        }
    }
}

impl<'source> From<&'source mut CheckResponseWithContext>
    for MatchPositions<'source, std::slice::IterMut<'source, Match>>
{
    fn from(response: &'source mut CheckResponseWithContext) -> Self {
        MatchPositions {
            text_chars: response.text.chars(),
            matches: response.response.iter_matches_mut(),
            line_number: 1,
            line_offset: 0,
            offset: 0,
        }
    }
}

impl<'source, T> MatchPositions<'source, T> {
    /// Set the line number to a give value.
    ///
    /// By default, the first line number is 1.
    pub fn set_line_number(mut self, line_number: usize) -> Self {
        self.line_number = line_number;
        self
    }

    fn update_line_number_and_offset(&mut self, m: &Match) {
        // TODO: check cases where newline is actually '\r\n' (Windows platforms)
        let n = m.offset - self.offset;
        for _ in 0..n {
            match self.text_chars.next() {
                    Some('\n') => {
                        self.line_number += 1;
                        self.line_offset = 0;
                    }
                    None => panic!("text is shorter than expected, are you sure this text was the one used for the check request?"),
                    _ => self.line_offset += 1,
                }
        }
        self.offset = m.offset;
    }
}

impl<'source> Iterator for MatchPositions<'source, std::slice::Iter<'source, Match>> {
    type Item = (usize, usize, &'source Match);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(m) = self.matches.next() {
            self.update_line_number_and_offset(m);
            Some((self.line_number, self.line_offset, m))
        } else {
            None
        }
    }
}

impl<'source> Iterator for MatchPositions<'source, std::slice::IterMut<'source, Match>> {
    type Item = (usize, usize, &'source mut Match);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(m) = self.matches.next() {
            self.update_line_number_and_offset(m);
            Some((self.line_number, self.line_offset, m))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::check::{Data, DataAnnotation};

    #[derive(Debug)]
    enum Token<'source> {
        Text(&'source str),
        Skip(&'source str),
    }

    #[derive(Debug, Clone)]
    struct ParseTokenError;

    impl<'source> From<&'source str> for Token<'source> {
        fn from(s: &'source str) -> Self {
            if s.chars().all(|c| c.is_ascii_alphabetic()) {
                Token::Text(s)
            } else {
                Token::Skip(s)
            }
        }
    }

    impl<'source> From<Token<'source>> for DataAnnotation {
        fn from(token: Token<'source>) -> Self {
            match token {
                Token::Text(s) => DataAnnotation::new_text(s.to_string()),
                Token::Skip(s) => DataAnnotation::new_markup(s.to_string()),
            }
        }
    }

    #[test]
    fn test_data_annotation() {
        let words: Vec<&str> = "My name is Q34XY".split(' ').collect();
        let data: Data = words.iter().map(|w| Token::from(*w)).collect();

        let expected_data = Data {
            annotation: vec![
                DataAnnotation::new_text("My".to_string()),
                DataAnnotation::new_text("name".to_string()),
                DataAnnotation::new_text("is".to_string()),
                DataAnnotation::new_markup("Q34XY".to_string()),
            ],
        };

        assert_eq!(data, expected_data);
    }
}
