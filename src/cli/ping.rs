use clap::Parser;
use std::io::Write;
use termcolor::StandardStream;

use crate::{api::server::ServerClient, error::Result};

use super::ExecuteSubcommand;

#[derive(Debug, Parser)]
pub struct Command {}

impl ExecuteSubcommand for Command {
    /// Execute the `languages` subcommand.
    async fn execute(self, mut stdout: StandardStream, server_client: ServerClient) -> Result<()> {
        let ping = server_client.ping().await?;

        writeln!(&mut stdout, "PONG! Delay: {ping} ms")?;
        Ok(())
    }
}
