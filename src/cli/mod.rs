//! Command line tools.
//!
//! This module is specifically designed to be used by LTRS's binary target.
//! It contains all the content needed to create LTRS's command line interface.

mod check;
#[cfg(feature = "cli-complete")]
mod completions;
#[cfg(feature = "docker")]
mod docker;
mod languages;
mod ping;
mod words;

use std::{io, ops::Deref};

use clap::{CommandFactory, Parser, Subcommand};
use is_terminal::IsTerminal;
#[cfg(feature = "annotate")]
use termcolor::{ColorChoice, StandardStream};

#[cfg(feature = "docker")]
pub use docker::Docker;

use crate::{
    api::server::{ServerCli, ServerClient},
    error::Result,
};

/// Read lines from standard input and write to buffer string.
///
/// Standard output is used when waiting for user to input text.
fn read_from_stdin<W>(stdout: &mut W, buffer: &mut String) -> Result<()>
where
    W: io::Write,
{
    if io::stdin().is_terminal() {
        #[cfg(windows)]
        writeln!(
            stdout,
            "Reading from STDIN, press [CTRL+Z] when you're done."
        )?;

        #[cfg(unix)]
        writeln!(
            stdout,
            "Reading from STDIN, press [CTRL+D] when you're done."
        )?;
    }
    let stdin = std::io::stdin();

    while stdin.read_line(buffer)? > 0 {}
    Ok(())
}

/// Main command line structure. Contains every subcommand.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "LanguageTool API bindings in Rust.",
    propagate_version(true),
    subcommand_required(true),
    verbatim_doc_comment
)]
pub struct Cli {
    /// Specify WHEN to colorize output.
    #[arg(short, long, value_name = "WHEN", default_value = "auto", default_missing_value = "always", num_args(0..=1), require_equals(true))]
    pub color: clap::ColorChoice,
    /// [`ServerCli`] arguments.
    #[command(flatten, next_help_heading = "Server options")]
    pub server_cli: ServerCli,
    /// Subcommand.
    #[command(subcommand)]
    #[allow(missing_docs)]
    pub command: Command,
}

/// All possible subcommands.
#[derive(Subcommand, Debug)]
#[allow(missing_docs)]
pub enum Command {
    /// Check text using LanguageTool server.
    Check(check::Command),
    /// Commands to easily run a LanguageTool server with Docker.
    #[cfg(feature = "docker")]
    Docker(docker::Command),
    /// Return list of supported languages.
    #[clap(visible_alias = "lang")]
    Languages(languages::Command),
    /// Ping the LanguageTool server and return time elapsed in ms if success.
    Ping(ping::Command),
    /// Retrieve some user's words list, or add / delete word from it.
    Words(words::Command),
    /// Generate tab-completion scripts for supported shells
    #[cfg(feature = "cli-complete")]
    Completions(completions::Command),
}

/// Provides a common interface for executing the subcommands.
trait ExecuteSubcommand {
    /// Executes the subcommand.
    async fn execute(self, stdout: StandardStream, server_client: ServerClient) -> Result<()>;
}

impl Cli {
    /// Return a standard output stream that optionally supports color.
    #[must_use]
    fn stdout(&self) -> StandardStream {
        let mut choice: ColorChoice = match self.color {
            clap::ColorChoice::Auto => ColorChoice::Auto,
            clap::ColorChoice::Always => ColorChoice::Always,
            clap::ColorChoice::Never => ColorChoice::Never,
        };

        if choice == ColorChoice::Auto && !io::stdout().is_terminal() {
            choice = ColorChoice::Never;
        }

        StandardStream::stdout(choice)
    }

    /// Execute command, possibly returning an error.
    pub async fn execute(self) -> Result<()> {
        let stdout = self.stdout();
        let server_client: ServerClient = self.server_cli.into();

        match self.command {
            Command::Check(cmd) => cmd.execute(stdout, server_client).await,
            Command::Languages(cmd) => cmd.execute(stdout, server_client).await,
            Command::Ping(cmd) => cmd.execute(stdout, server_client).await,
            Command::Words(cmd) => cmd.execute(stdout, server_client).await,
            #[cfg(feature = "docker")]
            Command::Docker(cmd) => cmd.execute(stdout, server_client).await,
            #[cfg(feature = "cli-complete")]
            Command::Completions(cmd) => cmd.execute(stdout, server_client).await,
        }
    }
}

/// Build a command from the top-level command line structure.
#[must_use]
pub fn build_cli() -> clap::Command {
    Cli::command()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cli() {
        Cli::command().debug_assert();
    }
}
