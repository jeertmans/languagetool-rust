use crate::check::{CheckRequest, CheckResponse};
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

type RequestResult<T> = Result<T, reqwest::Error>;

/// Check if `v` is a valid port.
///
/// A valid port is either
/// - an empty string
/// - a 4 chars long string with each char in [0-9]
pub fn is_port(v: &str) -> Result<(), String> {
    if v.is_empty() || (v.len() == 4 && v.chars().all(char::is_numeric)) {
        return Ok(());
    }
    Err(String::from(
        "The value should be a 4 characters long string with digits only",
    ))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
    max_text_length: Option<isize>,
    max_text_hard_length: Option<isize>,
    secret_token_key: Option<isize>,
    max_check_time_millis: Option<isize>,
    max_errors_per_word_rate: Option<isize>,
    max_spelling_suggestions: Option<isize>,
    max_check_threads: Option<isize>,
    cache_size: Option<isize>,
    cache_ttl_seconds: Option<isize>,
    request_limit: Option<isize>,
    request_limit_in_bytes: Option<isize>,
    timeout_request_limit: Option<isize>,
    request_limit_period_in_seconds: Option<isize>,
    language_model: Option<PathBuf>,
    word2vec_model: Option<PathBuf>,
    fasttext_model: Option<PathBuf>,
    fasttext_binary: Option<PathBuf>,
    max_work_queue_size: Option<PathBuf>,
    rules_file: Option<PathBuf>,
    warm_up: Option<bool>,
    blocked_referrers: Option<Vec<String>>,
    premium_only: Option<bool>,
    disable_rule_ids: Option<Vec<String>>,
    pipeline_caching: Option<bool>,
    max_pipeline_pool_size: Option<isize>,
    pipeline_expire_time_in_seconds: Option<isize>,
    pipeline_prewarming: Option<bool>,
    // TODO:
    // support lang-xx, lang-xx-dictPath
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
                    "{}={:?}",
                    key,
                    a.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>()
                )?,
                Value::Null => writeln!(w, "# {}=", key)?,
                _ => unreachable!(), // Cannot be a Value::Object
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
pub struct ServerCli {
    #[cfg_attr(feature = "cli", clap(long, default_value = "http://localhost"))]
    pub hostname: String,
    #[cfg_attr(feature = "cli", clap(short = 'p', long, name = "PRT", default_value = "8081", validator = is_port))]
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
    pub fn from_arg_matches(matches: &clap::ArgMatches) -> clap::Result<Self, clap::Error> {
        let params = ServerCli::from_arg_matches(&matches)?;
        Ok(Self::from_cli(params))
    }

    #[cfg(feature = "cli")]
    pub fn command<'help>() -> clap::Command<'help> {
        ServerCli::command()
    }

    pub async fn check(&self, request: &CheckRequest) -> RequestResult<CheckResponse> {
        match self
            .client
            .post(format!("{}/check", self.api))
            .query(request)
            .send()
            .await
        {
            Ok(resp) => resp.json::<CheckResponse>().await,
            Err(e) => Err(e),
        }
    }

    pub async fn languages(&self) -> RequestResult<LanguagesResponse> {
        match self
            .client
            .get(format!("{}/languages", self.api))
            .send()
            .await
        {
            Ok(resp) => resp.json::<LanguagesResponse>().await,
            Err(e) => Err(e),
        }
    }

    pub async fn words(&self, request: &WordsRequest) -> RequestResult<WordsResponse> {
        match self
            .client
            .get(format!("{}/words", self.api))
            .query(request)
            .send()
            .await
        {
            Ok(resp) => resp.json::<WordsResponse>().await,
            Err(e) => Err(e),
        }
    }

    pub async fn words_add(&self, request: &WordsAddRequest) -> RequestResult<WordsAddResponse> {
        match self
            .client
            .post(format!("{}/words/add", self.api))
            .query(request)
            .send()
            .await
        {
            Ok(resp) => resp.json::<WordsAddResponse>().await,
            Err(e) => Err(e),
        }
    }

    pub async fn words_delete(
        &self,
        request: &WordsDeleteRequest,
    ) -> RequestResult<WordsDeleteResponse> {
        match self
            .client
            .post(format!("{}/words/delete", self.api))
            .query(request)
            .send()
            .await
        {
            Ok(resp) => resp.json::<WordsDeleteResponse>().await,
            Err(e) => Err(e),
        }
    }

    pub async fn ping(&self) -> RequestResult<u128> {
        let start = Instant::now();
        match self.client.get(&self.api).send().await {
            Ok(_) => Ok((Instant::now() - start).as_millis()),
            Err(e) => Err(e),
        }
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
