//! Check text using LanguageTool server.
//!
//! The input can be one of the following:
//!
//! - raw text, if `--text TEXT` is provided;
//! - annotated data, if `--data TEXT` is provided;
//! - text from file(s), if `[FILE(S)]...` are provided.
//! - raw text through `stdin`, if nothing else is provided.
use std::{borrow::Cow, io::Write, path::PathBuf};

use clap::{Args, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use termcolor::{StandardStream, WriteColor};

use crate::{
    api::{
        check::{
            self, parse_language_code, Data, DataAnnotation, Level, Request, DEFAULT_LANGUAGE,
        },
        server::ServerClient,
    },
    error::{Error, Result},
    parsers::{html::parse_html, markdown::parse_markdown, typst::parse_typst},
};

use super::ExecuteSubcommand;

/// Parse a string slice into a [`PathBuf`], and error if the file does not
/// exist.
fn parse_filename(s: &str) -> Result<PathBuf> {
    let path_buf = PathBuf::from(s);

    if path_buf.is_file() {
        Ok(path_buf)
    } else {
        Err(Error::InvalidFilename(s.to_string()))
    }
}

/// Command to check a text with LanguageTool for possible style and grammar
/// issues.
#[derive(Debug, Parser)]
pub struct Command {
    /// If present, raw JSON output will be printed instead of annotated text.
    /// This has no effect if `--data` is used, because it is never
    /// annotated.
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
    #[clap(long, value_enum, default_value_t = FileType::default(), ignore_case = true)]
    pub r#type: FileType,
    /// Optional filenames from which input is read.
    #[arg(conflicts_with_all(["text", "data"]), value_parser = parse_filename)]
    pub filenames: Vec<PathBuf>,
    /// Inner [`Request`].
    #[command(flatten, next_help_heading = "Request options")]
    pub request: CliRequest,
}

/// Support file types.
#[derive(Clone, Debug, Default, ValueEnum)]
#[non_exhaustive]
pub enum FileType {
    /// Auto.
    #[default]
    Auto,
    /// Raw text.
    Raw,
    /// Markdown.
    Markdown,
    /// HTML.
    Html,
    /// Typst.
    Typst,
}

impl ExecuteSubcommand for Command {
    /// Executes the `check` subcommand.
    async fn execute(self, mut stdout: StandardStream, server_client: ServerClient) -> Result<()> {
        let mut request: check::Request = self.request.into();
        #[cfg(feature = "annotate")]
        let color = stdout.supports_color();

        let server_client = server_client.with_max_suggestions(self.max_suggestions);

        // ANNOTATED DATA, RAW TEXT, STDIN
        if self.filenames.is_empty() {
            // Fallback to `stdin` if nothing else is provided
            if request.text.is_none() && request.data.is_none() {
                let mut text = String::new();
                super::read_from_stdin(&mut stdout, &mut text)?;
                request = request.with_text(Cow::Owned(text));
            }

            if request.text.is_none() {
                // Handle annotated data
                let response = server_client.check(&request).await?;
                writeln!(&mut stdout, "{}", serde_json::to_string_pretty(&response)?)?;
                return Ok(());
            };

            let requests = request.split(self.max_length, self.split_pattern.as_str());
            let response = server_client.check_multiple_and_join(requests).await?;

            writeln!(
                &mut stdout,
                "{}",
                &response.annotate(response.text.as_ref(), None, color)
            )?;

            return Ok(());
        }

        // FILES
        for filename in self.filenames.iter() {
            let mut file_type = self.r#type.clone();

            // If file type is "Auto", guess file type from extension
            if matches!(self.r#type, FileType::Auto) {
                file_type = match PathBuf::from(filename).extension().and_then(|e| e.to_str()) {
                    Some(ext) => {
                        match ext {
                            "typ" => FileType::Typst,
                            "md" | "markdown" | "mdown" | "mdwn" | "mkd" | "mkdn" | "mdx" => {
                                FileType::Markdown
                            },

                            "html" | "htm" => FileType::Html,
                            _ => {
                                log::debug!("Unknown file type: {ext}.");
                                FileType::Raw
                            },
                        }
                    },
                    None => {
                        log::debug!("No extension found for file: {:?}.", filename);
                        FileType::Raw
                    },
                };
            };

            let file_content = std::fs::read_to_string(filename)?;

            let (response, text): (check::Response, String) = match &file_type {
                FileType::Auto => unreachable!(),
                FileType::Raw => {
                    let requests = (request.clone().with_text(&file_content))
                        .split(self.max_length, self.split_pattern.as_str());
                    let response = server_client.check_multiple_and_join(requests).await?;
                    (response.into(), file_content)
                },
                FileType::Typst | FileType::Markdown | FileType::Html => {
                    let data = match file_type {
                        FileType::Typst => parse_typst(&file_content),
                        FileType::Html => {
                            let text = parse_html(&file_content);
                            Data::from_iter([DataAnnotation::new_text(text)])
                        },
                        FileType::Markdown => parse_markdown(&file_content),
                        _ => unreachable!(),
                    };
                    let response = server_client
                        .check(&request.clone().with_data(data))
                        .await?;
                    (response, file_content)
                },
            };

            if !self.raw {
                writeln!(
                    &mut stdout,
                    "{}",
                    &response.annotate(&text, filename.to_str(), color)
                )?;
            } else {
                writeln!(&mut stdout, "{}", serde_json::to_string_pretty(&response)?)?;
            }
        }

        Ok(())
    }
}

// NOTE: The below structs are copied from `../api/check.rs` to avoid lifetime
// issues with `clap` TODO: Remove these once this upstream issue is resolved: <https://github.com/clap-rs/clap/issues/5773>
// -------------------------------------------------------------------------------------------------

/// LanguageTool POST check request.
///
/// The main feature - check a text with LanguageTool for possible style and
/// grammar issues.
///
/// The structure below tries to follow as closely as possible the JSON API
/// described [here](https://languagetool.org/http-api/swagger-ui/#!/default/post_check).
#[derive(Args, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct CliRequest {
    /// The text to be checked. This or 'data' is required.
    #[clap(short = 't', long, conflicts_with = "data", allow_hyphen_values(true))]
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
    #[clap(short = 'd', long, conflicts_with = "text")]
    pub data: Option<CliData>,
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
            default_value = DEFAULT_LANGUAGE,
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
    pub username: Option<String>,
    /// Set to get Premium API access: your API key (see <https://languagetool.org/editor/settings/api>).
    #[cfg_attr(
        feature = "cli",
        clap(short = 'k', long, requires = "username", env = "LANGUAGETOOL_API_KEY")
    )]
    pub api_key: Option<String>,
    /// Comma-separated list of dictionaries to include words from; uses special
    /// default dictionary if this is unset.
    #[cfg_attr(feature = "cli", clap(long))]
    pub dicts: Option<Vec<String>>,
    /// A language code of the user's native language, enabling false friends
    /// checks for some language pairs.
    #[cfg_attr(feature = "cli", clap(long))]
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
    pub preferred_variants: Option<Vec<String>>,
    /// IDs of rules to be enabled, comma-separated.
    #[cfg_attr(feature = "cli", clap(long))]
    pub enabled_rules: Option<Vec<String>>,
    /// IDs of rules to be disabled, comma-separated.
    #[cfg_attr(feature = "cli", clap(long))]
    pub disabled_rules: Option<Vec<String>>,
    /// IDs of categories to be enabled, comma-separated.
    #[cfg_attr(feature = "cli", clap(long))]
    pub enabled_categories: Option<Vec<String>>,
    /// IDs of categories to be disabled, comma-separated.
    #[cfg_attr(feature = "cli", clap(long))]
    pub disabled_categories: Option<Vec<String>>,
    /// If true, only the rules and categories whose IDs are specified with
    /// `enabledRules` or `enabledCategories` are enabled.
    #[cfg_attr(feature = "cli", clap(long))]
    pub enabled_only: bool,
    /// If set to `picky`, additional rules will be activated, i.e. rules that
    /// you might only find useful when checking formal text.
    #[cfg_attr(
        feature = "cli",
        clap(long, default_value = "default", ignore_case = true, value_enum)
    )]
    pub level: Level,
}

