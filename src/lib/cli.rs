//! Command line tools.
//!
//! This module is specifically designed to be used by LTRS's binary target.
//! It contains all the content needed to create LTRS's command line interface.

use crate::{
    error::Result,
    server::{ServerCli, ServerClient},
    words::WordsSubcommand,
};
use clap::{CommandFactory, Parser, Subcommand};
use is_terminal::IsTerminal;
use std::io::{self, Write};
#[cfg(feature = "annotate")]
use termcolor::WriteColor;
use termcolor::{ColorChoice, StandardStream};

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
    #[command(flatten)]
    pub server_cli: ServerCli,
    /// Subcommand.
    #[command(subcommand)]
    #[allow(missing_docs)]
    pub command: Command,
}

/// Enumerate all possible commands.
#[derive(Subcommand, Debug)]
#[allow(missing_docs)]
pub enum Command {
    /// Check command.
    Check(crate::check::CheckCommand),
    /// Docker commands.
    #[cfg(feature = "docker")]
    Docker(crate::docker::DockerCommand),
    /// LanguageTool GET languages request.
    #[clap(visible_alias = "lang")]
    Languages,
    /// Ping the LanguageTool server and return time elapsed in ms if success.
    Ping,
    /// Words commands.
    Words(crate::words::WordsCommand),
    /// Completions command.
    #[cfg(feature = "cli-complete")]
    Completions(complete::CompleteCommand),
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

    /// Execute command, possibily returning an error.
    pub async fn execute(self) -> Result<()> {
        let mut stdout = self.stdout();

        let server_client: ServerClient = self.server_cli.into();

        match self.command {
            Command::Check(cmd) => {
                let mut request = cmd.request;
                #[cfg(feature = "annotate")]
                let color = stdout.supports_color();

                type Item<'a> = Result<(Option<String>, Option<&'a str>)>;

                let sources_iter: Box<dyn Iterator<Item = Item>> =
                    if cmd.filenames.is_empty() {
                        if request.text.is_none() && request.data.is_none() {
                            let mut text = String::new();
                            match read_from_stdin(&mut stdout, &mut text) {
                                Ok(_) => Box::new(vec![Ok((Some(text), None))].into_iter()),
                                Err(e) => Box::new(vec![Err(e)].into_iter()),
                            }
                        } else {
                            Box::new(vec![Ok((None, None))].into_iter())
                        }
                    } else {
                        Box::new(cmd.filenames.iter().map(|filename| {
                            let text = std::fs::read_to_string(filename)?;
                            Ok((Some(text), filename.to_str()))
                        }))
                    };

                for source in sources_iter {
                    let (text, _filename) = source?;
                    if let Some(text) = text {
                        request = request.with_text(text);
                    }

                    #[cfg(feature = "annotate")]
                    if !cmd.raw {
                        writeln!(
                            &mut stdout,
                            "{}",
                            &server_client
                                .annotate_check(&request, _filename, color)
                                .await?
                        )?;
                    } else {
                        let mut resp = server_client.check(&request).await?;

                        if cmd.more_context {
                            use crate::check::CheckResponseWithContext;
                            let text = request.get_text();
                            resp = CheckResponseWithContext::new(text, resp).into();
                        }

                        writeln!(&mut stdout, "{}", serde_json::to_string_pretty(&resp)?)?;
                    }
                }
            },
            #[cfg(feature = "docker")]
            Command::Docker(cmd) => {
                cmd.execute(&mut stdout)?;
            },
            Command::Languages => {
                let languages_response = server_client.languages().await?;
                let languages = serde_json::to_string_pretty(&languages_response)?;

                writeln!(&mut stdout, "{languages}")?;
            },
            Command::Ping => {
                let ping = server_client.ping().await?;
                writeln!(&mut stdout, "PONG! Delay: {ping} ms")?;
            },
            Command::Words(cmd) => {
                let words = match &cmd.subcommand {
                    Some(WordsSubcommand::Add(request)) => {
                        let words_response = server_client.words_add(request).await?;
                        serde_json::to_string_pretty(&words_response)?
                    },
                    Some(WordsSubcommand::Delete(request)) => {
                        let words_response = server_client.words_delete(request).await?;
                        serde_json::to_string_pretty(&words_response)?
                    },
                    None => {
                        let words_response = server_client.words(&cmd.request).await?;
                        serde_json::to_string_pretty(&words_response)?
                    },
                };

                writeln!(&mut stdout, "{words}")?;
            },
            #[cfg(feature = "cli-complete")]
            Command::Completions(cmd) => {
                cmd.execute(&mut stdout)?;
            },
        }
        Ok(())
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

#[cfg(feature = "cli-complete")]
pub(crate) mod complete {
    //! Completion scripts generation with [`clap_complete`].

