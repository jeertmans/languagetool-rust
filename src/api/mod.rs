//! Raw bindings to the LanguageTool API v1.1.2.
//!
//! The current bindings were generated using the
//! [HTTP API documentation](https://languagetool.org/http-api/).
//!
//! Unfortunately, the LanguageTool API is not as documented as we could
//! hope, and requests might return undocumented fields. Those are de-serialized
//! to the `undocumented` field.
pub mod check;
pub mod languages;
pub mod server;
pub mod words;

use crate::error::{Error, Result};

/// A HTTP client for making requests to a LanguageTool server.
#[derive(Debug)]
pub struct Client {
    /// Server's hostname.
    hostname: String,
    /// Server's port.
    port: Option<String>,
    /// Inner client to perform HTTP requests.
    client: reqwest::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            hostname: "https://api.languagetoolplus.com".to_string(),
            port: None,
            client: Default::default(),
        }
    }
}

impl Client {
    /// Construct an HTTP URL base on the current hostname, optional port,
    /// and provided endpoint.
    #[inline]
    #[must_use]
    pub fn url(&self, endpoint: &str) -> String {
        let hostname = &self.hostname;
        match &self.port {
            Some(p) => format!("{hostname}:{p}/v2{endpoint}"),
            None => format!("{hostname}/v2{endpoint}"),
        }
    }

    /// Send a check request to the server and await for the response.
    pub async fn check(&self, request: &check::Request) -> Result<check::Response> {
        self.client
            .post(self.url("/check"))
            .query(request)
            .send()
            .await
            .map_err(Error::RequestEncode)?
            .json::<check::Response>()
            .await
            .map_err(Error::ResponseDecode)
    }

    /// Send a request for the list of supported languages to the server and
    /// await for the response.
    pub async fn languages(&self) -> reqwest::Result<languages::Response> {
        self.client
            .get(self.url("/languages"))
            .send()
            .await?
            .json::<languages::Response>()
            .await
    }
}
