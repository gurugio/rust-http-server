use anyhow::Result;
use http::{Request, Response};
use hyper::{server::conn::Http, service::service_fn, Body};
use hyper::{Method, StatusCode};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
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

async fn http_response(req: Request<Body>) -> Result<Response<Body>> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("start http handling {:?}: {:?}", since_the_epoch, &req);

    let mut response = Response::new(Body::empty());

    match req.method() {
        &Method::GET => response_get(req, &mut response).await?,
        &Method::PUT => response_put(req, &mut response).await?,
        &Method::POST => response_post(req, &mut response).await?,
        &Method::DELETE => response_delete(req, &mut response).await?,
        _ => {
            *response.body_mut() = Body::from("Unidentified request-method");
            *response.status_mut() = StatusCode::NOT_IMPLEMENTED;
        }
    }

    Ok(response)
}

async fn response_delete(req: Request<Body>, response: &mut Response<Body>) -> Result<()> {
    let uri = req.uri().to_string();
    let _ = fs::remove_file(&uri[1..]);
    *response.status_mut() = StatusCode::ACCEPTED;
    Ok(())
}

async fn response_post(req: Request<Body>, response: &mut Response<Body>) -> Result<()> {
    let uri = req.uri().to_string();
    let contents = hyper::body::to_bytes(req.into_body()).await?;
    let mut file = File::create(&uri[1..])?;

    file.write_all(&contents)?;
    *response.status_mut() = StatusCode::ACCEPTED;
    Ok(())
}

async fn response_put(req: Request<Body>, response: &mut Response<Body>) -> Result<()> {
    let uri = req.uri().to_string();
    let contents = hyper::body::to_bytes(req.into_body()).await?;
    // open file with write permission
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&uri[1..])?;
    // over-write data
    file.write_all(&contents)?;
    *response.status_mut() = StatusCode::ACCEPTED;
    Ok(())
}

async fn response_get(req: Request<Body>, response: &mut Response<Body>) -> Result<()> {
    let uri = req.uri().to_string();
    let mut fs = File::open(&uri[1..])?;
    let mut contents = String::new();

    fs.read_to_string(&mut contents)?;
    *response.body_mut() = Body::from(contents);
    *response.status_mut() = StatusCode::ACCEPTED;
    Ok(())
}
