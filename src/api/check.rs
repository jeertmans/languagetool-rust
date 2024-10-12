//! Structures for `check` requests and responses.

use std::{borrow::Cow, mem, ops::Deref};

#[cfg(feature = "annotate")]
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
#[cfg(feature = "cli")]
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize, Serializer};

use crate::error::{Error, Result};

// REQUESTS

/// Parse `v` is valid language code.
///
/// A valid language code is usually
/// - a two character string matching pattern `[a-z]{2}
/// - a five character string matching pattern `[a-z]{2}-[A-Z]{2}
/// - or some more complex ascii string (see below)
///
/// Language code is case-insensitive.
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
/// # use languagetool_rust::api::check::parse_language_code;
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

/// A portion of text to be checked.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct DataAnnotation<'source> {
    /// Text that should be treated as normal text.
    /// 
    /// This or `markup` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Cow<'source, str>>,
    /// Text that should be treated as markup.
    /// 
    /// This or `text` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markup: Option<Cow<'source, str>>,
    /// If set, the markup will be interpreted as this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpret_as: Option<Cow<'source, str>>,
}

impl<'source> DataAnnotation<'source> {
    /// Instantiate a new `DataAnnotation` with text only.
    #[inline]
    #[must_use]
    pub fn new_text<T: Into<Cow<'source, str>>>(text: T) -> Self {
        Self {
            text: Some(text.into()),
            markup: None,
            interpret_as: None,
        }
    }

    /// Instantiate a new `DataAnnotation` with markup only.
    #[inline]
    #[must_use]
    pub fn new_markup<M: Into<Cow<'source, str>>>(markup: M) -> Self {
        Self {
            text: None,
            markup: Some(markup.into()),
            interpret_as: None,
        }
    }

    /// Instantiate a new `DataAnnotation` with markup and its interpretation.
    #[inline]
    #[must_use]
    pub fn new_interpreted_markup<M: Into<Cow<'source, str>>, I: Into<Cow<'source, str>>>(markup: M, interpret_as: I) -> Self {
        Self {
            interpret_as: Some(interpret_as.into()),
            markup: Some(markup.into()),
            text: None,
        }
    }

    /// Return the text or markup within the data annotation.
    ///
    /// # Errors
    ///
    /// If this data annotation does not contain text or markup.
    pub fn try_get_text(&self) -> Result<Cow<'source, str>> {
        if let Some(ref text) = self.text {
            Ok(text.clone())
        } else if let Some(ref markup) = self.markup {
            Ok(markup.clone())
        } else{
            Err(Error::InvalidDataAnnotation(format!("missing either text or markup field in {self:?}")))
        }
    }
}

#[cfg(test)]
mod data_annotation_tests {

    use super::DataAnnotation;

    #[test]
    fn test_text() {
        let da = DataAnnotation::new_text("Hello");

        assert_eq!(da.text.unwrap(), "Hello");
        assert!(da.markup.is_none());
        assert!(da.interpret_as.is_none());
    }

    #[test]
    fn test_markup() {
        let da = DataAnnotation::new_markup("<a>Hello</a>");

        assert!(da.text.is_none());
        assert_eq!(da.markup.unwrap(), "<a>Hello</a>");
        assert!(da.interpret_as.is_none());
    }

    #[test]
    fn test_interpreted_markup() {
        let da =
            DataAnnotation::new_interpreted_markup("<a>Hello</a>", "Hello");

        assert!(da.text.is_none());
        assert_eq!(da.markup.unwrap(), "<a>Hello</a>");
        assert_eq!(da.interpret_as.unwrap(), "Hello");
    }
}

/// Alternative text to be checked.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Data<'source> {
    /// Vector of markup text, see [`DataAnnotation`].
    pub annotation: Vec<DataAnnotation<'source>>,
}

impl<'source, T: Into<DataAnnotation<'source>>> FromIterator<T> for Data<'source> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let annotation = iter.into_iter().map(std::convert::Into::into).collect();
        Data { annotation }
    }
}

impl Serialize for Data<'_> {
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
impl std::str::FromStr for Data<'_> {
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
    /// # use languagetool_rust::api::check::Level;
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
/// # use languagetool_rust::api::check::split_len;
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


macro_rules! declare_request {
    ($name:ident, $lt:lifetime) => {

/// LanguageTool POST check request.
///
/// The main feature - check a text with LanguageTool for possible style and
/// grammar issues.
///
/// The structure below tries to follow as closely as possible the JSON API
/// described [here](https://languagetool.org/http-api/swagger-ui/#!/default/post_check).
#[cfg_attr(all(feature = "cli", $lt == 'static), derive(Args))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct $name<$lt> {
    /// The text to be checked. This or 'data' is required.
    #[cfg_attr(
        feature = "cli",
        clap(short = 't', long, conflicts_with = "data", allow_hyphen_values(true))
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Cow<$lt, str>>,
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
    pub data: Option<Data<$lt>>,
    /// A language code like `en-US`, `de-DE`, `fr`, or `auto` to guess the
    /// language automatically (see `preferredVariants` below).
    ///
    /// For languages with variants (English, German, Portuguese) spell checking
    /// will only be activated when you specify the variant, e.g. `en-GB`
    /// instead of just `en`.
    #[cfg_attr(
        feature = "cli",
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
    /// Set to get Premium API access: your API key (see <https://languagetool.org/editor/settings/api>).
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
    #[serde(skip_serializing_if = "std::ops::Not::not")]
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

    };
}

