//! Structure to communite with some `LanguageTool` server through the API.

use crate::check::{CheckRequest, CheckResponse};
use crate::error::{Error, Result};
use crate::languages::LanguagesResponse;
use crate::words::{
    WordsAddRequest, WordsAddResponse, WordsDeleteRequest, WordsDeleteResponse, WordsRequest,
    WordsResponse,
};
#[cfg(feature = "annotate")]
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
#[cfg(feature = "clap")]
use clap::{CommandFactory, FromArgMatches, Parser};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

/// Check if `v` is a valid port.
///
/// A valid port is either
/// - an empty string
/// - a 4 chars long string with each char in [0-9]
///
/// # Examples
///
/// ```
/// # use languagetool_rust::server::is_port;
/// assert!(is_port("8081").is_ok());
///
/// assert!(is_port("").is_ok());  // No port specified, which is accepted
///
/// assert!(is_port("abcd").is_err());
/// ```
pub fn is_port(v: &str) -> Result<()> {
    if v.is_empty() || (v.len() == 4 && v.chars().all(char::is_numeric)) {
        return Ok(());
    }
    Err(Error::InvalidValue {
        body: "The value should be a 4 characters long string with digits only".to_owned(),
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A Java property file (one key=value entry per line) with values listed below.
pub struct ConfigFile {
    /// Maximum text length, longer texts will cause an error (optional)
    pub max_text_length: Option<isize>,
    /// Maximum text length, applies even to users with a special secret 'token' parameter (optional)
    pub max_text_hard_length: Option<isize>,
    /// Secret JWT token key, if set by user and valid, maxTextLength can be increased by the user (optional)
    pub secret_token_key: Option<isize>,
    /// Maximum time in milliseconds allowed per check (optional)
    pub max_check_time_millis: Option<isize>,
    /// Checking will stop with error if there are more rules matches per word (optional)
    pub max_errors_per_word_rate: Option<isize>,
    /// Only this many spelling errors will have suggestions for performance reasons (optional, affects Hunspell-based languages only)
    pub max_spelling_suggestions: Option<isize>,
    /// Maximum number of threads working in parallel (optional)
    pub max_check_threads: Option<isize>,
    /// Size of internal cache in number of sentences (optional, default: 0)
    pub cache_size: Option<isize>,
    /// How many seconds sentences are kept in cache (optional, default: 300 if 'cacheSize' is set)
    pub cache_ttl_seconds: Option<isize>,
    /// Maximum number of requests per requestLimitPeriodInSeconds (optional)
    pub request_limit: Option<isize>,
    /// Maximum aggregated size of requests per requestLimitPeriodInSeconds (optional)
    pub request_limit_in_bytes: Option<isize>,
    /// Maximum number of timeout request (optional)
    pub timeout_request_limit: Option<isize>,
    /// Time period to which requestLimit and timeoutRequestLimit applies (optional)
    pub request_limit_period_in_seconds: Option<isize>,
    /// A directory with '1grams', '2grams', '3grams' sub directories which contain a Lucene index each with ngram occurrence counts; activates the confusion rule if supported (optional)
    pub language_model: Option<PathBuf>,
    /// A directory with word2vec data (optional), see <https://github.com/languagetool-org/languagetool/blob/master/languagetool-standalone/CHANGES.md#word2vec>
    pub word2vec_model: Option<PathBuf>,
    /// A model file for better language detection (optional), see
    /// <https://fasttext.cc/docs/en/language-identification.html>
    pub fasttext_model: Option<PathBuf>,
    /// Compiled fasttext executable for language detection (optional), see
    /// <https://fasttext.cc/docs/en/support.html>
    pub fasttext_binary: Option<PathBuf>,
    /// Reject request if request queue gets larger than this (optional)
    pub max_work_queue_size: Option<isize>,
    /// A file containing rules configuration, such as .langugagetool.cfg (optional)
    pub rules_file: Option<PathBuf>,
    /// Set to 'true' to warm up server at start, i.e. run a short check with all languages (optional)
    pub warm_up: Option<bool>,
    /// A comma-separated list of HTTP referrers (and 'Origin' headers) that are blocked and will not be served (optional)
    pub blocked_referrers: Option<Vec<String>>,
    /// Activate only the premium rules (optional)
    pub premium_only: Option<bool>,
    /// A comma-separated list of rule ids that are turned off for this server (optional)
    pub disable_rule_ids: Option<Vec<String>>,
    /// Set to 'true' to enable caching of internal pipelines to improve performance
    pub pipeline_caching: Option<bool>,
    /// Cache size if 'pipelineCaching' is set
    pub max_pipeline_pool_size: Option<isize>,
    /// Time after which pipeline cache items expire
    pub pipeline_expire_time_in_seconds: Option<isize>,
    /// Set to 'true' to fill pipeline cache on start (can slow down start a lot)
    pub pipeline_prewarming: Option<bool>,
    /// Spellcheck-only languages: You can add simple spellcheck-only support for languages that LT
    /// doesn't support by defining two optional properties:
    ///     'lang-xx' - set name of the language, use language code instead of 'xx', e.g. lang-tr=Turkish
    ///     'lang-xx-dictPath' - absolute path to the hunspell .dic file, use language code instead of 'xx', e.g. lang-tr-dictPath=/path/to/tr.dic. Note that the same directory also needs to
    pub spellcheck_only: Option<std::collections::HashMap<String, String>>,
}

impl ConfigFile {
    /// Write the config file in a `key = value` format
    pub fn write_to<T: io::Write>(&self, w: &mut T) -> io::Result<()> {
        let json = serde_json::to_value(self.clone()).unwrap();
        let m = json.as_object().unwrap();
        for (key, value) in m.iter() {
            match value {
                Value::Bool(b) => writeln!(w, "{}={}", key, b)?,
                Value::Number(n) => writeln!(w, "{}={}", key, n)?,
                Value::String(s) => writeln!(w, "{}=\"{}\"", key, s)?,
                Value::Array(a) => writeln!(
                    w,
                    "{}=\"{}\"",
                    key,
                    a.iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(",")
                )?,
                Value::Object(o) => {
                    for (key, value) in o.iter() {
                        writeln!(w, "{}=\"{}\"", key, value)?
                    }
                }
                Value::Null => writeln!(w, "# {}=", key)?,
            }
        }
        Ok(())
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            max_text_length: None,
            max_text_hard_length: None,
            secret_token_key: None,
            max_check_time_millis: None,
            max_errors_per_word_rate: None,
            max_spelling_suggestions: None,
            max_check_threads: None,
            cache_size: Some(0),
            cache_ttl_seconds: Some(300),
            request_limit: None,
            request_limit_in_bytes: None,
            timeout_request_limit: None,
            request_limit_period_in_seconds: None,
            language_model: None,
            word2vec_model: None,
            fasttext_model: None,
            fasttext_binary: None,
            max_work_queue_size: None,
            rules_file: None,
            warm_up: None,
            blocked_referrers: None,
            premium_only: None,
            disable_rule_ids: None,
            pipeline_caching: None,
            max_pipeline_pool_size: None,
            pipeline_expire_time_in_seconds: None,
            pipeline_prewarming: None,
            spellcheck_only: None,
        }
    }
}

#[cfg_attr(feature = "clap", derive(Parser))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
/// Server parameters that are to be used when instantiating a `LanguageTool` server
pub struct ServerParameters {
    #[cfg_attr(feature = "clap", clap(long))]
    config: Option<PathBuf>,
    #[cfg_attr(feature = "clap", clap(short = 'p', long, name = "PRT", default_value = "8081", validator = is_port))]
    port: String,
    #[cfg_attr(feature = "clap", clap(long, takes_value = false))]
    public: bool,
    #[cfg_attr(feature = "clap", clap(long, name = "ORIGIN"))]
    allow_origin: Option<String>,
    #[cfg_attr(feature = "clap", clap(short = 'v', long, takes_value = false))]
    verbose: bool,
    #[cfg_attr(feature = "clap", clap(long, takes_value = false))]
    #[serde(rename = "languageModel")]
    language_model: Option<PathBuf>,
    #[cfg_attr(feature = "clap", clap(long, takes_value = false))]
    #[serde(rename = "word2vecModel")]
    word2vec_model: Option<PathBuf>,
    #[cfg_attr(feature = "clap", clap(long, takes_value = false))]
    #[serde(rename = "premiumAlways")]
    premium_always: bool,
}

impl Default for ServerParameters {
    fn default() -> Self {
        Self {
            config: None,
            port: String::from("8081"),
            public: false,
            allow_origin: None,
            verbose: false,
            language_model: None,
            word2vec_model: None,
            premium_always: false,
        }
    }
}

#[cfg_attr(feature = "clap", derive(Parser))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
/// Hostname and (optional) port to connect to a `LanguageTool` server.
///
/// To use your local server instead of online api, set:
/// - `hostname` to "http://localhost"
/// - `port` to "8081"
/// if you used the default configuration to start the server.
pub struct ServerCli {
    /// Server's hostname
    #[cfg_attr(
        feature = "cli",
        clap(
            long,
            default_value = "https://api.languagetoolplus.com",
            env = "LANGUAGETOOL_HOSTNAME",
        )
    )]
    pub hostname: String,
    /// Server's port number, with the empty string referring to no specific port
    #[cfg_attr(feature = "clap", clap(short = 'p', long, name = "PRT", default_value = "", validator = is_port, env = "LANGUAGETOOL_PORT"))]
    pub port: String,
}

