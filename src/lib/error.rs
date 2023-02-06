//! Error and Result structure used all across this crate.
use std::process::ExitStatus;

/// Enumeration of all possible error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "cli")]
    #[error(transparent)]
    /// Error from the command line parsing (see [clap::Error]).
    Cli(#[from] clap::Error),

    #[error("command failed: {0:?}")]
    /// Error from a command line process (see [std::process::Command]).
    ExitStatus(String),

    #[error(transparent)]
    /// Error from parsing JSON (see [serde_json::Error]).
    JSON(#[from] serde_json::Error),

    #[error(transparent)]
    /// Error from reading and writing to IO (see [std::io::Error]).
    IO(#[from] std::io::Error),

    #[error("invalid request: {0}")]
    /// Error specifying an invalid request.
    InvalidRequest(String),

    #[error("invalid value: {0:?}")]
    /// Error specifying an invalid value.
    InvalidValue(String),

    #[error("could not parse {0:?} in a Docker action")]
    /// Error while parsing Action.
    ParseAction(String),

    #[error("request could not be properly encoded: {0}")]
    /// Error from request encoding.
    RequestEncode(reqwest::Error),

    #[error("response could not be properly decoded: {0}")]
    /// Error from request decoding.
    ResponseDecode(reqwest::Error),

    #[error(transparent)]
    /// Any other error from requests (see [reqwest::Error]).
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    /// Error from reading environ variable (see [std::env::VarError]).
    VarError(#[from] std::env::VarError),

    #[error("command not found: {0}")]
    /// Error when a process command was not found.
    CommandNotFound(String),

    #[error("invalid filename (got '{0}', does not exist or is not a file)")]
    /// Error from checking if `filename` exists and is a actualla a file.
    InvalidFilename(String),
}

/// Result type alias with error type defined above (see [Error]).
pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub(crate) fn exit_status_error(exit_status: &ExitStatus) -> Result<()> {
    match exit_status.success() {
        true => Ok(()),
        false => {
            match exit_status.code() {
                Some(code) => {
                    Err(Error::ExitStatus(format!(
                        "Process terminated with exit code: {code}"
                    )))
                },
                None => {
                    Err(Error::ExitStatus(
                        "Process terminated by signal".to_string(),
                    ))
                },
            }
        },
    }
}

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

        assert!(matches!(error, Error::InvalidRequest(_)));
    }

    #[ignore]
    #[test]
    fn test_error_invalid_value() {
        let result = std::fs::read_to_string(""); // TODO
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::InvalidValue(_)));
    }

    #[ignore]
    #[test]
    fn test_error_request_encode() {
        let result = std::fs::read_to_string(""); // TODO
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::RequestEncode(_)));
    }

    #[ignore]
    #[test]
    fn test_error_response_decode() {
        let result = std::fs::read_to_string(""); // TODO
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert!(matches!(error, Error::ResponseDecode(_)));
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
