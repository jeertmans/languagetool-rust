//! Structures and methods to easily manipulate Docker images, especially for `LanguageTool`
//! applications.

use crate::error::{exit_status_error, Error, Result};
#[cfg(feature = "cli")]
use clap::Parser;
use std::process::{Command, Output, Stdio};

#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Debug, Clone)]
/// Commands to pull, start and stop a `LanguageTool` container using Docker.
pub struct Docker {
    #[cfg_attr(
        feature = "cli",
        clap(
            default_value = "erikvl87/languagetool",
            env = "LANGUAGETOOL_DOCKER_IMAGE"
        )
    )]
    /// Image or repository from a registry.
    name: String,
    #[cfg_attr(
        feature = "cli",
        clap(
            short = 'b',
            long,
            default_value = "docker",
            env = "LANGUAGETOOL_DOCKER_BIN"
        )
    )]
    /// Path to Docker's binaries.
    bin: String,
    #[cfg_attr(
        feature = "cli",
        clap(long, default_value = "languagetool", env = "LANGUAGETOOL_DOCKER_NAME")
    )]
    /// Name assigned to the container.
    container_name: String,
    #[cfg_attr(
        feature = "cli",
        clap(
            short = 'p',
            long,
            default_value = "8010:8010",
            env = "LANGUAGETOOL_DOCKER_PORT"
        )
    )]
    /// Publish a container's port(s) to the host.
    port: String,
    #[cfg_attr(feature = "cli", clap(subcommand))]
    /// Docker action.
    action: Action,
}

#[cfg_attr(feature = "cli", derive(clap::Subcommand))]
#[derive(Clone, Debug)]
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
    /// Alias to `{docker.bin} kill $({docker.bin} ps -l -f "name={docker.container_name}")`.
    Stop,
}

impl Docker {
    /// Pull a Docker image from the given repository/file/...
    pub fn pull(&self) -> Result<Output> {
        let output = Command::new(&self.bin)
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
        let output = Command::new(&self.bin)
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
        let output = Command::new(&self.bin)
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

        let output = Command::new(&self.bin)
            .args(["kill", &docker_id])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;

        exit_status_error(&output.status)?;

        Ok(output)
    }

    /// Run a Docker command according to self.action.
    pub fn run_action(&self) -> Result<Output> {
        match self.action {
            Action::Pull => self.pull(),
            Action::Start => self.start(),
            Action::Stop => self.stop(),
        }
    }
}