impl Default for ServerCli {
    fn default() -> Self {
        Self {
            hostname: "https://api.languagetoolplus.com".to_owned(),
            port: "".to_owned(),
        }
    }
}

impl ServerCli {
    /// Create a new [`ServeCli`] instance from environ variables:
    /// - `LANGUAGETOOL_HOSTNAME`
    /// - `LANGUAGETOOL_PORT`
    ///
    /// If one or both environ variables are empty, an error is returned.
    pub fn from_env() -> Result<Self> {
        let hostname = std::env::var("LANGUAGETOOL_HOSTNAME")?;
        let port = std::env::var("LANGUAGETOOL_PORT")?;

        Ok(Self { hostname, port })
    }

    /// Create a new [`ServerCli`] instance from environ variables,
    /// but defaults to [`ServerCli::default`()] if expected environ
    /// variables are not set.
    #[must_use]
    pub fn from_env_or_default() -> Self {
        ServerCli::from_env().unwrap_or_default()
    }
}

/// Client to communicate with the `LanguageTool` server using async requests.
#[derive(Clone, Debug)]
pub struct ServerClient {
    /// API string: hostname and, optionally, port number (see [ServerCli])
    pub api: String,
    /// Reqwest client that can send requests to the server
    pub client: Client,
    max_suggestions: isize,
}

