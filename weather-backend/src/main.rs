use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Request, Response, Server, Client, Uri};
use hyper::body::to_bytes;
use std::convert::Infallible;
use std::error::Error;

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Create the response with CORS headers
    println!("Incoming request: {req:#?}");
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

static OPEN_WEATHER_API_KEY: &str = "628eed24f6e68a1416e17548105de0a4";

async fn fetch_data() -> Result<String, Box<dyn Error>>
{
        // Create a Hyper client
        let lat = 51.049999;
        let lon = -114.066666;
        
        let api_url = format!(
            "http://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&appid={}",
            lat, lon, OPEN_WEATHER_API_KEY,
        );
    
        // Create a Hyper client
        let client = Client::new();
    
        // Parse the URL
        let url: Uri = api_url.parse()?;
    
        // Send a GET request
        let res  = client.get(url).await?;
    
        // Check if the status is successful
        if !res.status().is_success() {
            return Err(format!("API call failed with status: {}", res.status()).into());
        } 

        
        // Read the body of the response
        let body_bytes = to_bytes(res.into_body()).await?;
        let response_string = String::from_utf8(body_bytes.to_vec())?;        
        // Print the JSON response
        //println!("Weather Data: {}", serde_json::to_string_pretty(&json)?);
        
        Ok(response_string)
}

#[tokio::main]
async fn main() {
    let make_svc =
    make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let addr = ([127, 0, 0, 1], 8080).into();
    
    let server = Server::bind(&addr).serve(make_svc);
    
    let result = fetch_data().await;
    match result {
        Ok(_thing) => println!("Success!!"),
        Err(error) => println!("There is an error: {error}"),
    };

    println!("Server running at http://127.0.0.1:8080");

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
