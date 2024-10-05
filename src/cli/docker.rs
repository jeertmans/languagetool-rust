//! Structures and methods to easily manipulate Docker images, especially for
//! LanguageTool applications.

use std::process::{Command as ProcessCommand, Output, Stdio};

use clap::{Args, Parser};
use termcolor::StandardStream;

use crate::{
    api::server::ServerClient,
    error::{Error, Result, exit_status_error},
};

use super::ExecuteSubcommand;

/// Commands to pull, start and stop a `LanguageTool` container using Docker.
#[derive(Debug, Clone, Args)]
pub struct Docker {
    /// Image or repository from a registry.
    #[clap(
        default_value = "erikvl87/languagetool",
        env = "LANGUAGETOOL_DOCKER_IMAGE"
    )]
    name: String,
    /// Path to Docker's binaries.
    #[clap(
        short = 'b',
        long,
        default_value = "docker",
        env = "LANGUAGETOOL_DOCKER_BIN"
    )]
    bin: String,
    /// Name assigned to the container.
    #[clap(long, default_value = "languagetool", env = "LANGUAGETOOL_DOCKER_NAME")]
    container_name: String,
    /// Publish a container's port(s) to the host.
    #[clap(
        short = 'p',
        long,
        default_value = "8010:8010",
        env = "LANGUAGETOOL_DOCKER_PORT"
    )]
    port: String,
    /// Docker action.
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand, Clone, Debug)]
/// Enumerate supported Docker actions.
enum Action {
    /// Pull a docker docker image.
    ///
    /// Alias to `{docker.bin} pull {docker.name}`.
    Pull,
    /// Start a (detached) docker container.
    ///
    /// Alias to `{docker.bin} run --rm -d -p {docker.port} {docker.name}`
    Start,
    /// Stop a docker container.
    ///
    /// Alias to `{docker.bin} kill $({docker.bin} ps -l -f
    /// "name={docker.container_name}")`.
    Stop,
}

impl Docker {
    /// Pull a Docker image from the given repository/file/...
    pub fn pull(&self) -> Result<Output> {
        let output = ProcessCommand::new(&self.bin)
            .args(["pull", &self.name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|_| Error::CommandNotFound(self.bin.to_string()))?;

        exit_status_error(&output.status)?;

        Ok(output)
    }

    /// Start a Docker container with given specifications.
    pub fn start(&self) -> Result<Output> {
        let output = ProcessCommand::new(&self.bin)
            .args([
                "run",
                "--rm",
                "--name",
                &self.container_name,
                "-d",
                "-p",
                "8010:8010",
                &self.name,
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|_| Error::CommandNotFound(self.bin.to_string()))?;

        exit_status_error(&output.status)?;

        Ok(output)
    }

    /// Stop the latest Docker container with the given name.
    pub fn stop(&self) -> Result<Output> {
        let output = ProcessCommand::new(&self.bin)
            .args([
                "ps",
                "-l",
                "-f",
                &format!("name={}", self.container_name),
                "-q",
            ])
            .stderr(Stdio::inherit())
            .output()
            .map_err(|_| Error::CommandNotFound(self.bin.to_string()))?;

        exit_status_error(&output.status)?;

        let docker_id: String = String::from_utf8_lossy(&output.stdout)
            .chars()
            .filter(|c| c.is_alphanumeric()) // This avoids newlines
            .collect();

        let output = ProcessCommand::new(&self.bin)
            .args(["kill", &docker_id])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;

        exit_status_error(&output.status)?;

        Ok(output)
    }

    /// Run a Docker command according to `self.action`.
    pub fn run_action(&self) -> Result<Output> {
        match self.action {
            Action::Pull => self.pull(),
            Action::Start => self.start(),
            Action::Stop => self.stop(),
        }
    }
}

/// Commands to easily run a LanguageTool server with Docker.
#[derive(Debug, Parser)]
pub struct Command {
    /// Actual command arguments.
    #[command(flatten)]
    pub docker: Docker,
}

impl ExecuteSubcommand for Command {
    /// Execute the `docker` subcommand.
    async fn execute(self, _stdout: StandardStream, _: ServerClient) -> Result<()> {
        self.docker.run_action()?;
        Ok(())
    }
}