impl From<ServerCli> for ServerClient {
    #[inline]
    fn from(cli: ServerCli) -> Self {
        Self::new(&cli.hostname[..], &cli.port[..])
    }
}

impl ServerClient {
    /// Construct a new server client using hostname and (optional) port
    ///
    /// An empty string is accepeted as empty port.
    /// For port validation, please use [`is_port`] as this constructor does not check anything.
    pub fn new(hostname: &str, port: &str) -> Self {
        let api = if port.is_empty() {
            format!("{}/v2", hostname)
        } else {
            format!("{}:{}/v2", hostname, port)
        };
        let client = Client::new();
        Self {
            api,
            client,
            max_suggestions: -1,
        }
    }

    /// Sets the maximum number of suggestions (defaults to -1), a negative number will keep all
    /// replacement suggestions
    pub fn with_max_suggestions(mut self, max_suggestions: isize) -> Self {
        self.max_suggestions = max_suggestions;
        self
    }

    /// Converts a [`ServerCli`] into a proper (usable) client
    pub fn from_cli(cli: ServerCli) -> Self {
        cli.into()
    }

    #[cfg(feature = "clap")]
    /// This function has the same sementics as [`ServerCli::from_arg_matches`]
    pub fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self> {
        let params = ServerCli::from_arg_matches(matches)?;
        Ok(Self::from_cli(params))
    }

    /// This function has the same semantics as [`ServerCli::command`]
    #[cfg(feature = "clap")]
    pub fn command<'help>() -> clap::Command<'help> {
        ServerCli::command()
    }

    /// Send a check request to the server and await for the response
    pub async fn check(&self, request: &CheckRequest) -> Result<CheckResponse> {
        match self
            .client
            .post(format!("{}/check", self.api))
            .query(request)
            .send()
            .await
        {
            Ok(resp) => match resp.error_for_status_ref() {
                Ok(_) => resp
                    .json::<CheckResponse>()
                    .await
                    .map_err(|e| Error::ResponseDecode { source: e })
                    .map(|mut resp| {
                        if self.max_suggestions > 0 {
                            let max = self.max_suggestions as usize;
                            resp.matches.iter_mut().for_each(|m| {
                                let len = m.replacements.len();
                                if max < len {
                                    m.replacements[max] =
                                        format!("... ({} not shown)", len - max).into();
                                    m.replacements.truncate(max + 1);
                                }
                            });
                        }
                        resp
                    }),
                Err(_) => Err(Error::InvalidRequest {
                    body: resp.text().await?,
                }),
            },
            Err(e) => Err(Error::RequestEncode { source: e }),
        }
    }

