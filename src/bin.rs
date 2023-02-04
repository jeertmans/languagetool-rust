use clap::Parser;
use languagetool_rust::cli::Cli;
use languagetool_rust::error::Result;

#[tokio::main]
async fn main() {
    if let Err(e) = try_main().await {
        eprintln!("{e}");
        std::process::exit(2);
    }
}

async fn try_main() -> Result<()> {
    Cli::parse().execute().await
}
