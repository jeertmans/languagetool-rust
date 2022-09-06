use clap::{CommandFactory, FromArgMatches};
#[cfg(feature = "cli-complete")]
use clap_complete::{generate, shells};
use languagetool_rust::error::Result;
use languagetool_rust::*;
use std::io::{BufRead, Write};

#[cfg(feature = "cli-complete")]
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

fn build_cli() -> clap::Command<'static> {
    let command = ServerClient::command()
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .name(clap::crate_name!())
        .version(clap::crate_version!())
        .global_setting(clap::AppSettings::DeriveDisplayOrder)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .propagate_version(true)
        .subcommand(
            CheckRequest::command()
                .name("check")
                .author(clap::crate_authors!()),
        )
        .subcommand(
            clap::Command::new("languages")
                .author(clap::crate_authors!())
                .about("LanguageTool GET languages request"),
        )
        .subcommand(
            WordsRequest::command()
                .name("words")
                .author(clap::crate_authors!())
                .subcommand_negates_reqs(true)
                .subcommand(WordsAddRequest::command().name("add"))
                .subcommand(WordsDeleteRequest::command().name("delete")),
        )
        .subcommand(
            clap::Command::new("ping")
                .author(clap::crate_authors!())
                .about("Ping the LanguageTool server and return time elapsed in ms if success"),
        );

    #[cfg(feature = "docker")]
    let command = command.subcommand(
        Docker::command()
            .name("docker")
            .author(clap::crate_authors!()),
    );

    #[cfg(feature = "cli-complete")]
    let command = command.subcommand(
        clap::Command::new("completions")
            .author(clap::crate_authors!())
            .about("Generate tab-completion scripts for supported shells")
            .after_long_help(COMPLETIONS_HELP)
            .after_help("Use --help for installation help.")
            .arg_required_else_help(true)
            .arg(
                clap::Arg::new("shell")
                    .takes_value(true)
                    .required(true)
                    .ignore_case(true)
                    .value_parser([
                        clap::PossibleValue::new("bash"),
                        clap::PossibleValue::new("elvish"),
                        clap::PossibleValue::new("fish"),
                        clap::PossibleValue::new("powershell"),
                        clap::PossibleValue::new("zsh"),
                    ]),
            ),
    );

    command
}

#[tokio::main]
async fn main() {
    if let Err(e) = try_main().await {
        eprintln!("{}", e);
        std::process::exit(2);
    }
}

async fn try_main() -> Result<()> {
    let matches = build_cli().get_matches();

    // TODO: prompt max_suggestion
    let client = ServerClient::from_arg_matches(&matches)?.with_max_suggestions(5);
    let mut stdout = std::io::stdout();

    #[allow(clippy::significant_drop_in_scrutinee)]
    match matches.subcommand() {
        Some(("check", sub_matches)) => {
            let mut req = CheckRequest::from_arg_matches(sub_matches)?;

            if req.text.is_none() && req.data.is_none() {
                let mut text = String::new();

                for line in std::io::stdin().lock().lines() {
                    text.push_str(&line?);
                    text.push('\n');
                }

                req.text = Some(text);
            }

            #[cfg(feature = "annotate")]
            if !req.raw {
                writeln!(&stdout, "{}", &client.annotate_check(&req).await?)?;
                return Ok(());
            }

            let mut resp = client.check(&req).await?;

            #[cfg(feature = "cli")]
            if req.more_context {
                use crate::check::CheckResponseWithContext;
                let text = req.get_text();
                resp = CheckResponseWithContext::new(text, resp).into();
            }

            writeln!(&stdout, "{}", serde_json::to_string_pretty(&resp)?)?;
        }
        Some(("languages", _sub_matches)) => {
            writeln!(
                &stdout,
                "{}",
                serde_json::to_string_pretty(&client.languages().await?)?
            )?;
        } // TODO: `words` requests are not tested yet
        Some(("words", sub_matches)) => match sub_matches.subcommand() {
            Some(("add", sub_matches)) => {
                writeln!(
                    &stdout,
                    "{}",
                    serde_json::to_string_pretty(
                        &client
                            .words_add(&WordsAddRequest::from_arg_matches(sub_matches)?)
                            .await?
                    )?
                )?;
            }
            Some(("delete", sub_matches)) => {
                writeln!(
                    &stdout,
                    "{}",
                    serde_json::to_string_pretty(
                        &client
                            .words_delete(&WordsDeleteRequest::from_arg_matches(sub_matches)?)
                            .await?
                    )?
                )?;
            }
            _ => {
                writeln!(
                    &stdout,
                    "{}",
                    serde_json::to_string_pretty(
                        &client
                            .words(&WordsRequest::from_arg_matches(sub_matches)?)
                            .await?
                    )?
                )?;
            }
        },
        Some(("ping", _sub_matches)) => {
            writeln!(&stdout, "PONG! Delay: {} ms", client.ping().await?)?
        }
        #[cfg(feature = "docker")]
        Some(("docker", sub_matches)) => Docker::from_arg_matches(sub_matches)?
            .run_action()
            .map(|_| ())?,
        #[cfg(feature = "cli-complete")]
        Some(("completions", sub_matches)) => match sub_matches.value_of("shell").unwrap() {
            "bash" => generate(
                shells::Bash,
                &mut build_cli(),
                env!("CARGO_BIN_NAME"),
                &mut stdout,
            ),
            "elvish" => generate(
                shells::Elvish,
                &mut build_cli(),
                env!("CARGO_BIN_NAME"),
                &mut stdout,
            ),
            "fish" => generate(
                shells::Fish,
                &mut build_cli(),
                env!("CARGO_BIN_NAME"),
                &mut stdout,
            ),
            "powershell" => generate(
                shells::PowerShell,
                &mut build_cli(),
                env!("CARGO_BIN_NAME"),
                &mut stdout,
            ),
            "zsh" => generate(
                shells::Zsh,
                &mut build_cli(),
                env!("CARGO_BIN_NAME"),
                &mut stdout,
            ),

            _ => (),
        },
        _ => unreachable!(), // Can't be None since subcommand is required
    }

    Ok(())
}
