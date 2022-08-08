use crate::error::{Error, Result};
use clap::Parser;
use std::process::{Command, Output, Stdio};

trait CommandOk {
    fn command_ok(&mut self) -> Result<Output>;
}

impl CommandOk for Command {
    #[inline]
    fn command_ok(&mut self) -> Result<Output> {
        let output = self.output()?;
        let status = output.status;

        if status.success() {
            Ok(output)
        } else {
            match status.code() {
                Some(code) => Err(Error::CommandFailed {
                    body: format!("Process terminated with exit code: {}", code),
                }),
                None => Err(Error::CommandFailed {
                    body: "Process terminated by signal".to_string(),
                }),
            }
        }
    }
}

#[cfg_attr(feature = "cli", derive(Parser))]
/// Commands to pull, start and stop a LanguageTool using Docker.
pub struct Docker {
    #[cfg_attr(feature = "cli", clap(default_value = "erikvl87/languagetool"))]
    /// Image or repository from a registry.
    name: String,
    #[cfg_attr(feature = "cli", clap(short = 'b', long, default_value = "docker"))]
    /// Path to Docker's binaries.
    bin: String,
    #[cfg_attr(feature = "cli", clap(long, default_value = "languagetool"))]
    /// Name assigned to the container.
    container_name: String,
    #[cfg_attr(feature = "cli", clap(short = 'p', long, default_value = "8010:8010"))]
    /// Publish a container's port(s) to the host.
    port: String,
    #[clap(subcommand)]
    /// Docker action.
    action: Action,
}

#[derive(clap::Subcommand)]
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
    pub fn pull(&self) -> Result<Output> {
        Command::new(&self.bin)
            .args(["pull", &self.name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .command_ok()
    }

    pub fn start(&self) -> Result<Output> {
        Command::new(&self.bin)
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
            .command_ok()
    }

    pub fn stop(&self) -> Result<Output> {
        let output = Command::new(&self.bin)
            .args(["ps", "-l", "-f", &format!("name={}", self.container_name), "-q"])
            .stderr(Stdio::inherit())
            .command_ok()?;

        let docker_id: String = String::from_utf8_lossy(&output.stdout)
            .chars()
            .filter(|c| c.is_alphanumeric()) // This avoids newlines
            .collect();

        Command::new(&self.bin)
            .args(["kill", &docker_id])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .command_ok()
    }

    pub fn run_action(&self) -> Result<Output> {
        match self.action {
            Action::Pull => self.pull(),
            Action::Start => self.start(),
            Action::Stop => self.stop(),
        }
    }
}
