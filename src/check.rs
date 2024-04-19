//! Structures for `check` requests and responses.

use std::collections::HashMap;
#[cfg(feature = "cli")]
use std::path::PathBuf;

#[cfg(feature = "annotate")]
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
#[cfg(feature = "cli")]
use clap::{Args, Parser, ValueEnum};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use super::error::{Error, Result};

/// Requests

/// Parse `v` is valid language code.
///
/// A valid language code is usually
/// - a two character string matching pattern `[a-z]{2}
/// - a five character string matching pattern `[a-z]{2}-[A-Z]{2}
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
            },
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
        Err(Error::InvalidValue(
            "The value should be `\"auto\"` or match regex pattern: \
             ^[a-zA-Z]{2,3}(-[a-zA-Z]{2}(-[a-zA-Z]+)*)?$"
                .to_string(),
        ))
    }
}

/// Utility function to serialize a optional vector a strings
/// into a comma separated list of strings.
///
/// This is required by reqwest's RequestBuilder, otherwise it
/// will not work.
pub(crate) fn serialize_option_vec_string<S>(
    v: &Option<Vec<String>>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match v {
        Some(v) if v.len() == 1 => serializer.serialize_str(&v[0]),
        Some(v) if v.len() > 1 => {
            let size = v.iter().map(|s| s.len()).sum::<usize>() + v.len() - 1;
            let mut string = String::with_capacity(size);

            string.push_str(&v[0]);

            for s in &v[1..] {
                string.push(',');
                string.push_str(s);
            }

            serializer.serialize_str(string.as_ref())
        },
        _ => serializer.serialize_none(),
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
/// A portion of text to be checked.
pub struct DataAnnotation {
    /// If set, the markup will be interpreted as this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpret_as: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Text that should be treated as markup.
    pub markup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Text that should be treated as normal text.
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
    /// Instantiate a new `DataAnnotation` with text only.
    #[inline]
    #[must_use]
    pub fn new_text(text: String) -> Self {
        Self {
            interpret_as: None,
            markup: None,
            text: Some(text),
        }
    }

    /// Instantiate a new `DataAnnotation` with markup only.
    #[inline]
    #[must_use]
    pub fn new_markup(markup: String) -> Self {
        Self {
            interpret_as: None,
            markup: Some(markup),
            text: None,
        }
    }

    /// Instantiate a new `DataAnnotation` with markup and its interpretation.
    #[inline]
    #[must_use]
    pub fn new_interpreted_markup(markup: String, interpret_as: String) -> Self {
        Self {
            interpret_as: Some(interpret_as),
            markup: Some(markup),
            text: None,
        }
    }
}

#[cfg(test)]
mod data_annotation_tests {

    use crate::check::DataAnnotation;

    #[test]
    fn test_text() {
        let da = DataAnnotation::new_text("Hello".to_string());

        assert_eq!(da.text.unwrap(), "Hello".to_string());
        assert!(da.markup.is_none());
        assert!(da.interpret_as.is_none());
    }

    #[test]
    fn test_markup() {
        let da = DataAnnotation::new_markup("<a>Hello</a>".to_string());

        assert!(da.text.is_none());
        assert_eq!(da.markup.unwrap(), "<a>Hello</a>".to_string());
        assert!(da.interpret_as.is_none());
    }

    #[test]
    fn test_interpreted_markup() {
        let da =
            DataAnnotation::new_interpreted_markup("<a>Hello</a>".to_string(), "Hello".to_string());

        assert!(da.text.is_none());
        assert_eq!(da.markup.unwrap(), "<a>Hello</a>".to_string());
        assert_eq!(da.interpret_as.unwrap(), "Hello".to_string());
    }
}

/// Alternative text to be checked.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Data {
    /// Vector of markup text, see [`DataAnnotation`].
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

/// Possible levels for additional rules.
///
/// Currently, `Level::Picky` adds additional rules
/// with respect to `Level::Default`.
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Hash)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Level {
    /// Default level.
    #[default]
    Default,
    /// Picky level.
    Picky,
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

/// Split a string into as few fragments as possible, where each fragment
/// contains (if possible) a maximum of `n` characters. Pattern str `pat` is
/// used for splitting.
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
/// assert_eq!(
///     split,
///     vec![
///         "I have so many friends.\n",
///         "They are very funny.\n",
///         "I think I am very lucky to have them.\n",
///         "One day, I will write them a poem.\n",
///         "But, in the meantime, I write code.\n"
///     ]
/// );
///
/// let split = split_len(&s, 80, "\n");
///
/// assert_eq!(
///     split,
///     vec![
///         "I have so many friends.\nThey are very funny.\n",
///         "I think I am very lucky to have them.\nOne day, I will write them a poem.\n",
///         "But, in the meantime, I write code.\n"
///     ]
/// );
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
/// assert_eq!(
///     split,
///     vec![
///         "I have so many friends.\nThey are very funny.\nI think I am very lucky to have \
///          them.\n\n",
///         "One day, I will write them a poem.\nBut, in the meantime, I write code.\n"
///     ]
/// );
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

/// LanguageTool POST check request.
///
/// The main feature - check a text with LanguageTool for possible style and
/// grammar issues.
///
/// The structure below tries to follow as closely as possible the JSON API
/// described [here](https://languagetool.org/http-api/swagger-ui/#!/default/post_check).
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Request {
    /// The text to be checked. This or 'data' is required.
    #[cfg_attr(
        feature = "cli",
        clap(short = 't', long, conflicts_with = "data", allow_hyphen_values(true))
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The text to be checked, given as a JSON document that specifies what's
    /// text and what's markup. This or 'text' is required.
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
    /// If you have markup that should be interpreted as whitespace, like `<p>`
    /// in HTML, you can have it interpreted like this:
    ///
    /// ```json
    /// {"markup": "<p>", "interpretAs": "\n\n"}
    /// ```
    /// The 'data' feature is not limited to HTML or XML, it can be used for any
    /// kind of markup. Entities will need to be expanded in this input.
    #[cfg_attr(feature = "cli", clap(short = 'd', long, conflicts_with = "text"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Data>,
    /// A language code like `en-US`, `de-DE`, `fr`, or `auto` to guess the
    /// language automatically (see `preferredVariants` below).
    ///
    /// For languages with variants (English, German, Portuguese) spell checking
    /// will only be activated when you specify the variant, e.g. `en-GB`
    /// instead of just `en`.
    #[cfg_attr(
        all(feature = "cli", feature = "cli", feature = "cli"),
        clap(
            short = 'l',
            long,
            default_value = "auto",
            value_parser = parse_language_code
        )
    )]
    pub language: String,
    /// Set to get Premium API access: Your username/email as used to log in at
    /// languagetool.org.
    #[cfg_attr(
        feature = "cli",
        clap(short = 'u', long, requires = "api_key", env = "LANGUAGETOOL_USERNAME")
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Set to get Premium API access: [your API
    /// key](https://languagetool.org/editor/settings/api).
    #[cfg_attr(
        feature = "cli",
        clap(short = 'k', long, requires = "username", env = "LANGUAGETOOL_API_KEY")
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Comma-separated list of dictionaries to include words from; uses special
    /// default dictionary if this is unset.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub dicts: Option<Vec<String>>,
    /// A language code of the user's native language, enabling false friends
    /// checks for some language pairs.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mother_tongue: Option<String>,
    /// Comma-separated list of preferred language variants.
    ///
    /// The language detector used with `language=auto` can detect e.g. English,
    /// but it cannot decide whether British English or American English is
    /// used. Thus this parameter can be used to specify the preferred variants
    /// like `en-GB` and `de-AT`. Only available with `language=auto`. You
    /// should set variants for at least German and English, as otherwise the
    /// spell checking will not work for those, as no spelling dictionary can be
    /// selected for just `en` or `de`.
    #[cfg_attr(feature = "cli", clap(long, conflicts_with = "language"))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub preferred_variants: Option<Vec<String>>,
    /// IDs of rules to be enabled, comma-separated.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub enabled_rules: Option<Vec<String>>,
    /// IDs of rules to be disabled, comma-separated.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub disabled_rules: Option<Vec<String>>,
    /// IDs of categories to be enabled, comma-separated.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub enabled_categories: Option<Vec<String>>,
    /// IDs of categories to be disabled, comma-separated.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub disabled_categories: Option<Vec<String>>,
    /// If true, only the rules and categories whose IDs are specified with
    /// `enabledRules` or `enabledCategories` are enabled.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(skip_serializing_if = "is_false")]
    pub enabled_only: bool,
    /// If set to `picky`, additional rules will be activated, i.e. rules that
    /// you might only find useful when checking formal text.
    #[cfg_attr(
        feature = "cli",
        clap(long, default_value = "default", ignore_case = true, value_enum)
    )]
    #[serde(skip_serializing_if = "Level::is_default")]
    pub level: Level,
}

