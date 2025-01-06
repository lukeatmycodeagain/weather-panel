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
    let api_key = weather_utils::get_env_var("OPEN_WEATHER_API", "".to_string());
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

    let location_name = match get_location_name(api_key, query.latitude, query.longitude).await {
        Ok(name) => name,
        Err(message) => {
            println!("Error message: {message}");
            "Unknown Place".to_string()
        }
    };

    let weather = Weather {
        time: external_data["current"]["dt"].to_string(),
        temperature: external_data["current"]["temp"]
            .as_f64()
            .ok_or("Missing temperature")?,
        location_name: location_name,
    };

    Ok(weather)
}

async fn get_location_name(
    api_key: String,
    latitude: f64,
    longitude: f64,
) -> Result<String, String> {
    let api_url = format!("http://api.openweathermap.org/geo/1.0/reverse?lat={latitude}&lon={longitude}&appid={api_key}");
    let client = Client::new();
    let url: Uri = api_url.parse::<Uri>().map_err(|e| e.to_string())?;
    let res = client.get(url).await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!(
            "Reverse geocoding API call failed with status: {}",
            res.status()
        ));
    }

    let body_bytes = to_bytes(res.into_body()).await.map_err(|e| e.to_string())?;
    let external_data: Value = serde_json::from_slice(&body_bytes).map_err(|e| e.to_string())?;
    if let Some(result) = external_data.as_array() {
        if result.is_empty() {
            return Ok("...I dunno, the ocean or a desert maybe?".to_string())
        } else {
            let location_name = result[0]["name"].to_string();
            return Ok(location_name);
        }
    } else {
        return Err("Check if API has been updated!! No longer receiving an array".to_string());
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let address = weather_utils::ip_configuration();
    let port: u16 = weather_utils::get_env_var("WEATHER_MICROSERVICE_PORT", 8080);

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let addr = (address, port).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Weather Microservice running at {}:{}", address, port);

    if let Err(e) = server.await {
        eprintln!("Microservice error: {}", e);
    }
}
