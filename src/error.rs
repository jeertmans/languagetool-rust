//! Error and Result structure used all across this crate.

use std::process::ExitStatus;

/// Enumeration of all possible error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from the command line parsing (see [`clap::Error`]).
    #[cfg(feature = "cli")]
    #[error(transparent)]
    Cli(#[from] clap::Error),

    /// Error when a process command was not found.
    #[error("command not found: {0}")]
    CommandNotFound(String),

    /// Error from a command line process (see [`std::process::Command`]).
    #[error("command failed: {0:?}")]
    ExitStatus(String),

    /// Error specifying an invalid
    /// [`DataAnnotation`](`crate::api::check::DataAnnotation`).
    #[error("invalid request: {0}")]
    InvalidDataAnnotation(String),

    /// Error from checking if `filename` exists and is a actually a file.
    #[error("invalid filename (got '{0}', does not exist or is not a file)")]
    InvalidFilename(String),

    /// Error specifying an invalid request.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Error specifying an invalid value.
    #[error("invalid value: {0:?}")]
    InvalidValue(String),

    /// Error from reading and writing to IO (see [`std::io::Error`]).
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Error when joining multiple futures.
    #[cfg(feature = "multithreaded")]
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    /// Error from parsing JSON (see [`serde_json::Error`]).
    #[error(transparent)]
    JSON(#[from] serde_json::Error),

    /// Error while parsing Action.
    #[error("could not parse {0:?} in a Docker action")]
    ParseAction(String),

    /// Any other error from requests (see [`reqwest::Error`]).
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Error from reading environ variable (see [`std::env::VarError`]).
    #[error(transparent)]
    VarError(#[from] std::env::VarError),
}

/// Result type alias with error type defined above (see [`Error`]]).
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
    use assert_matches::assert_matches;

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

        assert_matches!(error, Error::Cli(_));
    }

    #[test]
    fn test_error_json() {
        let result = serde_json::from_str::<serde_json::Value>("invalid JSON");
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert_matches!(error, Error::JSON(_));
    }

    #[test]
    fn test_error_io() {
        let result = std::fs::read_to_string("");
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert_matches!(error, Error::IO(_));
    }

    #[test]
    fn test_error_invalid_request() {
        let result = crate::api::check::Request::new().try_get_text();
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert_matches!(error, Error::InvalidRequest(_));
    }

    #[test]
    fn test_error_invalid_value() {
        let result = crate::api::server::parse_port("test");
        assert!(result.is_err());

        let error: Error = result.unwrap_err().into();

        assert_matches!(error, Error::InvalidValue(_));
    }

    #[tokio::test]
    async fn test_error_reqwest() {
        let result = reqwest::get("").await;
        let error: Error = result.unwrap_err().into();

        assert_matches!(error, Error::Reqwest(_));
    }
}
