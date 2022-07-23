//! Structures for `check` requests and responses.

#[cfg(feature = "cli")]
use clap::Parser;
use serde::{Deserialize, Serialize};

/// Requests

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
    pub fn new_text(text: String) -> Self {
        Self {
            interpret_as: None,
            markup: None,
            text: Some(text),
        }
    }

    #[inline]
    /// Instantiate a new `DataAnnotation` with markup only
    pub fn new_markup(markup: String) -> Self {
        Self {
            interpret_as: None,
            markup: Some(markup),
            text: None,
        }
    }

    #[inline]
    /// Instantiate a new `DataAnnotation` with markup and its interpretation
    pub fn new_interpreted_markup(markup: String, interpret_as: String) -> Self {
        Self {
            interpret_as: Some(interpret_as),
            markup: Some(markup),
            text: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
/// Alternative text to be checked.
pub struct Data {
    /// Vector of markup text, see [DataAnnotation]
    pub annotation: Vec<DataAnnotation>,
}

impl<T: Into<DataAnnotation>> FromIterator<T> for Data {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let annotation = iter.into_iter().map(|x| x.into()).collect();
        Data { annotation }
    }
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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
    type Err = clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|e| clap::Command::new("").error(clap::ErrorKind::InvalidValue, e.to_string()))
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
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

#[cfg(feature = "cli")]
impl std::str::FromStr for Level {
    type Err = clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "default" => Ok(Level::Default),
            "picky" => Ok(Level::Picky),
            _ => Err(clap::Command::new("").error(
                clap::ErrorKind::InvalidValue,
                format!("Could not convert `{}` into either `default` or `picky`", s),
            )),
        }
    }
}

#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Clone, Deserialize, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
/// LanguageTool POST check request.
///
/// The main feature - check a text with LanguageTool for possible style and grammar issues.
///
/// The structure below tries to follow as closely as possible the JSON API decribed
/// [here](https://languagetool.org/http-api/swagger-ui/#!/default/post_check).
pub struct CheckRequest {
    #[cfg(all(feature = "cli", feature = "annotate"))]
    #[clap(short = 'r', long, takes_value = false)]
    /// If present, raw JSON output will be printed instead of annotated text.
    pub raw: bool,
    #[cfg_attr(feature = "cli", clap(short = 't', long, conflicts_with = "data",))]
    /// The text to be checked. This or 'data' is required.
    pub text: Option<String>,
    #[cfg_attr(feature = "cli", clap(short = 'd', long, conflicts_with = "text"))]
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
    /// If you have markup that should be interpreted as whitespace, like <p> in HTML, you can have it interpreted like this:
    ///
    /// ```json
    /// {"markup": "<p>", "interpretAs": "\n\n"}
    /// ```
    /// The 'data' feature is not limited to HTML or XML, it can be used for any kind of markup. Entities will need to be expanded in this input.
    pub data: Option<Data>,
    #[cfg_attr(feature = "cli", clap(short = 'l', long, default_value = "auto"))]
    /// A language code like `en-US`, `de-DE`, `fr`, or `auto` to guess the language automatically (see `preferredVariants` below).
    ///
    /// For languages with variants (English, German, Portuguese) spell checking will only be activated when you specify the variant, e.g. `en-GB` instead of just `en`.
    pub language: String,
    #[cfg_attr(feature = "cli", clap(short = 'u', long))]
    /// Set to get Premium API access: Your username/email as used to log in at languagetool.org.
    pub username: Option<String>,
    #[cfg_attr(feature = "cli", clap(short = 'k', long))]
    /// Set to get Premium API access: [your API key](https://languagetool.org/editor/settings/api)
    pub api_key: Option<String>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// Comma-separated list of dictionaries to include words from; uses special default dictionary if this is unset
    pub dicts: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long))]
    /// A language code of the user's native language, enabling false friends checks for some language pairs.
    pub mother_tongue: Option<String>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// Comma-separated list of preferred language variants.
    ///
    /// The language detector used with `language=auto` can detect e.g. English, but it cannot decide whether British English or American English is used. Thus this parameter can be used to specify the preferred variants like `en-GB` and `de-AT`. Only available with `language=auto`. You should set variants for at least German and English, as otherwise the spell checking will not work for those, as no spelling dictionary can be selected for just `en` or `de`.
    pub preferred_variants: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// IDs of rules to be enabled, comma-separated
    pub enabled_rules: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// IDs of rules to be disabled, comma-separated
    pub disabled_rules: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// IDs of categories to be enabled, comma-separated
    pub enabled_categories: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// IDs of categories to be disabled, comma-separated
    pub disabled_categories: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long, takes_value = false))]
    /// If true, only the rules and categories whose IDs are specified with `enabledRules` or `enabledCategories` are enabled.
    pub enabled_only: bool,
    #[cfg_attr(feature = "cli", clap(long, default_value = "default"))]
    /// If set to `picky`, additional rules will be activated, i.e. rules that you might only find useful when checking formal text.
    pub level: Level,
}

impl CheckRequest {
    #[inline]
    /// Create a default check requests that matches default values from CLI options
    pub fn default() -> Self {
        Self {
            language: "auto".to_owned(),
            ..Default::default()
        }
    }

    /// Set the text to be checked and removed potential data field
    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self.data = None;
        self
    }

    /// Set the data to be checked and removed potential text field
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
    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }
}

/// Reponses

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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
/// Language information in check response.
pub struct LanguageResponse {
    /// Language code, e.g., `"sk-SK"` for Slovak
    pub code: String,
    /// Detected language from provided request
    pub detected_language: DetectedLanguage,
    /// Language name, e.g., `"Slovak"`
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// Match context in check response.
pub struct Context {
    /// Length of the match
    pub length: usize,
    /// Char index at which the match starts
    pub offset: usize,
    /// Contextual text aroung the match
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// A rule category.
pub struct Category {
    /// Category id
    pub id: String,
    /// Category name
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// A possible url of a rule in a check response.
pub struct Url {
    /// Url value
    pub value: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Type of a given match.
pub struct Type {
    /// Type name
    pub type_name: String,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// LanguageTool software details.
pub struct Software {
    /// LanguageTool API version
    pub api_version: usize,
    /// Some information about build date
    pub build_date: String,
    /// Name (should be `"LanguageTool"`)
    pub name: String,
    /// Tell wether the server uses premium API or not
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

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Warnings about check response.
pub struct Warnings {
    /// Indicate if results are imcomplete
    pub incomplete_results: bool,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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
