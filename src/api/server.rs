//! Structure to communicate with some `LanguageTool` server through the API.

#[cfg(feature = "multithreaded")]
use crate::api::check;
use crate::{
    api::{
        check::{Request, Response},
        languages, words,
    },
    error::{Error, Result},
};
#[cfg(feature = "cli")]
use clap::Args;
#[cfg(feature = "multithreaded")]
use lifetime::IntoStatic;
use reqwest::{
    header::{HeaderValue, ACCEPT},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io, path::PathBuf, time::Instant};

/// Parse `v` if valid port.
///
/// A valid port is either
/// - an empty string
/// - a 4 chars long string with each char in [0-9]
///
/// # Examples
///
/// ```
/// # use languagetool_rust::api::server::parse_port;
/// assert!(parse_port("8081").is_ok());
///
/// assert!(parse_port("").is_ok()); // No port specified, which is accepted
///
/// assert!(parse_port("abcd").is_err());
/// ```
pub fn parse_port(v: &str) -> Result<String> {
    if v.is_empty() || (v.len() == 4 && v.chars().all(char::is_numeric)) {
        return Ok(v.to_string());
    }
    Err(Error::InvalidValue(
        "The value should be a 4 characters long string with digits only".to_string(),
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A Java property file (one `key = value` entry per line) with values listed
/// below.
pub struct ConfigFile {
    /// Maximum text length, longer texts will cause an error (optional).
    pub max_text_length: Option<isize>,
    /// Maximum text length, applies even to users with a special secret 'token'
    /// parameter (optional).
    pub max_text_hard_length: Option<isize>,
    /// Secret JWT token key, if set by user and valid, maxTextLength can be
    /// increased by the user (optional).
    pub secret_token_key: Option<isize>,
    /// Maximum time in milliseconds allowed per check (optional).
    pub max_check_time_millis: Option<isize>,
    /// Checking will stop with error if there are more rules matches per word
    /// (optional).
    pub max_errors_per_word_rate: Option<isize>,
    /// Only this many spelling errors will have suggestions for performance
    /// reasons (optional, affects Hunspell-based languages only).
    pub max_spelling_suggestions: Option<isize>,
    /// Maximum number of threads working in parallel (optional).
    pub max_check_threads: Option<isize>,
    /// Size of internal cache in number of sentences (optional, default: 0).
    pub cache_size: Option<isize>,
    /// How many seconds sentences are kept in cache (optional, default: 300 if
    /// 'cacheSize' is set).
    pub cache_ttl_seconds: Option<isize>,
    /// Maximum number of requests per requestLimitPeriodInSeconds (optional).
    pub request_limit: Option<isize>,
    /// Maximum aggregated size of requests per requestLimitPeriodInSeconds
    /// (optional).
    pub request_limit_in_bytes: Option<isize>,
    /// Maximum number of timeout request (optional).
    pub timeout_request_limit: Option<isize>,
    /// Time period to which requestLimit and timeoutRequestLimit applies
    /// (optional).
    pub request_limit_period_in_seconds: Option<isize>,
    /// A directory with '1grams', '2grams', '3grams' sub directories which
    /// contain a Lucene index each with ngram occurrence counts; activates the
    /// confusion rule if supported (optional).
    pub language_model: Option<PathBuf>,
    /// A directory with word2vec data (optional), see <https://github.com/languagetool-org/languagetool/blob/master/languagetool-standalone/CHANGES.md#word2vec>.
    pub word2vec_model: Option<PathBuf>,
    /// A model file for better language detection (optional), see
    /// <https://fasttext.cc/docs/en/language-identification.html>.
    pub fasttext_model: Option<PathBuf>,
    /// Compiled fasttext executable for language detection (optional), see
    /// <https://fasttext.cc/docs/en/support.html>.
    pub fasttext_binary: Option<PathBuf>,
    /// Reject request if request queue gets larger than this (optional).
    pub max_work_queue_size: Option<isize>,
    /// A file containing rules configuration, such as .langugagetool.cfg
    /// (optional).
    pub rules_file: Option<PathBuf>,
    /// Set to 'true' to warm up server at start, i.e. run a short check with
    /// all languages (optional).
    pub warm_up: Option<bool>,
    /// A comma-separated list of HTTP referrers (and 'Origin' headers) that are
    /// blocked and will not be served (optional).
    pub blocked_referrers: Option<Vec<String>>,
    /// Activate only the premium rules (optional).
    pub premium_only: Option<bool>,
    /// A comma-separated list of rule ids that are turned off for this server
    /// (optional).
    pub disable_rule_ids: Option<Vec<String>>,
    /// Set to 'true' to enable caching of internal pipelines to improve
    /// performance.
    pub pipeline_caching: Option<bool>,
    /// Cache size if 'pipelineCaching' is set.
    pub max_pipeline_pool_size: Option<isize>,
    /// Time after which pipeline cache items expire.
    pub pipeline_expire_time_in_seconds: Option<isize>,
    /// Set to 'true' to fill pipeline cache on start (can slow down start a
    /// lot).
    pub pipeline_prewarming: Option<bool>,
    /// Spellcheck-only languages: You can add simple spellcheck-only support
    /// for languages that LT doesn't support by defining two optional
    /// properties:
    ///
    /// * 'lang-xx' - set name of the language, use language code instead of
    ///   'xx', e.g. lang-tr=Turkish;
    ///
    /// * 'lang-xx-dictPath' - absolute path to the hunspell .dic file, use
    ///   language code instead of 'xx', e.g. lang-tr-dictPath=/path/to/tr.dic.
    ///   Note that the same directory also needs to contain a common_words.txt
    ///   file with the most common 10,000 words (used for better language
    ///   detection).
    pub spellcheck_only: Option<std::collections::HashMap<String, String>>,
}

impl ConfigFile {
    /// Write the config file in a `key = value` format.
    pub fn write_to<T: io::Write>(&self, w: &mut T) -> io::Result<()> {
        let json = serde_json::to_value(self.clone()).unwrap();
        let m = json.as_object().unwrap();
        for (key, value) in m.iter() {
            match value {
                Value::Bool(b) => writeln!(w, "{key}={b}")?,
                Value::Number(n) => writeln!(w, "{key}={n}")?,
                Value::String(s) => writeln!(w, "{key}=\"{s}\"")?,
                Value::Array(a) => {
                    writeln!(
                        w,
                        "{}=\"{}\"",
                        key,
                        a.iter()
                            .map(std::string::ToString::to_string)
                            .collect::<Vec<String>>()
                            .join(",")
                    )?
                },
                Value::Object(o) => {
                    for (key, value) in o.iter() {
                        writeln!(w, "{key}=\"{value}\"")?
                    }
                },
                Value::Null => writeln!(w, "# {key}=")?,
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

/// Server parameters that are to be used when instantiating a `LanguageTool`
/// server.
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ServerParameters {
    /// A Java property file (one `key = value` entry per line) with values
    /// listed in [`ConfigFile`].
    #[cfg_attr(feature = "cli", clap(long))]
    config: Option<PathBuf>,
    /// Port to bind to, defaults to 8081 if not specified.
    #[cfg_attr(feature = "cli", clap(short = 'p', long, name = "PRT", default_value = "8081", value_parser = parse_port))]
    port: String,
    /// Allow this server process to be connected from anywhere; if not set, it
    /// can only be connected from the computer it was started on.
    #[cfg_attr(feature = "cli", clap(long))]
    public: bool,
    /// set the Access-Control-Allow-Origin header in the HTTP response, used
    /// for direct (non-proxy) JavaScript-based access from browsers. Example: --allow-origin "https://my-website.org".
    /// Don't set a parameter for `*`, i.e. access from all websites.
    #[cfg_attr(feature = "cli", clap(long, name = "ORIGIN"))]
    #[allow(rustdoc::bare_urls)]
    allow_origin: Option<String>,
    /// In case of exceptions, log the input text (up to 500 characters).
    #[cfg_attr(feature = "cli", clap(short = 'v', long))]
    verbose: bool,
    /// A directory with '1grams', '2grams', '3grams' sub directories (per
    /// language) which contain a Lucene index (optional, overwrites
    /// 'languageModel' parameter in properties files).
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(rename = "languageModel")]
    language_model: Option<PathBuf>,
    /// A directory with word2vec data (optional), see <https://github.com/languagetool-org/languagetool/blob/master/languagetool-standalone/CHANGES.md#word2vec>.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(rename = "word2vecModel")]
    word2vec_model: Option<PathBuf>,
    /// Activate the premium rules even when user has no username/password -
    /// useful for API servers.
    #[cfg_attr(feature = "cli", clap(long))]
    #[serde(rename = "premiumAlways")]
    premium_always: bool,
}

impl Default for ServerParameters {
    fn default() -> Self {
        Self {
            config: None,
            port: "8081".to_string(),
            public: false,
            allow_origin: None,
            verbose: false,
            language_model: None,
            word2vec_model: None,
            premium_always: false,
        }
    }
}

/// Hostname and (optional) port to connect to a `LanguageTool` server.
///
/// To use your local server instead of online api, set:
/// * `hostname` to "http://localhost"
/// * `port` to "8081"
///
/// if you used the default configuration to start the server.
#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ServerCli {
    /// Server's hostname.
    #[cfg_attr(
        feature = "cli",
        clap(
            long,
            default_value = "https://api.languagetoolplus.com",
            env = "LANGUAGETOOL_HOSTNAME",
        )
    )]
    pub hostname: String,
    /// Server's port number, with the empty string referring to no specific
    /// port.
    #[cfg_attr(feature = "cli", clap(short = 'p', long, name = "PRT", default_value = "", value_parser = parse_port, env = "LANGUAGETOOL_PORT"))]
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

impl ServerCli {
    /// Create a new [`ServerCli`] instance from environ variables:
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
    /// API string: hostname and, optionally, port number (see [`ServerCli`]).
    pub api: String,
    /// Reqwest client that can send requests to the server.
    pub client: Client,
    max_suggestions: isize,
}

impl From<ServerCli> for ServerClient {
    #[inline]
    fn from(cli: ServerCli) -> Self {
        Self::new(cli.hostname.as_str(), cli.port.as_str())
    }
}

impl ServerClient {
    /// Construct a new server client using hostname and (optional) port
    ///
    /// An empty string is accepted as empty port.
    /// For port validation, please use [`parse_port`] as this constructor does
    /// not check anything.
    #[must_use]
    pub fn new(hostname: &str, port: &str) -> Self {
        let api = if port.is_empty() {
            format!("{hostname}/v2")
        } else {
            format!("{hostname}:{port}/v2")
        };
        let client = Client::new();
        Self {
            api,
            client,
            max_suggestions: -1,
        }
    }

    /// Set the maximum number of suggestions (defaults to -1), a negative
    /// number will keep all replacement suggestions.
    #[must_use]
    pub fn with_max_suggestions(mut self, max_suggestions: isize) -> Self {
        self.max_suggestions = max_suggestions;
        self
    }

    /// Convert a [`ServerCli`] into a proper (usable) client.
    #[must_use]
    pub fn from_cli(cli: ServerCli) -> Self {
        cli.into()
    }

    /// Send a check request to the server and await for the response.
    pub async fn check(&self, request: &Request<'_>) -> Result<Response> {
        let resp = self
            .client
            .post(format!("{0}/check", self.api))
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .form(request)
            .send()
            .await
            .map_err(Error::Reqwest)?;

        match resp.error_for_status_ref() {
            Ok(_) => {
                resp.json::<Response>()
                    .await
                    .map_err(Into::into)
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
                    })
            },
            Err(_) => Err(Error::InvalidRequest(resp.text().await?)),
        }
    }

    /// Send multiple check requests and join them into a single response.
    ///
    /// # Error
    ///
    /// If any of the requests has `self.text` field which is none, or
    /// if zero request is provided.
    #[cfg(feature = "multithreaded")]
    pub async fn check_multiple_and_join<'source>(
        &self,
        requests: Vec<Request<'source>>,
    ) -> Result<check::ResponseWithContext<'source>> {
        use std::borrow::Cow;

        if requests.is_empty() {
            return Err(Error::InvalidRequest(
                "no request; cannot join zero request".to_string(),
            ));
        }

        let tasks = requests
            .into_iter()
            .map(|r| r.into_static())
            .map(|request| {
                let server_client = self.clone();

                tokio::spawn(async move {
                    let response = server_client.check(&request).await?;
                    let text = request.text.ok_or_else(|| {
                        Error::InvalidRequest(
                            "missing text field; cannot join requests with data annotations"
                                .to_string(),
                        )
                    })?;
                    Result::<(Cow<'static, str>, Response)>::Ok((text, response))
                })
            });
      
        let mut response_with_context: Option<check::ResponseWithContext> = None;

        for task in tasks {
            let (text, response) = task.await.unwrap()?;

            response_with_context = Some(match response_with_context {
                Some(resp) => resp.append(check::ResponseWithContext::new(text, response)),
                None => check::ResponseWithContext::new(text, response),
            })
        }

        Ok(response_with_context.unwrap())
    }

    /// Send multiple check requests and join them into a single response,
    /// without any context.
    ///
    /// # Error
    ///
    /// If any of the requests has `self.text` or `self.data` field which is
    /// [`None`].
    #[cfg(feature = "multithreaded")]
    pub async fn check_multiple_and_join_without_context<'source>(
        &self,
        requests: Vec<Request<'source>>,
    ) -> Result<check::Response> {
        let mut response: Option<check::Response> = None;

        let tasks = requests
            .into_iter()
            .map(|r| r.into_static())
            .map(|request| {
                let server_client = self.clone();

                tokio::spawn(async move {
                    let response = server_client.check(&request).await?;
                    Result::<Response>::Ok(response)
                })
            });

        // Make requests in sequence
        for task in tasks {
            let resp = task.await.unwrap()?;

            response = Some(match response {
                Some(r) => r.append(resp),
                None => resp,
            })
        }

        Ok(response.unwrap())
    }

    /// Send a check request to the server, await for the response and annotate
    /// it.
    #[cfg(feature = "annotate")]
    pub async fn annotate_check(
        &self,
        request: &Request<'_>,
        origin: Option<&str>,
        color: bool,
    ) -> Result<String> {
        let text = request.get_text();
        let resp = self.check(request).await?;

        Ok(resp.annotate(text.as_ref(), origin, color))
    }

    /// Send a languages request to the server and await for the response.
    pub async fn languages(&self) -> Result<languages::Response> {
        let resp = self
            .client
            .get(format!("{}/languages", self.api))
            .send()
            .await
            .map_err(Error::Reqwest)?;

        match resp.error_for_status_ref() {
            Ok(_) => resp.json::<languages::Response>().await.map_err(Into::into),
            Err(_) => Err(Error::InvalidRequest(resp.text().await?)),
        }
    }

    /// Send a words request to the server and await for the response.
    pub async fn words(&self, request: &words::Request) -> Result<words::Response> {
        let resp = self
            .client
            .get(format!("{}/words", self.api))
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .query(request)
            .send()
            .await
            .map_err(Error::Reqwest)?;

        match resp.error_for_status_ref() {
            Ok(_) => resp.json::<words::Response>().await.map_err(Error::Reqwest),
            Err(_) => Err(Error::InvalidRequest(resp.text().await?)),
        }
    }

    /// Send a words/add request to the server and await for the response.
    pub async fn words_add(&self, request: &words::add::Request) -> Result<words::add::Response> {
        let resp = self
            .client
            .post(format!("{}/words/add", self.api))
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .form(request)
            .send()
            .await
            .map_err(Error::Reqwest)?;

        match resp.error_for_status_ref() {
            Ok(_) => {
                resp.json::<words::add::Response>()
                    .await
                    .map_err(Error::Reqwest)
            },
            Err(_) => Err(Error::InvalidRequest(resp.text().await?)),
        }
    }

    /// Send a words/delete request to the server and await for the response.
    pub async fn words_delete(
        &self,
        request: &words::delete::Request,
    ) -> Result<words::delete::Response> {
        let resp = self
            .client
            .post(format!("{}/words/delete", self.api))
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .form(request)
            .send()
            .await
            .map_err(Error::Reqwest)?;

        match resp.error_for_status_ref() {
            Ok(_) => {
                resp.json::<words::delete::Response>()
                    .await
                    .map_err(Error::Reqwest)
            },
            Err(_) => Err(Error::InvalidRequest(resp.text().await?)),
        }
    }

    /// Ping the server and return the elapsed time in milliseconds if the
    /// server responded.
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
    /// but defaults to [`ServerClient::default`] if expected environ
    /// variables are not set.
    #[must_use]
    pub fn from_env_or_default() -> Self {
        Self::from_cli(ServerCli::from_env_or_default())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::ServerClient;
    use crate::{api::check::Request, error::Error};

    fn dbg_err(e: &Error) {
        eprintln!("Error: {e:?}")
    }

    #[tokio::test]
    async fn test_server_ping() {
        let client = ServerClient::from_env_or_default();
        assert!(client.ping().await.inspect_err(dbg_err).is_ok());
    }

    #[tokio::test]
    async fn test_server_check_text() {
        let client = ServerClient::from_env_or_default();

        let req = Request::default().with_text("je suis une poupee");
        assert!(client.check(&req).await.inspect_err(dbg_err).is_ok());

        // Too long
        let req = Request::default().with_text("Repeat ".repeat(1500));
        assert_matches!(client.check(&req).await, Err(Error::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_server_check_data() {
        let client = ServerClient::from_env_or_default();
        let req = Request::default()
            .with_data_str("{\"annotation\":[{\"text\": \"je suis une poupee\"}]}")
            .unwrap();
        assert!(client.check(&req).await.inspect_err(dbg_err).is_ok());

        // Too long
        let req = Request::default()
            .with_data_str(&format!(
                "{{\"annotation\":[{{\"text\": \"{}\"}}]}}",
                "repeat".repeat(5000)
            ))
            .unwrap();
        assert_matches!(client.check(&req).await, Err(Error::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_server_languages() {
        let client = ServerClient::from_env_or_default();
        assert!(client.languages().await.inspect_err(dbg_err).is_ok());
    }
}
