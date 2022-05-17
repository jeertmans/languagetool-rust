use clap::{CommandFactory, FromArgMatches};
use languagetool_rust::*;

#[tokio::main]
async fn main() {
    let matches = ServerCli::command()
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
                .about("LanguageTool [GET] languages request"),
        )
        .subcommand(
            WordsRequest::command()
                .name("words")
                .author(clap::crate_authors!())
                .subcommand_negates_reqs(true)
                .subcommand(WordsAddRequest::command().name("add"))
                .subcommand(WordsDeleteRequest::command().name("delete")),
        )
        .get_matches();

    let server = Server::from_cli(ServerCli::from_arg_matches(&matches).unwrap());

    match matches.subcommand() {
        Some(("check", sub_matches)) => {
            let req = CheckRequest::from_arg_matches(sub_matches).unwrap();
            println!("{:?}", server.check(&req).await);
        }
        Some(("languages", _sub_matches)) => {
            println!("{:?}", server.languages().await);
        }
        Some(("words", sub_matches)) => match sub_matches.subcommand() {
            Some(("add", sub_matches)) => {
                let req = WordsAddRequest::from_arg_matches(sub_matches).unwrap();
                println!("{:?}", server.words_add(&req).await);
            }
            Some(("delete", sub_matches)) => {
                let req = WordsDeleteRequest::from_arg_matches(sub_matches).unwrap();
                println!("{:?}", server.words_delete(&req).await);
            }
            _ => {
                let req = WordsRequest::from_arg_matches(sub_matches).unwrap();
                println!("{:?}", server.words(&req).await);
            }
        },
        _ => unreachable!(),
    }
}