#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "cli")]
    #[error(transparent)]
    Cli(#[from] clap::Error),
    #[error(transparent)]
    JSON(#[from] serde_json::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("invalid request: {body:?}")]
    InvalidRequest { body: String },
    #[error("invalid value: {body:?}")]
    InvalidValue { body: String },
    #[error("request could not be properly encoded: {source}")]
    RequestEncode { source: reqwest::Error },
    #[error("response could not be properly decoded: {source}")]
    ResponseDecode { source: reqwest::Error },
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {

    use crate::error::Error;
    use crate::server::ServerClient;
    use clap::Command;

    #[cfg(feature = "cli")]
    #[test]
    fn test_error_cli() {
        let result =
            Command::new("").try_get_matches_from(vec!["some", "args", "that", "should", "fail"]);
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::Cli(_)));
    }

    #[test]
    fn test_error_json() {
        let result = serde_json::from_str::<serde_json::Value>("invalid JSON");
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::JSON(_)));
    }

    #[test]
    fn test_error_io() {
        let result = std::fs::read_to_string("");
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::IO(_)));
    }

    #[ignore]
    #[test]
    fn test_error_invalid_request() {
        let result = std::fs::read_to_string(""); // TODO
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::InvalidRequest { body: _ }));
    }

    #[ignore]
    #[test]
    fn test_error_invalid_value() {
        let result = std::fs::read_to_string(""); // TODO
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::InvalidValue { body: _ }));
    }

    #[ignore]
    #[test]
    fn test_error_request_encode() {
        let result = std::fs::read_to_string(""); // TODO
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::RequestEncode { source: _ }));
    }

    #[ignore]
    #[test]
    fn test_error_response_decode() {
        let result = std::fs::read_to_string(""); // TODO
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::ResponseDecode { source: _ }));
    }

    #[ignore]
    #[test]
    fn test_error_reqwest() {
        let result = std::fs::read_to_string(""); // TODO
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::Reqwest(_)));
    }}
