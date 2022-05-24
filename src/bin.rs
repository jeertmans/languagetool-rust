use clap::{CommandFactory, FromArgMatches};
use languagetool_rust::error::Result;
use languagetool_rust::*;
use std::io::Write;

#[tokio::main]
async fn main() {
    if let Err(e) = try_main().await {
        eprintln!("{}", e);
        std::process::exit(2);
    }
}

async fn try_main() -> Result<()> {
    let matches = ServerClient::command()
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .name(clap::crate_name!())
        .version(clap::crate_version!())
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
        )
        .get_matches();

    let client = ServerClient::from_arg_matches(&matches)?;
    let stdout = std::io::stdout();

    match matches.subcommand() {
        Some(("check", sub_matches)) => {
            writeln!(
                &stdout,
                "{}",
                serde_json::to_string_pretty(
                    &client
                        .check(&CheckRequest::from_arg_matches(sub_matches)?)
                        .await?
                )?
            )?;
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
        _ => unreachable!(), // Can't be None since subcommand is required
    }

    Ok(())
}