declare_request!(Request, 'source);

impl<'source> Request<'source> {
    /// Create a new empty request with language set to `"auto"`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            language: "auto".to_string(),
            ..Default::default()
        }
    }

    /// Set the text to be checked and remove potential data field.
    #[must_use]
    pub fn with_text<T: Into<Cow<'source, str>>>(mut self, text: T) -> Self {
        self.text = Some(text.into());
        self.data = None;
        self
    }

    /// Set the data to be checked and remove potential text field.
    #[must_use]
    pub fn with_data(mut self, data: Data<'source>) -> Self {
        self.data = Some(data);
        self.text = None;
        self
    }

    /// Set the data (obtained from string) to be checked and remove potential
    /// text field
    pub fn with_data_str(self, data: &str) -> serde_json::Result<Self> {
        serde_json::from_str(data).map(|data| self.with_data(data))
    }

    /// Set the language of the text / data.
    #[must_use]
    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    /// Return the text within the request.
    ///
    /// # Errors
    ///
    /// If both `self.text` and `self.data` are [`None`].
    /// If any data annotation does not contain text or markup.
    pub fn try_get_text(&self) -> Result<Cow<'source, str>> {
        if let Some(ref text) = self.text {
            Ok(text.clone())
        } else if let Some(ref data) = self.data {
            match data.annotation.len() {
                0 => Ok(Default::default()),
                1 => data.annotation[0].try_get_text(),
                _ => {
                    let mut text = String::new();
                    
                    for da in data.annotation.iter() {
                        text.push_str(da.try_get_text()?.deref());
                    }

                    Ok(Cow::Owned(text))
                }
            }
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
    pub fn get_text(&self) -> Cow<'source, str> {
        self.try_get_text().unwrap()
    }

    /// Split this request into multiple, using [`split_len`] function to split
    /// text.
    ///
    /// # Errors
    ///
    /// If `self.text` is none.
    pub fn try_split(mut self, n: usize, pat: &str) -> Result<Vec<Self>> {
        let text = mem::take(&mut self.text)
            .ok_or_else(|| Error::InvalidRequest("missing text field".to_string()))?;
        let string: &str = match &text {
            Cow::Owned(s) => s.as_str(),
            Cow::Borrowed(s) => s,
        };

        Ok(split_len(string, n, pat)
            .iter()
            .map(|text_fragment| {
                self.clone()
                    .with_text(Cow::Owned(text_fragment.to_string()))
            })
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
    pub fn split(self, n: usize, pat: &str) -> Vec<Self> {
        self.try_split(n, pat).unwrap()
    }
}

#[cfg(test)]
mod request_tests {

    use super::Request;

    #[test]
    fn test_with_text() {
        let req = Request::default().with_text("hello");

        assert_eq!(req.text.unwrap(), "hello");
        assert!(req.data.is_none());
    }

    #[test]
    fn test_with_data() {
        let req = Request::default().with_text("hello");

        assert_eq!(req.text.unwrap(), "hello");
        assert!(req.data.is_none());
    }
}

// RESPONSES

/// Detected language from check request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DetectedLanguage {
    /// Language code, e.g., `"sk-SK"` for Slovak.
    pub code: String,
    /// Confidence level, from 0 to 1.
    #[cfg(feature = "unstable")]
    pub confidence: Option<f64>,
    /// Language name, e.g., `"Slovak"`.
    pub name: String,
    /// Source (file) for the language detection.
    #[cfg(feature = "unstable")]
    pub source: Option<String>,
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
    /// Indicate if the rule is from the premium API.
    #[cfg(feature = "unstable")]
    pub is_premium: Option<bool>,
    /// Issue type.
    pub issue_type: String,
    /// Rule source file.
    #[cfg(feature = "unstable")]
    pub source_file: Option<String>,
    /// Rule sub id.
    pub sub_id: Option<String>,
    /// Rule list of urls.
    pub urls: Option<Vec<Url>>,
}

/// Type of given match.
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
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for.
    #[cfg(feature = "unstable")]
    pub context_for_sure_match: isize,
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for.
    #[cfg(feature = "unstable")]
    pub ignore_for_incomplete_sentence: bool,
    /// Match length.
    pub length: usize,
    /// Error message.
    pub message: String,
    /// More context to match, post-processed using original text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_context: Option<MoreContext>,
    /// Char index at which the match starts.
    pub offset: usize,
    /// List of possible replacements (if applies).
    pub replacements: Vec<Replacement>,
    /// Match rule that was not satisfied.
    pub rule: Rule,
    /// Sentence in which the error was found.
    pub sentence: String,
    /// Short message about the error.
    pub short_message: String,
    /// Match type.
    #[cfg(feature = "unstable")]
    #[serde(rename = "type")]
    pub type_: Type,
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
            return "No errors were found in provided text".to_string();
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
    pub text: Cow<'static, str>,
    /// Check response.
    pub response: Response,
    /// Text's length.
    pub text_length: usize,
}

impl Deref for ResponseWithContext {
    type Target = Response;
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl ResponseWithContext {
    /// Bind a check response with its original text.
    #[must_use]
    pub fn new(text: Cow<'static, str>, response: Response) -> Self {
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

        let mut string = self.text.into_owned();
        string.push_str(other.text.as_ref());
        self.text = Cow::Owned(string);
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

    impl<'source> From<Token<'source>> for DataAnnotation<'source> {
        fn from(token: Token<'source>) -> Self {
            match token {
                Token::Text(s) => DataAnnotation::new_text(s),
                Token::Skip(s) => DataAnnotation::new_markup(s),
            }
        }
    }

    #[test]
    fn test_data_annotation() {
        let words: Vec<&str> = "My name is Q34XY".split(' ').collect();
        let data: Data = words.iter().map(|w| Token::from(*w)).collect();

        let expected_data = Data {
            annotation: vec![
                DataAnnotation::new_text("My"),
                DataAnnotation::new_text("name"),
                DataAnnotation::new_text("is"),
                DataAnnotation::new_markup("Q34XY"),
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
