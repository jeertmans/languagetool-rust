use crate::check::{CheckRequest, CheckResponse};
use crate::error::{Error, Result};
use crate::languages::LanguagesResponse;
use crate::words::{
    WordsAddRequest, WordsAddResponse, WordsDeleteRequest, WordsDeleteResponse, WordsRequest,
    WordsResponse,
};
#[cfg(feature = "cli")]
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
/// assert!(is_port("8081").is_ok())
///
/// assert!(is_port("").is_ok())  # No port specified, which is accepted
///
/// assert!(is_port("abcd").is_err())
/// ```
pub fn is_port(v: &str) -> Result<()> {
    if v.is_empty() || (v.len() == 4 && v.chars().all(char::is_numeric)) {
        return Ok(());
    }
    Err(Error::InvalidValue {
        body: "The value should be a 4 characters long string with digits only".to_string(),
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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
    /// A directory with word2vec data (optional), see https://github.com/languagetool-org/languagetool/blob/master/languagetool-standalone/CHANGES.md#word2vec
    pub word2vec_model: Option<PathBuf>,
    /// A model file for better language detection (optional), see
    /// https://fasttext.cc/docs/en/language-identification.html
    pub fasttext_model: Option<PathBuf>,
    /// Compiled fasttext executable for language detection (optional), see
    /// https://fasttext.cc/docs/en/support.html
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
                        .map(|v| v.to_string())
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

#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerParameters {
    #[cfg_attr(feature = "cli", clap(long))]
    config: Option<PathBuf>,
    #[cfg_attr(feature = "cli", clap(short = 'p', long, name = "PRT", default_value = "8081", validator = is_port))]
    port: String,
    #[cfg_attr(feature = "cli", clap(long, takes_value = false))]
    public: bool,
    #[cfg_attr(feature = "cli", clap(long, name = "ORIGIN"))]
    allow_origin: Option<String>,
    #[cfg_attr(feature = "cli", clap(short = 'v', long, takes_value = false))]
    verbose: bool,
    #[cfg_attr(feature = "cli", clap(long, takes_value = false))]
    #[serde(rename = "languageModel")]
    language_model: Option<PathBuf>,
    #[cfg_attr(feature = "cli", clap(long, takes_value = false))]
    #[serde(rename = "word2vecModel")]
    word2vec_model: Option<PathBuf>,
    #[cfg_attr(feature = "cli", clap(long, takes_value = false))]
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

#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Debug)]
/// Hostname and (optional) port to connect to a LanguageTool server.
///
/// To use your local server instead of online api, set:
/// - `hostname` to "http://localhost"
/// - `port` to "8081"
/// if you used the default configuration to start the server.
pub struct ServerCli {
    #[cfg_attr(
        feature = "cli",
        clap(long, default_value = "https://api.languagetoolplus.com")
    )]
    pub hostname: String,
    #[cfg_attr(feature = "cli", clap(short = 'p', long, name = "PRT", default_value = "", validator = is_port))]
    pub port: String,
}

impl Default for ServerCli {
    fn default() -> Self {
        Self {
            hostname: "https://api.languagetoolplus.com".to_string(),
            port: "".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ServerClient {
    api: String,
    client: Client,
}

impl ServerClient {
    pub fn new(hostname: String, port: String) -> Self {
        let api = if port.is_empty() {
            format!("{}/v2", hostname)
        } else {
            format!("{}:{}/v2", hostname, port)
        };
        let client = Client::new();
        Self { api, client }
    }

    pub fn from_cli(cli: ServerCli) -> Self {
        Self::new(cli.hostname, cli.port)
    }

    #[cfg(feature = "cli")]
    pub fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self> {
        let params = ServerCli::from_arg_matches(matches)?;
        Ok(Self::from_cli(params))
    }

    #[cfg(feature = "cli")]
    pub fn command<'help>() -> clap::Command<'help> {
        ServerCli::command()
    }

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
                    .map_err(|e| Error::ResponseDecode { source: e }),
                Err(_) => Err(Error::InvalidRequest {
                    body: resp.text().await?,
                }),
            },
            Err(e) => Err(Error::RequestEncode { source: e }),
        }
    }

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

#[cfg(test)]
mod tests {
    use crate::check::CheckRequest;
    use crate::ServerClient;

    #[tokio::test]
    async fn test_server_ping() {
        let server = ServerClient::default();
        assert!(server.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_server_check_text() {
        let server = ServerClient::default();
        let req = CheckRequest::default()
            .with_language("auto")
            .with_text("je suis une poupee");
        assert!(server.check(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_server_check_data() {
        let server = ServerClient::default();
        let req = CheckRequest::default()
            .with_language("auto")
            .with_data("{\"annotation\":[{\"text\": \"je suis une poupee\"}]}")
            .unwrap();
        assert!(server.check(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_server_languages() {
        let server = ServerClient::default();
        assert!(server.languages().await.is_ok());
    }
}
