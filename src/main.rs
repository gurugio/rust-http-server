use http::{Request, Response};
use hyper::{server::conn::Http, service::service_fn, Body};
use hyper::{Method, StatusCode};
use std::str;
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
///
/// COMMAND: POST
/// curl -d '{"key1":"value1", "key2":"value2"}' -H "Content-Type: application/json" -X POST http://localhost:8080/data
///
/// COMMAND: GET
/// curl -X GET http://localhost:8080/data

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

async fn http_response(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("start http handling {:?}: {:?}", since_the_epoch, &req);

    let mut response = Response::new(Body::empty());

    match req.method() {
        &Method::GET => {
            let uri = req.uri();
            println!("uri={}", uri);
            *response.body_mut() = Body::from("Hello World!\n");
        }
        &Method::PUT => *response.body_mut() = Body::from("put!"),
        &Method::POST => {
            let uri = req.uri();
            println!("uri={}", uri);
            if let Ok(contents) = hyper::body::to_bytes(req.into_body()).await {
                let c = str::from_utf8(&contents).unwrap();
                println!("contents={}", c);
            } else {
                println!("parse error from json contents");
            }

            *response.status_mut() = StatusCode::ACCEPTED;
        }
        &Method::DELETE => *response.body_mut() = Body::from("delete!"),
        _ => *response.status_mut() = StatusCode::NOT_IMPLEMENTED,
    }

    Ok(response)
}