    /// Send a check request to the server, await for the response and annotate it
    #[cfg(feature = "annotate")]
    pub async fn annotate_check(&self, request: &CheckRequest) -> Result<String> {
        let text = request.get_text();
        let resp = self.check(request).await?;

        if resp.matches.is_empty() {
            return Ok("No error were found in provided text".to_owned());
        }
        let replacements: Vec<_> = resp
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

        let snippets = resp
            .matches
            .iter()
            .zip(replacements.iter())
            .map(|(m, r)| Snippet {
                title: Some(Annotation {
                    label: Some(&m.message),
                    id: Some(&m.rule.id),
                    annotation_type: AnnotationType::Error,
                }),
                footer: vec![],
                slices: vec![Slice {
                    source: &m.context.text,
                    line_start: 1 + text.chars().take(m.offset).filter(|c| *c == '\n').count(),
                    origin: None,
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
                    color: true,
                    ..Default::default()
                },
            });

        let mut annotation = String::new();

        for snippet in snippets {
            if !annotation.is_empty() {
                annotation.push('\n');
            }
            annotation.push_str(&DisplayList::from(snippet).to_string());
        }
        Ok(annotation)
    }

    /// Send a languages request to the server and await for the response
    pub async fn languages(&self) -> Result<LanguagesResponse> {
        match self
            .client
            .get(format!("{}/languages", self.api))
            .send()
            .await
        {
            Ok(resp) => match resp.error_for_status_ref() {
                Ok(_) => resp
                    .json::<LanguagesResponse>()
                    .await
                    .map_err(|e| Error::ResponseDecode { source: e }),
                Err(_) => Err(Error::InvalidRequest {
                    body: resp.text().await?,
                }),
            },
            Err(e) => Err(Error::RequestEncode { source: e }),
        }
    }

    /// Send a words request to the server and await for the response
    pub async fn words(&self, request: &WordsRequest) -> Result<WordsResponse> {
        match self
            .client
            .get(format!("{}/words", self.api))
            .query(request)
            .send()
            .await
        {
            Ok(resp) => match resp.error_for_status_ref() {
                Ok(_) => resp
                    .json::<WordsResponse>()
                    .await
                    .map_err(|e| Error::ResponseDecode { source: e }),
                Err(_) => Err(Error::InvalidRequest {
                    body: resp.text().await?,
                }),
            },
            Err(e) => Err(Error::RequestEncode { source: e }),
        }
    }

    /// Send a words/add request to the server and await for the response
    pub async fn words_add(&self, request: &WordsAddRequest) -> Result<WordsAddResponse> {
        match self
            .client
            .post(format!("{}/words/add", self.api))
            .query(request)
            .send()
            .await
        {
            Ok(resp) => match resp.error_for_status_ref() {
                Ok(_) => resp
                    .json::<WordsAddResponse>()
                    .await
                    .map_err(|e| Error::ResponseDecode { source: e }),
                Err(_) => Err(Error::InvalidRequest {
                    body: resp.text().await?,
                }),
            },
            Err(e) => Err(Error::RequestEncode { source: e }),
        }
    }

    /// Send a words/delete request to the server and await for the response
    pub async fn words_delete(&self, request: &WordsDeleteRequest) -> Result<WordsDeleteResponse> {
        match self
            .client
            .post(format!("{}/words/delete", self.api))
            .query(request)
            .send()
            .await
        {
            Ok(resp) => match resp.error_for_status_ref() {
                Ok(_) => resp
                    .json::<WordsDeleteResponse>()
                    .await
                    .map_err(|e| Error::ResponseDecode { source: e }),
                Err(_) => Err(Error::InvalidRequest {
                    body: resp.text().await?,
                }),
            },
            Err(e) => Err(Error::RequestEncode { source: e }),
        }
    }

    /// Ping the server and return the elapsed time in milliseconds if the server responded
    pub async fn ping(&self) -> Result<u128> {
        let start = Instant::now();
        self.client.get(&self.api).send().await?;
        Ok((Instant::now() - start).as_millis())
    }
}

impl Default for ServerClient {
    fn default() -> Self {
        Self::from_cli(ServerCli::default())
    }
}

impl ServerClient {
    /// Create a new [`ServerClient`] instance from environ variables.
    ///
    /// See [`ServerCli::from_env`] for more details.
    pub fn from_env() -> Result<Self> {
        Ok(Self::from_cli(ServerCli::from_env()?))
    }

    /// Create a new [`ServerClient`] instance from environ variables,
    /// but defaults to [`ServerClient::default`()] if expected environ
    /// variables are not set.
    pub fn from_env_or_default() -> Self {
        Self::from_cli(ServerCli::from_env_or_default())
    }
}

#[cfg(test)]
mod tests {
    use crate::check::CheckRequest;
    use crate::ServerClient;

    #[tokio::test]
    async fn test_server_ping() {
        let client = ServerClient::from_env_or_default();
        assert!(client.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_server_check_text() {
        let client = ServerClient::from_env_or_default();
        let req = CheckRequest::default().with_text("je suis une poupee".to_owned());
        assert!(client.check(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_server_check_data() {
        let client = ServerClient::from_env_or_default();
        let req = CheckRequest::default()
            .with_data_str("{\"annotation\":[{\"text\": \"je suis une poupee\"}]}")
            .unwrap();
        assert!(client.check(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_server_languages() {
        let client = ServerClient::from_env_or_default();
        assert!(client.languages().await.is_ok());
    }
}
