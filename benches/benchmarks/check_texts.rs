use criterion::{black_box, criterion_group, Criterion};
use futures::future::join_all;
use languagetool_rust::{
    check::{CheckRequest, CheckResponse, CheckResponseWithContext},
    error::Error,
    server::ServerClient,
};

async fn request_until_success(req: &CheckRequest, client: &ServerClient) -> CheckResponse {
    loop {
        match client.check(req).await {
            Ok(resp) => return resp,
            Err(Error::InvalidRequest { body })
                if body == *"Error: Server overloaded, please try again later" =>
            {
                continue
            }
            Err(e) => panic!("Some unexpected error occured: {}", e),
        }
    }
}

#[tokio::main]
async fn check_text_basic(text: &str) -> CheckResponse {
    let client = ServerClient::from_env().unwrap();
    let req = CheckRequest::default().with_text(text.to_string());
    request_until_success(&req, &client).await
}

#[tokio::main]
async fn check_text_split(text: &str) -> CheckResponse {
    let client = ServerClient::from_env().unwrap();
    let lines = text.lines();

    let resps = join_all(lines.map(|line| async {
        let req = CheckRequest::default().with_text(line.to_string());
        let resp = request_until_success(&req, &client).await;
        CheckResponseWithContext::new(req.get_text(), resp)
    }))
    .await;

    resps
        .into_iter()
        .reduce(|acc, item| acc.append(item))
        .unwrap()
        .into()
}

#[macro_export]
macro_rules! compare_checks_on_file {
    ($name:ident, $filename:expr) => {
        fn $name(c: &mut Criterion) {
            let content = std::fs::read_to_string($filename).unwrap();
            let text = content.as_str();
            let name = stringify!($name);
            c.bench_function(format!("Check basic for {}", name).as_str(), |b| {
                b.iter(|| check_text_basic(black_box(text)))
            });
            c.bench_function(format!("Check split for {}", name).as_str(), |b| {
                b.iter(|| check_text_split(black_box(text)))
            });
        }
    };
}

compare_checks_on_file!(compare_checks_small_file, "./benches/small.txt");
compare_checks_on_file!(compare_checks_medium_file, "./benches/medium.txt");
compare_checks_on_file!(compare_checks_large_file, "./benches/large.txt");

criterion_group!(
    checks,
    compare_checks_small_file,
    compare_checks_medium_file,
    compare_checks_large_file
);
