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