impl Default for Request {
    #[inline]
    fn default() -> Request {
        Request {
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

impl Request {
    /// Set the text to be checked and remove potential data field.
    #[must_use]
    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self.data = None;
        self
    }

    /// Set the data to be checked and remove potential text field.
    #[must_use]
    pub fn with_data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self.text = None;
        self
    }

    /// Set the data (obtained from string) to be checked and remove potential
    /// text field
    pub fn with_data_str(self, data: &str) -> serde_json::Result<Self> {
        Ok(self.with_data(serde_json::from_str(data)?))
    }

    /// Set the language of the text / data.
    #[must_use]
    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    /// Return a copy of the text within the request.
    ///
    /// # Errors
    ///
    /// If both `self.text` and `self.data` are [`None`].
    /// If any data annotation does not contain text or markup.
    pub fn try_get_text(&self) -> Result<String> {
        if let Some(ref text) = self.text {
            Ok(text.clone())
        } else if let Some(ref data) = self.data {
            let mut text = String::new();
            for da in data.annotation.iter() {
                if let Some(ref t) = da.text {
                    text.push_str(t.as_str());
                } else if let Some(ref t) = da.markup {
                    text.push_str(t.as_str());
                } else {
                    return Err(Error::InvalidDataAnnotation(
                        "missing either text or markup field in {da:?}".to_string(),
                    ));
                }
            }
            Ok(text)
        } else {
            Err(Error::InvalidRequest(
                "missing either text or data field".to_string(),
            ))
        }
    }

    /// Return a copy of the text within the request.
    /// Call [`Request::try_get_text`] but panic on error.
    ///
    /// # Panics
    ///
    /// If both `self.text` and `self.data` are [`None`].
    /// If any data annotation does not contain text or markup.
    #[must_use]
    pub fn get_text(&self) -> String {
        self.try_get_text().unwrap()
    }

    /// Split this request into multiple, using [`split_len`] function to split
    /// text.
    ///
    /// # Errors
    ///
    /// If `self.text` is none.
    pub fn try_split(&self, n: usize, pat: &str) -> Result<Vec<Self>> {
        let text = self
            .text
            .as_ref()
            .ok_or(Error::InvalidRequest("missing text field".to_string()))?;

        Ok(split_len(text.as_str(), n, pat)
            .iter()
            .map(|text_fragment| self.clone().with_text(text_fragment.to_string()))
            .collect())
    }

    /// Split this request into multiple, using [`split_len`] function to split
    /// text.
    /// Call [`Request::try_split`] but panic on error.
    ///
    /// # Panics
    ///
    /// If `self.text` is none.
    #[must_use]
    pub fn split(&self, n: usize, pat: &str) -> Vec<Self> {
        self.try_split(n, pat).unwrap()
    }
}

/// Parse a string slice into a [`PathBuf`], and error if the file does not
/// exist.
#[cfg(feature = "cli")]
fn parse_filename(s: &str) -> Result<PathBuf> {
    let path_buf: PathBuf = s.parse().unwrap();

    if path_buf.is_file() {
        Ok(path_buf)
    } else {
        Err(Error::InvalidFilename(s.to_string()))
    }
}

/// Supported file types.
#[cfg(feature = "cli")]
#[derive(Clone, Debug, Default, ValueEnum)]
#[non_exhaustive]
pub enum FileTypeOptions {
    /// Auto.
    #[default]
    Auto,
    /// Text.
    Text,
    /// Markdown.
    Markdown,
    /// Typst.
    Typst,
}

pub enum FileType {}

/// Check text using LanguageTool server.
///
/// The input can be one of the following:
///
/// - raw text, if `--text TEXT` is provided;
/// - annotated data, if `--data TEXT` is provided;
/// - raw text, if `-- [FILE]...` are provided. Note that some file types will
///   use a
/// - raw text, through stdin, if nothing is provided.
#[cfg(feature = "cli")]
#[derive(Debug, Parser)]
pub struct CheckCommand {
    /// If present, raw JSON output will be printed instead of annotated text.
    /// This has no effect if `--data` is used, because it is never
    /// annotated.
    #[cfg(feature = "cli")]
    #[clap(short = 'r', long)]
    pub raw: bool,
    /// Sets the maximum number of characters before splitting.
    #[clap(long, default_value_t = 1500)]
    pub max_length: usize,
    /// If text is too long, will split on this pattern.
    #[clap(long, default_value = "\n\n")]
    pub split_pattern: String,
    /// Max. number of suggestions kept. If negative, all suggestions are kept.
    #[clap(long, default_value_t = 5, allow_negative_numbers = true)]
    pub max_suggestions: isize,
    /// Specify the files type to use the correct parser.
    ///
    /// If set to auto, the type is guessed from the filename extension.
    #[clap(long, default_value = "default", ignore_case = true, value_enum)]
    pub r#type: FileType,
    /// Optional filenames from which input is read.
    #[arg(conflicts_with_all(["text", "data"]), value_parser = parse_filename)]
    pub filenames: Vec<PathBuf>,
    /// Inner [`Request`].
    #[command(flatten, next_help_heading = "Request options")]
    pub request: Request,
}

#[cfg(test)]
mod request_tests {

