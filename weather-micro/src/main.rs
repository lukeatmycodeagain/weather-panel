use chrono::{DateTime, Local};
use dotenvy::dotenv;
use hyper::{header, Body, Client, Request, Response, Server, Uri};
use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use serde_json::Value;
use std::convert::Infallible;
use weather_utils::Weather;

// Query types for easily adding microservice endpoints as they are added
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum Query {
    Weather(weather_utils::WeatherQuery),
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

    if let Err(e) = server.await {
        eprintln!("Microservice error: {}", e);
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Handle only `/` requests for weather data
    if req.uri().path() == "/" {
        let body_bytes = to_bytes(req.into_body()).await.unwrap_or_default();
        let query: Result<Query, _> = serde_json::from_slice(&body_bytes);

        // route query variants to handlers, can easily add microservice query variants here
        let result = match query {
            Ok(Query::Weather(weather_query)) => handle_weather_query(weather_query).await,
            Err(_) => {
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

        // Apply CORS headers, TODO: what is CORS, why does it matter, how does it affect production environments
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

    // call the external api
    let res = call_external(api_url).await;

    let result = match res {
        Ok(response) => {
            let external_data =  parse_response_to_json(response).await?;
            let location_name =
                match get_location_name(api_key, query.latitude, query.longitude).await {
                    Ok(name) => name,
                    Err(message) => {
                        println!("Error message: {message}");
                        "Unknown Place".to_string()
                    }
                };
            let weather = pack_weather(external_data, location_name);

            Ok(weather)
        }
        Err(message) => Err(format!("API call failed with status: {}", message)),
    };
    result
}

async fn get_location_name(
    api_key: String,
    latitude: f64,
    longitude: f64,
) -> Result<String, String> {
    let api_url = format!("http://api.openweathermap.org/geo/1.0/reverse?lat={latitude}&lon={longitude}&appid={api_key}");

    // call the external api
    let call = call_external(api_url).await;

    let result = match call {
        Ok(response) => {
            let external_data = parse_response_to_json(response).await?;
            if let Some(result) = external_data.as_array() {
                if result.is_empty() {
                    return Ok("... I dunno, the ocean or a desert maybe?".to_string());
                } else {
                    // this should be improved in the future when I feel like pulling my hair out over Strings
                    let location_trim = result[0]["name"]
                        .to_string()
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    let location_name = format!("... {}", location_trim);
                    return Ok(location_name);
                }
            } else {
                return Err(
                    "Check if API has been updated!! No longer receiving an array".to_string(),
                );
            }
        }
        Err(message) => Err(format!(
            "Reverse geocoding API call failed with status: {}",
            message
        )),
    };
    result
}

async fn call_external(url: String) -> Result<Response<Body>, String> {
    let client = Client::new();
    let url: Uri = url.parse::<Uri>().map_err(|e| e.to_string())?;
    let response = client.get(url).await.map_err(|e| e.to_string());
    return response;
}

async fn parse_response_to_json(response: Response<Body>) -> Result<Value, String> {
    let body_bytes = to_bytes(response.into_body())
        .await
        .map_err(|e| e.to_string())?;
    let external_data: Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| e.to_string())?;
    Ok(external_data)
}

fn pack_weather(data: Value, location: String) -> Weather {
    return Weather {
        time: convert_to_human(data["current"]["dt"].to_string()),
        temperature: data["current"]["temp"].as_f64().unwrap_or_default(),
        feels_like: data["current"]["feels_like"].as_f64().unwrap_or_default(),
        location_name: location,
        description: data["daily"][0]["summary"]
            .to_string()
            .trim()
            .trim_matches('"')
            .to_string(),
    };
}

fn convert_to_human(unix_time: String) -> String {
    let unix_seconds: i64 = unix_time.parse().unwrap_or(0); // Example Unix timestamp (in seconds)

    // Convert Unix seconds to chrono types
    let datetime = DateTime::from_timestamp(unix_seconds, 0);
    if let Some(utc_time) = datetime {
        let local_time = utc_time.with_timezone(&Local);
        return local_time.format("%r, %A %B %e %Y ").to_string();
    } else {
        return "Current Time Failed to convert".to_string();
    }
}

//TODO: Once there are multiple query types, extract logic to separate modules