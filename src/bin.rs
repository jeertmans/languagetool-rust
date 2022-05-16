use clap::Parser;
use languagetool_rust::*;

#[tokio::main]
async fn main() {
    let server = Server::from_cli();
    let check = CheckRequest::default()
        .with_language("en-US")
        .with_text("I makke some mistake");
    //println!("{:?}", server.languages().await);
    println!("{:?}", server.check(&check).await);
    let args = ServerParameters::parse();

    //let args = ServerParameters { ..args, ..ServerParameters::default() };

    println!("{:?}", args);

    println!("Hello, world!");
}