    use crate::Request;

    #[test]
    fn test_with_text() {
        let req = Request::default().with_text("hello".to_string());

        assert_eq!(req.text.unwrap(), "hello".to_string());
        assert!(req.data.is_none());
    }

    #[test]
    fn test_with_data() {
        let req = Request::default().with_text("hello".to_string());

        assert_eq!(req.text.unwrap(), "hello".to_string());
        assert!(req.data.is_none());
    }
}

/// Responses

/// Detected language from check request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DetectedLanguage {
    /// Language code, e.g., `"sk-SK"` for Slovak.
    pub code: String,
    /// Language name, e.g., `"Slovak"`.
    pub name: String,
    /// Undocumented fields.
    ///
    /// Examples are:
    ///
    /// - 'confidence', the confidence level, from 0 to 1;
    /// - 'source', the source file for the language detection.
    #[cfg(feature = "undoc")]
    #[serde(flatten)]
    pub undocumented: HashMap<String, Value>,
}

/// Language information in check response.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LanguageResponse {
    /// Language code, e.g., `"sk-SK"` for Slovak.
    pub code: String,
    /// Detected language from provided request.
    pub detected_language: DetectedLanguage,
    /// Language name, e.g., `"Slovak"`.
    pub name: String,
}

/// Match context in check response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Context {
    /// Length of the match.
    pub length: usize,
    /// Char index at which the match starts.
    pub offset: usize,
    /// Contextual text around the match.
    pub text: String,
}