impl From<CliRequest> for Request<'_> {
    fn from(val: CliRequest) -> Self {
        Request {
            text: val.text.map(Cow::Owned),
            data: val.data.map(Into::into),
            language: val.language,
            username: val.username,
            api_key: val.api_key,
            dicts: val.dicts,
            mother_tongue: val.mother_tongue,
            preferred_variants: val.preferred_variants,
            enabled_rules: val.enabled_rules,
            disabled_rules: val.disabled_rules,
            enabled_categories: val.enabled_categories,
            disabled_categories: val.disabled_categories,
            enabled_only: val.enabled_only,
            level: val.level,
        }
    }
}

/// Alternative text to be checked.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct CliData {
    /// Vector of markup text, see [`DataAnnotation`].
    pub annotation: Vec<CliDataAnnotation>,
}

impl From<CliData> for Data<'_> {
    fn from(val: CliData) -> Self {
        Data {
            annotation: val
                .annotation
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<DataAnnotation>>(),
        }
    }
}

/// A portion of text to be checked.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CliDataAnnotation {
    /// Text that should be treated as normal text.
    ///
    /// This or `markup` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Text that should be treated as markup.
    ///
    /// This or `text` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markup: Option<String>,
    /// If set, the markup will be interpreted as this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpret_as: Option<String>,
}

impl From<CliDataAnnotation> for DataAnnotation<'_> {
    fn from(val: CliDataAnnotation) -> Self {
        DataAnnotation {
            text: val.text.map(Cow::Owned),
            markup: val.markup.map(Cow::Owned),
            interpret_as: val.interpret_as.map(Cow::Owned),
        }
    }
}

impl std::str::FromStr for CliData {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let v: Self = serde_json::from_str(s)?;
        Ok(v)
    }
}
