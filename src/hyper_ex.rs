fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(app()) {
        Ok(_) => println!("done"),
        Err(e) => println!("err:{:?}", e),
    };
}

async fn app() -> Result<()> {
    /*let service = make_service_fn(move |_| async {
        Ok::<_, hyper::Error>(service_fn(move |req| response(req)))
    });*/
    let make_service = Shared::new(service_fn(response));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    //let server = Server::bind(&addr);

    loop {
        let server = Server::bind(&addr).serve(make_service.clone());
        //let per_serv = server.clone().serve(service);
        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("server error: {:?}", e);
            }
        });
    }
}

pub async fn response(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from("hello"))),
        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn sleepy(Seconds(seconds): Seconds) -> Result<impl warp::Reply, Infallible> {
    tokio::time::sleep(Duration::from_secs(seconds)).await;
    Ok(format!("I waited {} seconds!", seconds))
}

/// A newtype to enforce our maximum allowed seconds.
struct Seconds(u64);

impl FromStr for Seconds {
    type Err = ();
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        src.parse::<u64>().map_err(|_| ()).and_then(|num| {
            if num <= 500 {
                Ok(Seconds(num))
            } else {
                Err(())
            }
        })
    }
}