/// More context, post-processed in check response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct MoreContext {
    /// Line number where match occurred.
    pub line_number: usize,
    /// Char index at which the match starts on the current line.
    pub line_offset: usize,
}

/// Possible replacement for a given match in check response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Replacement {
    /// Possible replacement value.
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

/// A rule category.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Category {
    /// Category id.
    pub id: String,
    /// Category name.
    pub name: String,
}

/// A possible url of a rule in a check response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Url {
    /// Url value.
    pub value: String,
}

/// The rule that was not satisfied in a given match.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Rule {
    /// Rule category.
    pub category: Category,
    /// Rule description.
    pub description: String,
    /// Rule id.
    pub id: String,
    /// Issue type.
    pub issue_type: String,
    /// Rule source file.
    /// Rule sub id.
    pub sub_id: Option<String>,
    /// Rule list of urls.
    pub urls: Option<Vec<Url>>,
    /// Undocumented fields.
    ///
    /// Examples are:
    ///
    /// - 'is_premium', indicate if the rule is from the premium API;
    /// - 'source_file', the source file of the rule.
    #[cfg(feature = "undoc")]
    #[serde(flatten)]
    pub undocumented: HashMap<String, Value>,
}

/// Type of a given match.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Type {
    /// Type name.
    pub type_name: String,
}

