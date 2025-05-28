use clap::Parser;
use std::io::Write;
use termcolor::StandardStream;

use crate::{api::server::ServerClient, error::Result};

use super::ExecuteSubcommand;

#[derive(Debug, Parser)]
pub struct Command {}

impl ExecuteSubcommand for Command {
    /// Executes the `languages` subcommand.
    async fn execute(self, mut stdout: StandardStream, server_client: ServerClient) -> Result<()> {
        let languages_response = server_client.languages().await?;
        let languages = serde_json::to_string_pretty(&languages_response)?;

        writeln!(&mut stdout, "{languages}")?;
        Ok(())
    }
}
