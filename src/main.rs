use http::{Request, Response};
use hyper::{server::conn::Http, service::service_fn, Body};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

///
/// How to test
/// run 2 terminals:
/// terminal-1: cargo run
/// terminal-2: for i in {0..10}; do curl http://127.0.0.1:8080/ & done
/// See that http_response starts immediately for each connection without sleeping 5 seconds.
///
/// $ cargo run
/// start http handling 1668004667.467865888s
/// start http handling 1668004667.467865888s
/// start http handling 1668004667.467865888s
/// start http handling 1668004667.467867354s
/// start http handling 1668004667.467936009s
/// start http handling 1668004667.467867354s
/// start http handling 1668004667.46789599s

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();

    let tcp_listener = TcpListener::bind(addr).await?;
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(http_err) = Http::new()
                .serve_connection(tcp_stream, service_fn(http_response))
                .await
            {
                eprintln!("Error while serving HTTP connection: {}", http_err);
            }
        });
    }
}

async fn http_response(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("start http handling {:?}", since_the_epoch);
    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok(Response::new(Body::from("Hello World!\n")))
}
