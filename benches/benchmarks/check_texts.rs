use codspeed_criterion_compat::{criterion_group, Criterion, Throughput};
use futures::future::join_all;
use languagetool_rust::{
    check::{CheckRequest, CheckResponse, CheckResponseWithContext},
    error::Error,
    server::ServerClient,
};

static FILES: [(&str, &str); 3] = [
    ("small", include_str!("../small.txt")),
    ("medium", include_str!("../medium.txt")),
    ("large", include_str!("../large.txt")),
];

async fn request_until_success(req: &CheckRequest, client: &ServerClient) -> CheckResponse {
    loop {
        match client.check(req).await {
            Ok(resp) => return resp,
            Err(Error::InvalidRequest(body))
                if body == *"Error: Server overloaded, please try again later" =>
            {
                continue;
            },
            Err(e) => panic!("Some unexpected error occurred: {}", e),
        }
    }
}

#[tokio::main]
async fn check_text_basic(text: &str) -> CheckResponse {
    let client = ServerClient::from_env().expect(
        "Please use a local server for benchmarking, and configure the environ variables to use \
         it.",
    );
    let req = CheckRequest::default().with_text(text.to_string());
    request_until_success(&req, &client).await
}

#[tokio::main]
async fn check_text_split(text: &str) -> CheckResponse {
    let client = ServerClient::from_env().expect(
        "Please use a local server for benchmarking, and configure the environ variables to use \
         it.",
    );
    let lines = text.lines();

    let resps = join_all(lines.map(|line| {
        async {
            let req = CheckRequest::default().with_text(line.to_string());
            let resp = request_until_success(&req, &client).await;
            CheckResponseWithContext::new(req.get_text(), resp)
        }
    }))
    .await;

    resps
        .into_iter()
        .reduce(|acc, item| acc.append(item))
        .unwrap()
        .into()
}

fn bench_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic");

    for (name, source) in FILES {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(name, &source, |b, &s| b.iter(|| check_text_basic(s)));
    }
}

fn bench_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("split");

    for (name, source) in FILES {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(name, &source, |b, &s| b.iter(|| check_text_split(s)));
    }
}

criterion_group!(checks, bench_basic, bench_split,);
