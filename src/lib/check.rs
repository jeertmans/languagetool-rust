#[cfg(feature = "cli")]
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DataAnnotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpret_as: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub annotation: Vec<DataAnnotation>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Software {
    pub name: String,
    pub version: String,
    pub build_date: String,
    pub api_version: isize,
    pub status: String,
    pub premium: bool,
}

#[derive(Debug, Deserialize)]
pub struct DetectedLanguage {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageCheck {
    pub name: String,
    pub code: String,
    pub detected_language: DetectedLanguage,
}

#[derive(Debug, Deserialize)]
pub struct Replacement {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Context {
    pub text: String,
    pub offset: isize,
    pub length: isize,
}

#[derive(Debug, Deserialize)]
pub struct Url {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub sub_id: String,
    pub description: String,
    pub urls: Vec<Url>,
    pub issue_type: String,
    pub category: Category,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub message: String,
    pub short_message: String,
    pub offset: isize,
    pub length: isize,
    pub replacements: Vec<Replacement>,
    pub context: Context,
    pub sentence: String,
    // rule: Rule, // Seems to cause problems with missing fields
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Default,
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
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// LanguageTool [POST] check request.
///
/// The main feature - check a text with LanguageTool for possible style and grammar issues.
///
/// The structure below tries to follow as closely as possible the JSON API decribed
/// [here](https://languagetool.org/http-api/swagger-ui/#!/default/post_check).
pub struct CheckRequest {
    #[cfg_attr(
        feature = "cli",
        clap(
            short = 'c',
            long,
            required_unless_present = "data",
            conflicts_with = "data"
        )
    )]
    /// The text to be checked. This or 'data' is required.
    pub text: Option<String>,
    #[cfg_attr(
        feature = "cli",
        clap(
            short = 'd',
            long,
            required_unless_present = "text",
            conflicts_with = "text"
        )
    )]
    //#[serde(serialize_with = "serde_qs::serialize_struct")]
    /// The text to be checked, given as a JSON document that specifies what's text and what's markup. This or 'text' is required. Markup will be ignored when looking for errors. Example text:
    /// ```
    /// A <b>test</b>
    /// ```
    /// JSON for the example text:
    /// ```
    /// {"annotation":[
    ///  {"text": "A "},
    ///  {"markup": "<b>"},
    ///  {"text": "test"},
    ///  {"markup": "</b>"}
    /// ]}
    /// ```
    /// If you have markup that should be interpreted as whitespace, like <p> in HTML, you can have it interpreted like this:
    ///
    /// ```
    /// {"markup": "<p>", "interpretAs": "\n\n"}
    /// ```
    /// The 'data' feature is not limited to HTML or XML, it can be used for any kind of markup. Entities will need to be expanded in this input.
    pub data: Option<Data>,
    #[cfg_attr(feature = "cli", clap(short = 'l', long, required = true))]
    /// A language code like `en-US`, `de-DE`, `fr`, or `auto` to guess the language automatically (see `preferredVariants` below). For languages with variants (English, German, Portuguese) spell checking will only be activated when you specify the variant, e.g. `en-GB` instead of just `en`.
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
    /// Comma-separated list of preferred language variants. The language detector used with `language=auto` can detect e.g. English, but it cannot decide whether British English or American English is used. Thus this parameter can be used to specify the preferred variants like `en-GB` and `de-AT`. Only available with `language=auto`. You should set variants for at least German and English, as otherwise the spell checking will not work for those, as no spelling dictionary can be selected for just `en` or `de`.
    pub preferred_variants: Option<Vec<String>>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// IDs of rules to be enabled, comma-separated
    pub enabled_rules: Option<Vec<isize>>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// IDs of rules to be disabled, comma-separated
    pub disabled_rules: Option<Vec<isize>>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// IDs of categories to be enabled, comma-separated
    pub enabled_categories: Option<Vec<isize>>,
    #[cfg_attr(feature = "cli", clap(long, multiple_values = true))]
    /// IDs of categories to be disabled, comma-separated
    pub disabled_categories: Option<Vec<isize>>,
    #[cfg_attr(feature = "cli", clap(long, takes_value = false))]
    /// If true, only the rules and categories whose IDs are specified with `enabledRules` or `enabledCategories` are enabled.
    pub enabled_only: bool,
    #[cfg_attr(feature = "cli", clap(long, default_value = "default"))]
    /// If set to `picky`, additional rules will be activated, i.e. rules that you might only find useful when checking formal text.
    pub level: Level,
}

impl CheckRequest {
    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        //self.data = None;
        self
    }

    pub fn with_language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResponse {
    software: Software,
    language: LanguageCheck,
    matches: Vec<Match>,
}
