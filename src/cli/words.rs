use clap::{Parser, Subcommand};
use std::io::Write;
use termcolor::StandardStream;

use crate::{
    api::{self, server::ServerClient, words::RequestArgs},
    error::Result,
};

use super::ExecuteSubcommand;

/// Retrieve some user's words list.
#[derive(Debug, Parser)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(subcommand_negates_reqs = true)]
pub struct Command {
    /// Actual GET request.
    #[command(flatten)]
    pub request: RequestArgs,
    /// Optional subcommand.
    #[command(subcommand)]
    pub subcommand: Option<WordsSubcommand>,
}

/// Words' optional subcommand.
#[derive(Clone, Debug, Subcommand)]
pub enum WordsSubcommand {
    /// Add a word to some user's list.
    Add(api::words::add::Request),
    /// Remove a word from some user's list.
    Delete(api::words::delete::Request),
}

impl ExecuteSubcommand for Command {
    /// Executes the `words` subcommand.
    async fn execute(self, mut stdout: StandardStream, server_client: ServerClient) -> Result<()> {
        let words = match self.subcommand {
            Some(WordsSubcommand::Add(request)) => {
                let words_response = server_client.words_add(&request).await?;
                serde_json::to_string_pretty(&words_response)?
            },
            Some(WordsSubcommand::Delete(request)) => {
                let words_response = server_client.words_delete(&request).await?;
                serde_json::to_string_pretty(&words_response)?
            },
            None => {
                let words_response = server_client.words(&self.request.into()).await?;
                serde_json::to_string_pretty(&words_response)?
            },
        };

        writeln!(&mut stdout, "{words}")?;
        Ok(())
    }
}
