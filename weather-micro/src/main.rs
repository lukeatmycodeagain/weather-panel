use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Client, Request, Response, Server, Uri};
use serde_json::Value;
use std::convert::Infallible;
use std::env;
use std::error::Error;
use weather_utils::Weather;
use dotenvy::dotenv;

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Handle only `/` requests for weather data
    if req.uri().path() == "/" {
        let information = fetch_data().await;
        let mut response: Response<Body>;
        if let Ok(message) = information {
            println!("message: {message}");
            response = Response::new(Body::from(message));
        } else {
            response = Response::new(Body::from("Data fetch failed"));
        };

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
        return Ok(response);
    }

    // Respond with 404 for unknown routes
    Ok(Response::builder()
        .status(404)
        .body(Body::from("Not Found"))
        .unwrap())
}

async fn fetch_data() -> Result<String, Box<dyn Error>> {
    let lat = 51.049999;
    let lon = -114.066666;

    let api_key = env::var("OPEN_WEATHER_API").unwrap_or_else(|_| "bigproblem".to_string());

    println!("Key is {api_key}");

    let api_url = format!(
        "http://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&exclude=minutely",
        lat, lon, api_key,
    );

    // Create a Hyper client
    let client = Client::new();

    // Parse the URL
    let url: Uri = api_url.parse()?;

    // Send a GET request
    let res = client.get(url).await?;

    // Check if the status is successful
    if !res.status().is_success() {
        return Err(format!("API call failed with status: {}", res.status()).into());
    }

    //let mut test_string: String = String::new();
    //test_string = "This is a test string".to_string();
    // Read the body of the response
    let body_bytes = to_bytes(res.into_body()).await?;
    let external_data: Value = serde_json::from_slice(&body_bytes)?;
    let weather = Weather {
        time: external_data["current"]["dt"].to_string(),
        temperature: external_data["current"]["temp"].as_f64().unwrap_or(0.0),
    };
    println!("Weather Data: {}", serde_json::to_string_pretty(&weather)?);
    let weather_json = serde_json::to_string(&weather)?;
    Ok(weather_json)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let address = weather_utils::ip_configuration();
    let port = weather_utils::port_from_env("WEATHER_MICROSERVICE_PORT", 8080);

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let addr = (address, port).into();

    let server = Server::bind(&addr).serve(make_svc);

    let result = fetch_data().await;
    match result {
        Ok(thing) => println!("Success!! {thing:#?}"),
        Err(error) => println!("There is an error: {error}"),
    };

    println!("Weather Microservice running at {}:{}", address, port);

    if let Err(e) = server.await {
        eprintln!("Microservice error: {}", e);
    }
}