/// Grammatical error match.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Match {
    /// Match context.
    pub context: Context,
    /// Match length.
    pub length: usize,
    /// Error message.
    pub message: String,
    /// More context to match, post-processed using original text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_context: Option<MoreContext>,
    /// Char index at which the match start.
    pub offset: usize,
    /// List of possible replacements (if applies).
    pub replacements: Vec<Replacement>,
    /// Match rule that was not satisfied.
    pub rule: Rule,
    /// Sentence in which the error was found.
    pub sentence: String,
    /// Short message about the error.
    pub short_message: String,
    /// Undocumented fields.
    ///
    /// Examples are:
    ///
    /// - 'type', the match type;
    /// - 'context_for_sure_match', unknown;
    /// - 'ignore_for_incomplete_sentence', unknown;
    #[cfg(feature = "undoc")]
    #[serde(flatten)]
    pub undocumented: HashMap<String, Value>,
}

/// LanguageTool software details.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Software {
    /// LanguageTool API version.
    pub api_version: usize,
    /// Some information about build date.
    pub build_date: String,
    /// Name (should be `"LanguageTool"`).
    pub name: String,
    /// Tell whether the server uses premium API or not.
    pub premium: bool,
    /// Sentence that indicates if using premium API would find more errors.
    #[cfg(feature = "unstable")]
    pub premium_hint: Option<String>,
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for.
    pub status: String,
    /// LanguageTool version.
    pub version: String,
}

/// Warnings about check response.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Warnings {
    /// Indicate if results are incomplete.
    pub incomplete_results: bool,
}

/// LanguageTool POST check response.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Response {
    /// Language information.
    pub language: LanguageResponse,
    /// List of error matches.
    pub matches: Vec<Match>,
    /// Ranges ([start, end]) of sentences.
    #[cfg(feature = "unstable")]
    pub sentence_ranges: Option<Vec<[usize; 2]>>,
    /// LanguageTool software information.
    pub software: Software,
    /// Possible warnings.
    #[cfg(feature = "unstable")]
    pub warnings: Option<Warnings>,
}

impl Response {
    /// Return an iterator over matches.
    pub fn iter_matches(&self) -> std::slice::Iter<'_, Match> {
        self.matches.iter()
    }

    /// Return an iterator over mutable matches.
    pub fn iter_matches_mut(&mut self) -> std::slice::IterMut<'_, Match> {
        self.matches.iter_mut()
    }

    /// Creates an annotated string from current response.
    #[cfg(feature = "annotate")]
    #[must_use]
    pub fn annotate(&self, text: &str, origin: Option<&str>, color: bool) -> String {
        if self.matches.is_empty() {
            return "No error were found in provided text".to_string();
        }
        let replacements: Vec<_> = self
            .matches
            .iter()
            .map(|m| {
                m.replacements.iter().fold(String::new(), |mut acc, r| {
                    if !acc.is_empty() {
                        acc.push_str(", ");
                    }
                    acc.push_str(&r.value);
                    acc
                })
            })
            .collect();

        let snippets = self.matches.iter().zip(replacements.iter()).map(|(m, r)| {
            Snippet {
                title: Some(Annotation {
                    label: Some(&m.message),
                    id: Some(&m.rule.id),
                    annotation_type: AnnotationType::Error,
                }),
                footer: vec![],
                slices: vec![Slice {
                    source: &m.context.text,
                    line_start: 1 + text.chars().take(m.offset).filter(|c| *c == '\n').count(),
                    origin,
                    fold: true,
                    annotations: vec![
                        SourceAnnotation {
                            label: &m.rule.description,
                            annotation_type: AnnotationType::Error,
                            range: (m.context.offset, m.context.offset + m.context.length),
                        },
                        SourceAnnotation {
                            label: r,
                            annotation_type: AnnotationType::Help,
                            range: (m.context.offset, m.context.offset + m.context.length),
                        },
                    ],
                }],
                opt: FormatOptions {
                    color,
                    ..Default::default()
                },
            }
        });

        let mut annotation = String::new();

        for snippet in snippets {
            if !annotation.is_empty() {
                annotation.push('\n');
            }
            annotation.push_str(&DisplayList::from(snippet).to_string());
        }
        annotation
    }
}

