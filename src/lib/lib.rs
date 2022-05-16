pub mod check;
pub mod languages;
pub mod words;

pub use crate::check::{CheckRequest, CheckResponse};
pub use crate::languages::LanguagesResponse;
#[cfg(feature = "cli")]
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use std::path::PathBuf;

type RequestResult<T> = Result<T, reqwest::Error>;

pub fn is_port(v: &str) -> Result<(), String> {
    if v.len() == 4 && v.chars().all(char::is_numeric) {
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
    hostname: String,
    #[cfg_attr(feature = "cli", clap(short = 'p', long, name = "PRT", default_value = "8081", validator = is_port))]
    port: String,
}

#[derive(Debug)]
pub struct Server {
    api: String,
    client: Client,
}

impl Server {
    pub fn new(hostname: String, port: String) -> Self {
        let api = format!("{}:{}/v2", hostname, port);
        let client = Client::new();
        Self { api, client }
    }

    #[cfg(feature = "cli")]
    pub fn from_cli() -> Self {
        let server = ServerCli::parse();
        Self::new(server.hostname, server.port)
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
    pub fn words(&self) {}
    pub fn add_words(&self) {}
    pub fn delete_words(&self) {}
}

#[cfg(test)]
mod tests {
    use crate::ConfigFile;

    #[test]
    fn test_write_config_file() {
        let mut stdout = std::io::stdout();
        let config = ConfigFile::default();
        config.write_to(&mut stdout).unwrap();
    }
}
