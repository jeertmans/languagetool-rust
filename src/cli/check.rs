//! Check text using LanguageTool server.
//!
//! The input can be one of the following:
//!
//! - raw text, if `--text TEXT` is provided;
//! - annotated data, if `--data TEXT` is provided;
//! - text from file(s), if `[FILE(S)]...` are provided.
//! - raw text through `stdin`, if nothing else is provided.
use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use termcolor::{StandardStream, WriteColor};

use crate::{
    api::{self, check::Request, server::ServerClient},
    error::{Error, Result},
};

use super::ExecuteSubcommand;

/// Parse a string slice into a [`PathBuf`], and error if the file does not
/// exist.
fn parse_filename(s: &str) -> Result<PathBuf> {
    let path_buf = PathBuf::from(s);

    if path_buf.is_file() {
        Ok(path_buf)
    } else {
        Err(Error::InvalidFilename(s.to_string()))
    }
}

#[derive(Debug, Parser)]
pub struct Command {
    /// If present, raw JSON output will be printed instead of annotated text.
    /// This has no effect if `--data` is used, because it is never
    /// annotated.
    #[clap(short = 'r', long)]
    pub raw: bool,
    /// Sets the maximum number of characters before splitting.
    #[clap(long, default_value_t = 1500)]
    pub max_length: usize,
    /// If text is too long, will split on this pattern.
    #[clap(long, default_value = "\n\n")]
    pub split_pattern: String,
    /// Max. number of suggestions kept. If negative, all suggestions are kept.
    #[clap(long, default_value_t = 5, allow_negative_numbers = true)]
    pub max_suggestions: isize,
    /// Specify the files type to use the correct parser.
    ///
    /// If set to auto, the type is guessed from the filename extension.
    #[clap(long, value_enum, default_value_t = FileType::default(), ignore_case = true)]
    pub r#type: FileType,
    /// Optional filenames from which input is read.
    #[arg(conflicts_with_all(["text", "data"]), value_parser = parse_filename)]
    pub filenames: Vec<PathBuf>,
    /// Inner [`Request`].
    #[command(flatten, next_help_heading = "Request options")]
    pub request: Request,
}

/// Support file types.
#[derive(Clone, Debug, Default, ValueEnum)]
#[non_exhaustive]
pub enum FileType {
    /// Auto.
    #[default]
    Auto,
    /// Markdown.
    Markdown,
    /// Typst.
    Typst,
}

impl ExecuteSubcommand for Command {
    /// Executes the `check` subcommand.
    async fn execute(self, mut stdout: StandardStream, server_client: ServerClient) -> Result<()> {
        let mut request = self.request;
        #[cfg(feature = "annotate")]
        let color = stdout.supports_color();

        let server_client = server_client.with_max_suggestions(self.max_suggestions);

        // ANNOTATED DATA, RAW TEXT, STDIN
        if self.filenames.is_empty() {
            // Fallback to `stdin` if nothing else is provided
            if request.text.is_none() && request.data.is_none() {
                let mut text = String::new();
                super::read_from_stdin(&mut stdout, &mut text)?;
                request = request.with_text(text);
            }

            let Some(text) = &request.text else {
                // Handle annotated data
                let response = server_client.check(&request).await?;
                writeln!(&mut stdout, "{}", serde_json::to_string_pretty(&response)?)?;
                return Ok(());
            };

            let requests = request.split(self.max_length, self.split_pattern.as_str());
            let mut response = server_client.check_multiple_and_join(requests).await?;
            response = api::check::ResponseWithContext::new(text.clone(), response).into();

            writeln!(
                &mut stdout,
                "{}",
                &response.annotate(text.as_str(), None, color)
            )?;

            return Ok(());
        }

        // FILES
        for filename in self.filenames.iter() {
            let text = std::fs::read_to_string(filename)?;
            let requests = request
                .clone()
                .with_text(text.clone())
                .split(self.max_length, self.split_pattern.as_str());
            let response = server_client.check_multiple_and_join(requests).await?;

            if !self.raw {
                writeln!(
                    &mut stdout,
                    "{}",
                    &response.annotate(text.as_str(), filename.to_str(), color)
                )?;
            } else {
                writeln!(&mut stdout, "{}", serde_json::to_string_pretty(&response)?)?;
            }
        }

        Ok(())
    }
}