/// Check response with additional context.
///
/// This structure exists to keep a link between a check response
/// and the original text that was checked.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseWithContext {
    /// Original text that was checked by LT.
    pub text: String,
    /// Check response.
    pub response: Response,
    /// Text's length.
    pub text_length: usize,
}

impl ResponseWithContext {
    /// Bind a check response with its original text.
    #[must_use]
    pub fn new(text: String, response: Response) -> Self {
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

    /// Return an iterator over matches and corresponding line number and line
    /// offset.
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
                },
                None => {
                    std::mem::swap(
                        &mut self.response.sentence_ranges,
                        &mut other.response.sentence_ranges,
                    );
                },
            }
        }

        self.response.matches.append(&mut other.response.matches);
        self.text.push_str(other.text.as_str());
        self.text_length += other.text_length;
        self
    }
}

impl From<ResponseWithContext> for Response {
    #[allow(clippy::needless_borrow)]
    fn from(mut resp: ResponseWithContext) -> Self {
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

impl<'source> From<&'source ResponseWithContext>
    for MatchPositions<'source, std::slice::Iter<'source, Match>>
{
    fn from(response: &'source ResponseWithContext) -> Self {
        MatchPositions {
            text_chars: response.text.chars(),
            matches: response.iter_matches(),
            line_number: 1,
            line_offset: 0,
            offset: 0,
        }
    }
}

impl<'source> From<&'source mut ResponseWithContext>
    for MatchPositions<'source, std::slice::IterMut<'source, Match>>
{
    fn from(response: &'source mut ResponseWithContext) -> Self {
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
        let n = m.offset - self.offset;
        for _ in 0..n {
            match self.text_chars.next() {
                Some('\n') => {
                    self.line_number += 1;
                    self.line_offset = 0;
                },
                None => {
                    panic!(
                        "text is shorter than expected, are you sure this text was the one used \
                         for the check request?"
                    )
                },
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
    use super::*;

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

    #[test]
    fn test_serialize_option_vec_string() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Foo {
            #[serde(serialize_with = "serialize_option_vec_string")]
            values: Option<Vec<String>>,
        }

        impl Foo {
            fn new<I, T>(values: I) -> Self
            where
                I: IntoIterator<Item = T>,
                T: ToString,
            {
                Self {
                    values: Some(values.into_iter().map(|v| v.to_string()).collect()),
                }
            }
            fn none() -> Self {
                Self { values: None }
            }
        }

        let got = serde_json::to_string(&Foo::new(vec!["en-US", "de-DE"])).unwrap();
        assert_eq!(got, r#"{"values":"en-US,de-DE"}"#);

        let got = serde_json::to_string(&Foo::new(vec!["en-US"])).unwrap();
        assert_eq!(got, r#"{"values":"en-US"}"#);

        let got = serde_json::to_string(&Foo::new(Vec::<String>::new())).unwrap();
        assert_eq!(got, r#"{"values":null}"#);

        let got = serde_json::to_string(&Foo::none()).unwrap();
        assert_eq!(got, r#"{"values":null}"#);
    }
}

/// Annotate a response by using its request context.
pub fn annotate(response: &Response, request: &Request) -> String {}
