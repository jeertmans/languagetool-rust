/// Raw bindings to the LanguageTool API v1.1.2.
///
/// The current bindings were generated using the
/// [HTTP API documentation](https://languagetool.org/http-api/).
///
/// Unfortunately, the LanguageTool API is not as documented as we could
/// hope, and resquests might return undocumented fields. Those are deserialized
/// to the `undocumented` field.
pub mod check;
pub mod languages;
pub mod server;
pub mod words;

/// A HTTP client for making requests to some LanguageTool server.
pub struct Client {
    /// Server's hostname.
    hostname: String,
    /// Server's port.
    port: Option<String>,
    /// Inner client to perform HTTP requets.
    client: reqwest::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            hostname: "https://api.languagetoolplus.com".to_string(),
            ..Default::default()
        }
    }
}

impl Client {
    /// Construct a HTTP url base on the current hostname, optional port,
    /// and provided endpoint.
    #[inline]
    pub fn url(&self, endpoint: &str) -> String {
        let hostname = self.hostname;
        match self.port {
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
            .await?
            .json::<check::Response>()
    }

    /// Send a words request to the server and await for the response.
    pub async fn languages(&self, request: &languages::Request) -> Result<languages::Response> {
        self.client
            .get(self.url("/languages"))
            .query(request)
            .send()
            .await?
            .json::<check::Response>()
    }
}
