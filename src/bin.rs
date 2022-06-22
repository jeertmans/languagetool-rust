use clap::{CommandFactory, FromArgMatches};
use languagetool_rust::error::Result;
use languagetool_rust::*;
use std::io::{BufRead, Write};

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
        )
        .get_matches();

    // TODO: prompt max_suggestion
    let client = ServerClient::from_arg_matches(&matches)?.with_max_suggestions(5);
    let stdout = std::io::stdout();

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

            writeln!(
                &stdout,
                "{}",
                serde_json::to_string_pretty(&client.check(&req).await?)?
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
