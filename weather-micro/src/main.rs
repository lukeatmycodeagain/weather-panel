use dotenvy::dotenv;
use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Client, Request, Response, Server, Uri};
use serde_json::Value;
use std::convert::Infallible;
use weather_utils::Weather;

// Rust has cooler enums than C++
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum Query {
    Weather(weather_utils::WeatherQuery),
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Handle only `/` requests for weather data
    if req.uri().path() == "/" {
        let body_bytes = to_bytes(req.into_body()).await.unwrap_or_default();
        let query: Result<Query, _> = serde_json::from_slice(&body_bytes);
        println!("Query recieved by micro is: {:#?}", query);
        // route query variants to handlers
        let result = match query {
            Ok(Query::Weather(weather_query)) => handle_weather_query(weather_query).await,
            Err(_) => {
                println!("Invalid query!!");
                Err("Invalid Query".to_string())
            }
        };

        let response = match result {
            Ok(weather) => Response::builder()
                .status(200)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&weather).unwrap()))
                .unwrap(),
            Err(error_message) => Response::builder()
                .status(400)
                .body(Body::from(error_message))
                .unwrap(),
        };

        // Apply CORS headers
        let mut response = response;
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "Content-Type".parse().unwrap(),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "POST, OPTIONS".parse().unwrap(),
        );

        return Ok(response);
    }

    // Respond with 404 for unknown routes
    Ok(Response::builder()
        .status(404)
        .body(Body::from("Not Found"))
        .unwrap())
}

async fn handle_weather_query(query: weather_utils::WeatherQuery) -> Result<Weather, String> {
    let api_key = std::env::var("OPEN_WEATHER_API").map_err(|_| "Missing API key".to_string())?;
    let api_url = format!(
        "http://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&exclude=minutely",
        query.latitude, query.longitude, api_key
    );

    let client = Client::new();
    let url: Uri = api_url.parse::<Uri>().map_err(|e| e.to_string())?;
    let res = client.get(url).await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("API call failed with status: {}", res.status()));
    }

    let body_bytes = to_bytes(res.into_body()).await.map_err(|e| e.to_string())?;
    let external_data: Value = serde_json::from_slice(&body_bytes).map_err(|e| e.to_string())?;

    let weather = Weather {
        time: external_data["current"]["dt"].to_string(),
        temperature: external_data["current"]["temp"]
            .as_f64()
            .ok_or("Missing temperature")?,
    };

    Ok(weather)
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

    println!("Weather Microservice running at {}:{}", address, port);

    if let Err(e) = server.await {
        eprintln!("Microservice error: {}", e);
    }
}
