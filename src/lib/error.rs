//! Error and Result structure used all across this crate.

/// Enumeration of all possible error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "cli")]
    #[error(transparent)]
    /// Error from the command line parsing (see [clap::Error])
    Cli(#[from] clap::Error),
    #[error("command failed: {body:?}")]
    CommandFailed {
        /// Error body
        body: String,
    },
    #[error(transparent)]
    /// Error from parsing JSON (see [serde_json::Error])
    JSON(#[from] serde_json::Error),
    #[error(transparent)]
    /// Error from reading and writing to IO (see [std::io::Error])
    IO(#[from] std::io::Error),
    #[error("invalid request: {body:?}")]
    /// Error specifying an invalid request
    InvalidRequest {
        /// Error body
        body: String,
    },
    #[error("invalid value: {body:?}")]
    /// Error specifying an invalid value
    InvalidValue {
        /// Error body
        body: String,
    },
    #[error("request could not be properly encoded: {source}")]
    /// Error from request encoding
    RequestEncode {
        /// Source error
        source: reqwest::Error,
    },
    #[error("response could not be properly decoded: {source}")]
    /// Error from request decoding
    ResponseDecode {
        /// Source error
        source: reqwest::Error,
    },
    #[error(transparent)]
    /// Any other error from requests (see [reqwest::Error])
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    /// Error from reading environ variable (see [std::env::VarError])
    VarError(#[from] std::env::VarError),
}

/// Result type alias with error type defined above (see [Error]).
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {

    use crate::error::Error;
    #[cfg(feature = "cli")]
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
    }
}
