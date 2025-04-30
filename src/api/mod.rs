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

use crate::error::Result;

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
    /// Construct an HTTP URL based on the current hostname, optional port,
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
    pub async fn check(&self, request: &check::Request<'_>) -> Result<check::Response> {
        self.client
            .post(self.url("/check"))
            .query(request)
            .send()
            .await?
            .json::<check::Response>()
            .await
            .map_err(Into::into)
    }

    /// Send a request for the list of supported languages to the server and
    /// await for the response.
    pub async fn languages(&self) -> Result<languages::Response> {
        self.client
            .get(self.url("/languages"))
            .send()
            .await?
            .json::<languages::Response>()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    use reqwest::Url;

    use super::*;

    fn get_testing_client() -> Client {
        Client {
            hostname: "http://localhost".into(),
            port: Some("8010".into()),
            ..Default::default()
        }
    }

    #[test]
    fn test_url() {
        // Without port
        let client = Client::default();
        let url = client.url("/endpoint");
        assert!(url.contains(&client.hostname));
        assert!(!url.contains(":80"));
        assert!(url.ends_with("/endpoint"));
        assert!(Url::parse(&url).is_ok());

        // With port
        let client = Client {
            port: Some("80".to_string()),
            ..Default::default()
        };
        let url = client.url("/other_endpoint");
        assert!(url.contains(&client.hostname));
        assert!(url.contains(":80"));
        assert!(url.ends_with("/other_endpoint"));
        assert!(Url::parse(&url).is_ok());
    }

    #[tokio::test]
    async fn test_check() {
        let client = get_testing_client();
        let req = check::Request::new();
        let req = req.with_text("There are no spelling mistakes here.");

        let check_res = client.check(&req).await.unwrap();
        assert!(check_res.matches.is_empty());
        assert_eq!(&check_res.language.code, "en-US");
        assert_eq!(&check_res.software.name, "LanguageTool");
    }

    #[tokio::test]
    async fn test_languages() {
        let client = get_testing_client();

        let languages_res = client.languages().await;
        assert!(languages_res.is_ok_and(|r| !r.is_empty()));
    }
}
