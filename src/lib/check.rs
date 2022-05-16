#[cfg(feature = "cli")]
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Software {
    name: String,
    version: String,
    build_date: String,
    api_version: isize,
    status: String,
    premium: bool,
}
#[derive(Debug, Deserialize)]
pub struct DetectedLanguage {
    name: String,
    code: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageCheck {
    name: String,
    code: String,
    detected_language: DetectedLanguage,
}
#[derive(Debug, Deserialize)]
pub struct Replacement {
    value: String,
}
#[derive(Debug, Deserialize)]
pub struct Context {
    text: String,
    offset: isize,
    length: isize,
}
#[derive(Debug, Deserialize)]
pub struct Url {
    value: String,
}
#[derive(Debug, Deserialize)]
pub struct Category {
    id: String,
    name: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    id: String,
    sub_id: String,
    description: String,
    urls: Vec<Url>,
    issue_type: String,
    category: Category,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    message: String,
    short_message: String,
    offset: isize,
    length: isize,
    replacements: Vec<Replacement>,
    context: Context,
    sentence: String,
    // rule: Rule, // Seems to cause problems with missing fields
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Level {
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
pub struct CheckRequest {
    #[cfg_attr(feature = "cli", clap(short = 'c', long))]
    text: Option<String>,
    //#[cfg_attr(feature = "cli", clap(short = 'd', long, conflicts_with = "text"))]
    //data: ,
    #[cfg_attr(feature = "cli", clap(short = 'l', long, required = true))]
    language: String,
    #[cfg_attr(feature = "cli", clap(short = 'u', long))]
    username: Option<String>,
    #[cfg_attr(feature = "cli", clap(short = 'k', long))]
    api_key: Option<String>,
    /*
    dicts: Option<Vec<String>>,
    mother_tongue: Option<String>,
    preferred_variants: Option<Vec<String>>,
    enabled_rules: Option<Vec<isize>>,
    disabled_rules: Option<Vec<isize>>,
    enabled_categories: Option<Vec<isize>>,
    disabled_categories: Option<Vec<isize>>,*/
    #[cfg_attr(feature = "cli", clap(long, takes_value = false))]
    enabled_only: bool,
    #[cfg_attr(feature = "cli", clap(long, default_value = "default"))]
    level: Level,
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
