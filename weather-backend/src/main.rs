use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Request, Response, Server};
use std::convert::Infallible;

async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Create the response with CORS headers
    let mut response = Response::new(Body::from("Hello, World from Rust and Luke!"));
    response
        .headers_mut()
        .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "Content-Type".parse().unwrap(),
    );
    response
        .headers_mut()
        .insert(header::ACCESS_CONTROL_ALLOW_METHODS, "GET".parse().unwrap());
    Ok(response)
}

#[tokio::main]
async fn main() {
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let addr = ([127, 0, 0, 1], 8080).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running at http://127.0.0.1:8080");

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
