use clap::Parser;
use languagetool_rust::{cli::Cli, error::Result};

#[tokio::main]
async fn main() {
    if let Err(e) = try_main().await {
        eprintln!("{e}");
        std::process::exit(2);
    }
}

async fn try_main() -> Result<()> {
    let cli = Cli::parse();
    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbose.log_level_filter())
        .init();
    cli.execute().await
}