    use crate::error::Result;
    use clap::{Command, Parser};
    use clap_complete::{generate, shells::Shell};
    use std::io::Write;

    /// Command structure to generate complete scripts.
    #[derive(Debug, Parser)]
    #[command(
    about = "Generate tab-completion scripts for supported shells",
    after_help = "Use --help for installation help.",
    after_long_help = COMPLETIONS_HELP
)]
    pub struct CompleteCommand {
        /// Shell for which to completion script is generated.
        #[arg(value_enum, ignore_case = true)]
        shell: Shell,
    }

    impl CompleteCommand {
        /// Generate completion file for current shell and write to buffer.
        pub fn generate_completion_file<F, W>(&self, build_cli: F, buffer: &mut W)
        where
            F: FnOnce() -> Command,
            W: Write,
        {
            generate(self.shell, &mut build_cli(), "ltrs", buffer);
        }

        /// Execute command by writing completion script to stdout.
        pub fn execute<W>(&self, stdout: &mut W) -> Result<()>
        where
            W: Write,
        {
            self.generate_completion_file(super::build_cli, stdout);
            Ok(())
        }
    }

    pub(crate) static COMPLETIONS_HELP: &str = r"DISCUSSION:
    Enable tab completion for Bash, Fish, Zsh, or PowerShell
    Elvish shell completion is currently supported, but not documented below.
    The script is output on `stdout`, allowing one to re-direct the
    output to the file of their choosing. Where you place the file
    will depend on which shell, and which operating system you are
    using. Your particular configuration may also determine where
    these scripts need to be placed.
    Here are some common set ups for the three supported shells under
    Unix and similar operating systems (such as GNU/Linux).
    BASH:
    Completion files are commonly stored in `/etc/bash_completion.d/` for
    system-wide commands, but can be stored in
    `~/.local/share/bash-completion/completions` for user-specific commands.
    Run the command:
        $ mkdir -p ~/.local/share/bash-completion/completions
        $ ltrs completions bash >> ~/.local/share/bash-completion/completions/ltrs
    This installs the completion script. You may have to log out and
    log back in to your shell session for the changes to take effect.
    BASH (macOS/Homebrew):
    Homebrew stores bash completion files within the Homebrew directory.
    With the `bash-completion` brew formula installed, run the command:
        $ mkdir -p $(brew --prefix)/etc/bash_completion.d
        $ ltrs completions bash > $(brew --prefix)/etc/bash_completion.d/ltrs.bash-completion
    FISH:
    Fish completion files are commonly stored in
    `$HOME/.config/fish/completions`. Run the command:
        $ mkdir -p ~/.config/fish/completions
        $ ltrs completions fish > ~/.config/fish/completions/ltrs.fish
    This installs the completion script. You may have to log out and
    log back in to your shell session for the changes to take effect.
    ZSH:
    ZSH completions are commonly stored in any directory listed in
    your `$fpath` variable. To use these completions, you must either
    add the generated script to one of those directories, or add your
    own to this list.
    Adding a custom directory is often the safest bet if you are
    unsure of which directory to use. First create the directory; for
    this example we'll create a hidden directory inside our `$HOME`
    directory:
        $ mkdir ~/.zfunc
    Then add the following lines to your `.zshrc` just before
    `compinit`:
        fpath+=~/.zfunc
    Now you can install the completions script using the following
    command:
        $ ltrs completions zsh > ~/.zfunc/_ltrs
    You must then either log out and log back in, or simply run
        $ exec zsh
    for the new completions to take effect.
    CUSTOM LOCATIONS:
    Alternatively, you could save these files to the place of your
    choosing, such as a custom directory inside your $HOME. Doing so
    will require you to add the proper directives, such as `source`ing
    inside your login script. Consult your shells documentation for
    how to add such directives.
    POWERSHELL:
    The powershell completion scripts require PowerShell v5.0+ (which
    comes with Windows 10, but can be downloaded separately for windows 7
    or 8.1).
    First, check if a profile has already been set
        PS C:\> Test-Path $profile
    If the above command returns `False` run the following
        PS C:\> New-Item -path $profile -type file -force
    Now open the file provided by `$profile` (if you used the
    `New-Item` command it will be
    `${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`
    Next, we either save the completions file into our profile, or
    into a separate file and source it inside our profile. To save the
    completions into our profile simply use
        PS C:\> ltrs completions powershell >> ${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
    SOURCE:
        This documentation is directly taken from: https://github.com/rust-lang/rustup/blob/8f6b53628ad996ad86f9c6225fa500cddf860905/src/cli/help.rs#L157";
}
